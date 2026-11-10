//! Does a ray query work at all on this stack?
//!
//! Spec 0096 is a draft because wgpu documents EXPERIMENTAL_RAY_QUERY as
//! having major bugs and expecting breaking changes. This asks the cheapest
//! question there is, outside the engine: build an acceleration structure
//! over one triangle, trace four rays at it, and see whether the answers are
//! the ones arithmetic already knows.
//!
//! Two should hit and two should miss. Anything else is the answer.

use wgpu::util::DeviceExt;

/// One triangle in the z = 0 plane, a unit across, sitting at the origin.
const TRIANGLE: [[f32; 3]; 3] = [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]];

/// Four rays, as (from, along) pairs, aimed along -z at z = 0.
///
/// The first two cross the triangle: its centroid is (0, -1/3) and (0, 0) is
/// inside it. The second two are outside its edges by a wide margin.
const RAYS: [[f32; 4]; 8] = [
    [0.0, 0.0, 5.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
    [0.0, -0.5, 5.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
    [5.0, 5.0, 5.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
    [0.0, 3.0, 5.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
];

fn main() {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("an adapter");

    let info = adapter.get_info();
    let features = adapter.features();
    let query = features.contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY);
    let vertex_return = features.contains(wgpu::Features::EXPERIMENTAL_RAY_HIT_VERTEX_RETURN);

    println!("adapter            {} on {:?}", info.name, info.backend);
    println!("ray_query          {query}");
    println!("hit_vertex_return  {vertex_return}");

    if !query {
        println!("\nnothing to spike: this adapter has no ray queries");
        return;
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ray spike"),
        required_features: wgpu::Features::EXPERIMENTAL_RAY_QUERY,
        // the defaults put max_blas_geometry_count at 0, so an
        // acceleration structure cannot be made with them
        required_limits: adapter.limits(),
        // wgpu 30 will not hand out an EXPERIMENTAL_ feature without this,
        // and the constructor is `const unsafe fn` on purpose: its contract
        // is that there may be UB-containing bugs reachable from otherwise
        // safe code. That is a fact about adding this to an engine sixteen
        // games build against, not a formality.
        experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
        ..Default::default()
    }))
    .expect("a device with ray queries");

    device.set_device_lost_callback(Box::new(|_, why| eprintln!("device lost: {why}")));

    let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("triangle"),
        contents: bytemuck::cast_slice(&TRIANGLE),
        usage: wgpu::BufferUsages::BLAS_INPUT,
    });

    let size = wgpu::BlasTriangleGeometrySizeDescriptor {
        vertex_format: wgpu::VertexFormat::Float32x3,
        vertex_count: 3,
        index_format: None,
        index_count: None,
        flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
    };

    let blas = device.create_blas(
        &wgpu::CreateBlasDescriptor {
            label: Some("one triangle"),
            flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: wgpu::AccelerationStructureUpdateMode::Build,
        },
        wgpu::BlasGeometrySizeDescriptors::Triangles {
            descriptors: vec![size.clone()],
        },
    );

    let mut tlas = device.create_tlas(&wgpu::CreateTlasDescriptor {
        label: Some("the world"),
        max_instances: 1,
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
    });

    // identity, row major 3x4, and a custom index of 7 to see whether it
    // comes back
    tlas[0] = Some(wgpu::TlasInstance::new(
        &blas,
        [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0,
        ],
        7,
        0xFF,
    ));

    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.build_acceleration_structures(
        std::iter::once(&wgpu::BlasBuildEntry {
            blas: &blas,
            geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
                size: &size,
                vertex_buffer: &vertices,
                first_vertex: 0,
                vertex_stride: 12,
                index_buffer: None,
                first_index: None,
                transform_buffer: None,
                transform_buffer_offset: None,
            }]),
        }),
        std::iter::once(&tlas),
    );
    queue.submit(std::iter::once(encoder.finish()));

    let asked = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("rays"),
        contents: bytemuck::cast_slice(&RAYS),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let answers = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("answers"),
        size: 4 * 4 * 4 * 3,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let read_back = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("read back"),
        size: 4 * 4 * 4 * 3,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ray.wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("ray.wgsl").into()),
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("trace"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let bound = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::AccelerationStructure(&tlas),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: asked.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: answers.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bound, &[]);
        pass.dispatch_workgroups(4, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&answers, 0, &read_back, 0, 4 * 4 * 4 * 3);
    queue.submit(std::iter::once(encoder.finish()));

    read_back.slice(..).map_async(wgpu::MapMode::Read, |_| ());
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .expect("the queue drains");

    let held = read_back
        .slice(..)
        .get_mapped_range()
        .expect("the answers map");
    let got: &[f32] = bytemuck::cast_slice(&held);

    println!("\n ray  hit     t       triangle  barycentrics        where it landed");
    let expected = [true, true, false, false];
    let mut right = 0;

    // the triangle again, to put the hit back together on the CPU
    let v = TRIANGLE;

    for n in 0..4 {
        let a = &got[n * 12..n * 12 + 12];
        let hit = a[0] > 0.5;
        let (u, w) = (a[4], a[5]);
        let weights = [a[8], a[9], a[10]];

        let put_back = [0, 1, 2].map(|k| {
            weights[0] * v[0][k] + weights[1] * v[1][k] + weights[2] * v[2][k]
        });

        println!(
            "  {n}   {:<6}  {:<6.3}  {:<8}  {:<6.3} {:<6.3}      {:>6.3} {:>6.3} {:>6.3}",
            hit, a[1], a[2] as i32, u, w, put_back[0], put_back[1], put_back[2]
        );

        if hit == expected[n] {
            right += 1;
        }

        // the ray is RAYS[n*2] going along RAYS[n*2+1], so where it struck is
        // arithmetic the GPU never saw
        if hit {
            let from = RAYS[n * 2];
            let along = RAYS[n * 2 + 1];
            let should = [0, 1, 2].map(|k| from[k] + along[k] * a[1]);
            let off = (0..3)
                .map(|k| (put_back[k] - should[k]).abs())
                .fold(0.0f32, f32::max);

            println!(
                "        ray says {:>6.3} {:>6.3} {:>6.3}, apart by {:.5}",
                should[0], should[1], should[2], off
            );
        }
    }

    println!("\n{right} of 4 rays answered the way arithmetic says they should");
    println!(
        "custom_data came back as {} and was set to 7",
        got[3]
    );
}
