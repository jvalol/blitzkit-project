//! What a ray costs, over a building the size of the arcade.
//!
//! Spec 0096 says three things are untried and all three are numbers: what a
//! ray per pixel costs, what an acceleration structure over a building that
//! moves costs to build and to keep up, and whether either of those is the
//! same order as the budget the shadow maps already pay. `ray-spike` beside
//! this answered whether a ray works and what a hit returns. This one is
//! only about cost, and it is a separate binary for that reason: a file that
//! answers one question is a file you can read the answer out of.
//!
//! **A spike and not a benchmark.** Everything here is wall clock around a
//! submit and a wait, which is the dispatch plus the submission plus the
//! driver, over a scene of boxes rather than the arcade. What it is for is
//! telling a millisecond from a hundred, because that is the size of the
//! decision in front of the spec. Anything finer wants the engine's own
//! timestamps, which is spec 0067's `cost::Frame` and is not this.

use std::time::Instant;
use wgpu::util::DeviceExt;

/// About what the arcade keeps in one run, per spec 0096.
const INSTANCES: usize = 351;

/// How many of them move: the doors and bookcases that swing, the people,
/// the gate at the top of the bridge, and the water.
const MOVING: usize = 40;

/// Six more for a floor, a ceiling and four walls.
///
/// **Because a building is closed and a scatter of boxes is not.** Measured
/// against the scatter alone, three rays in four left through the gap
/// between two boxes and never came back, and a ray that leaves is the
/// cheapest kind there is: it costs a walk down the top of the structure
/// and nothing else. That is not what a reflection in a room does. With the
/// shell on, nearly every ray finds something, which is the number this is
/// for.
const SHELL: usize = 6;

/// A screen's worth of rays, which is the reflection budget: one a pixel.
const WIDE: u32 = 1920;
const TALL: u32 = 1080;

/// What the maps already pay for, per `lighting::MAX_SHADOWING_POINT_LIGHTS`
/// plus four spots and the sun. A shadow ray replaces the march for each of
/// them, so the shadow budget is this many rays a fragment where the
/// reflection budget is one.
const CASTING: u32 = 7;

/// How many times each measurement is taken, after a warm-up.
const TIMES: u32 = 20;

fn main() {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("an adapter");

    let info = adapter.get_info();

    if !adapter
        .features()
        .contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY)
    {
        println!("nothing to measure: {} has no ray queries", info.name);
        return;
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ray cost"),
        required_features: wgpu::Features::EXPERIMENTAL_RAY_QUERY,
        required_limits: adapter.limits(),
        experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
        ..Default::default()
    }))
    .expect("a device with ray queries");

    device.set_device_lost_callback(Box::new(|_, why| eprintln!("device lost: {why}")));

    println!("adapter  {} on {:?}", info.name, info.backend);
    println!(
        "scene    {} instances in a closed room, {} of them moving, {} by {} rays a pass\n",
        INSTANCES + SHELL,
        MOVING,
        WIDE,
        TALL
    );

    // A cube and a ball, which is most of what this building is made of.
    let (cube_vertices, cube_indices) = cube();
    let (ball_vertices, ball_indices) = ball(24, 16);

    let shapes = [
        Shape::new(&device, &cube_vertices, &cube_indices),
        Shape::new(&device, &ball_vertices, &ball_indices),
    ];

    let mut tlas = device.create_tlas(&wgpu::CreateTlasDescriptor {
        label: Some("the building"),
        max_instances: (INSTANCES + SHELL) as u32,
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: wgpu::AccelerationStructureUpdateMode::PreferUpdate,
    });

    for n in 0..INSTANCES {
        tlas[n] = Some(wgpu::TlasInstance::new(
            &shapes[n % shapes.len()].blas,
            placed(n, 0.0),
            n as u32,
            0xFF,
        ));
    }

    for (n, walls) in shell().into_iter().enumerate() {
        tlas[INSTANCES + n] = Some(wgpu::TlasInstance::new(
            &shapes[0].blas,
            walls,
            (INSTANCES + n) as u32,
            0xFF,
        ));
    }

    // **The shapes once and the building every time.** A BLAS is the thing
    // that is built over triangles and the arcade's triangles do not change;
    // what changes is where its boxes are, which is the TLAS.
    let over_triangles: Vec<wgpu::BlasBuildEntry> = shapes.iter().map(Shape::entry).collect();
    let shapes_took = timed(&device, &queue, 1, |encoder| {
        encoder.build_acceleration_structures(over_triangles.iter(), []);
    });

    println!("the shapes, once          {:>8.3} ms", shapes_took);

    let whole_took = timed(&device, &queue, TIMES, |encoder| {
        encoder.build_acceleration_structures([], std::iter::once(&tlas));
    });

    println!("the building, all of it   {:>8.3} ms", whole_took);

    // and again with the movers moved, which is what a frame would do
    for n in 0..MOVING {
        tlas[n] = Some(wgpu::TlasInstance::new(
            &shapes[n % shapes.len()].blas,
            placed(n, 1.0),
            n as u32,
            0xFF,
        ));
    }

    let moved_took = timed(&device, &queue, TIMES, |encoder| {
        encoder.build_acceleration_structures([], std::iter::once(&tlas));
    });

    println!("the building, {MOVING} moved    {:>8.3} ms", moved_took);

    // and the rays themselves
    let answers = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("answers"),
        size: (WIDE * TALL * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    // **And somewhere to read them back from**, because a measurement of a
    // scene the rays all miss is a measurement of nothing: a miss leaves the
    // structure early and costs a fraction of a hit.
    let read_back = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("read back"),
        size: (WIDE * TALL * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("cost.wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("cost.wgsl").into()),
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
                resource: answers.as_entire_binding(),
            },
        ],
    });

    let rays_took = timed(&device, &queue, TIMES, |encoder| {
        let mut pass = encoder.begin_compute_pass(&Default::default());

        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bound, &[]);
        pass.dispatch_workgroups(WIDE / 8, TALL / 8, 1);
    });

    let rays = (WIDE * TALL) as f64;

    // how many of them found anything
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(&answers, 0, &read_back, 0, (WIDE * TALL * 4) as u64);
    queue.submit(std::iter::once(encoder.finish()));
    read_back.slice(..).map_async(wgpu::MapMode::Read, |_| ());
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .expect("the queue drains");

    let hit = {
        let held = read_back.slice(..).get_mapped_range().expect("the answers");
        let got: &[f32] = bytemuck::cast_slice(&held);

        got.iter().filter(|t| **t > 0.0).count()
    };

    read_back.unmap();

    println!(
        "of those rays, {:.0}% hit something",
        hit as f64 / rays * 100.0
    );
    println!("one ray a pixel           {:>8.3} ms", rays_took);
    println!(
        "                          {:>8.1} million rays a second",
        rays / rays_took / 1000.0
    );
    println!();
    println!("What that comes to, at sixty frames a second, which is 16.7 ms a frame:");
    println!();
    println!(
        "  a reflection a pixel    {:>8.3} ms, {:>4.1}% of a frame",
        rays_took,
        rays_took / 16.7 * 100.0
    );
    println!(
        "  a shadow ray a lamp     {:>8.3} ms, {:>4.1}% of a frame, at {} casting",
        rays_took * CASTING as f64,
        rays_took * CASTING as f64 / 16.7 * 100.0,
        CASTING
    );
    println!(
        "  keeping the structure   {:>8.3} ms, {:>4.1}% of a frame",
        moved_took,
        moved_took / 16.7 * 100.0
    );
}

/// A shape, and the structure built over its triangles.
struct Shape {
    blas: wgpu::Blas,
    size: wgpu::BlasTriangleGeometrySizeDescriptor,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
}

impl Shape {
    fn new(device: &wgpu::Device, vertices: &[[f32; 3]], indices: &[u32]) -> Self {
        let size = wgpu::BlasTriangleGeometrySizeDescriptor {
            vertex_format: wgpu::VertexFormat::Float32x3,
            vertex_count: vertices.len() as u32,
            index_format: Some(wgpu::IndexFormat::Uint32),
            index_count: Some(indices.len() as u32),
            flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
        };

        Self {
            blas: device.create_blas(
                &wgpu::CreateBlasDescriptor {
                    label: Some("a shape"),
                    flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                    update_mode: wgpu::AccelerationStructureUpdateMode::Build,
                },
                wgpu::BlasGeometrySizeDescriptors::Triangles {
                    descriptors: vec![size.clone()],
                },
            ),
            size,
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::BLAS_INPUT,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::BLAS_INPUT,
            }),
        }
    }

    fn entry(&self) -> wgpu::BlasBuildEntry<'_> {
        wgpu::BlasBuildEntry {
            blas: &self.blas,
            geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
                size: &self.size,
                vertex_buffer: &self.vertices,
                first_vertex: 0,
                vertex_stride: 12,
                index_buffer: Some(&self.indices),
                first_index: Some(0),
                transform_buffer: None,
                transform_buffer_offset: None,
            }]),
        }
    }
}

/// The floor, the ceiling and the four walls, as six flattened cubes.
fn shell() -> Vec<[f32; 12]> {
    let (wide, high) = (52.0f32, 18.0f32);
    let thick = 1.0f32;

    vec![
        laid(wide, thick, wide, 0.0, -0.5, 0.0),
        laid(wide, thick, wide, 0.0, high, 0.0),
        laid(thick, high, wide, -wide * 0.5, high * 0.5, 0.0),
        laid(thick, high, wide, wide * 0.5, high * 0.5, 0.0),
        laid(wide, high, thick, 0.0, high * 0.5, -wide * 0.5),
        laid(wide, high, thick, 0.0, high * 0.5, wide * 0.5),
    ]
}

/// One box, scaled and put somewhere, as a row major 3x4.
fn laid(wide: f32, high: f32, deep: f32, x: f32, y: f32, z: f32) -> [f32; 12] {
    [
        wide, 0.0, 0.0, x, //
        0.0, high, 0.0, y, //
        0.0, 0.0, deep, z,
    ]
}

/// Where instance `n` sits, scattered through a volume about the size of the
/// building, and `moved` along to shift the ones that move.
fn placed(n: usize, moved: f32) -> [f32; 12] {
    let n = n as f32;
    let x = (n * 0.37).sin() * 20.0 + moved * 0.4;
    let y = (n * 0.71).sin() * 4.0 + 4.0;
    let z = (n * 0.13).sin() * 20.0;
    let size = 0.4 + (n * 0.29).sin().abs() * 1.6;

    [
        size, 0.0, 0.0, x, //
        0.0, size, 0.0, y, //
        0.0, 0.0, size, z,
    ]
}

/// Wall clock around `times` submissions of `what`, after one that is thrown
/// away, in milliseconds a go.
fn timed(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    times: u32,
    what: impl Fn(&mut wgpu::CommandEncoder),
) -> f64 {
    let run = |times: u32| {
        for _ in 0..times {
            let mut encoder = device.create_command_encoder(&Default::default());

            what(&mut encoder);
            queue.submit(std::iter::once(encoder.finish()));
        }

        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("the queue drains");
    };

    // **A warm-up that is thrown away**, because the first of anything here
    // is the shader being compiled and the allocator finding room.
    run(1);

    let from = Instant::now();

    run(times);

    from.elapsed().as_secs_f64() * 1000.0 / times as f64
}

/// A unit cube, as twelve triangles.
fn cube() -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (normal, right, up) in [
        ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
        ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
        ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
    ] {
        let base = vertices.len() as u32;

        for (a, b) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            vertices.push([
                normal[0] * 0.5 + (right[0] * a + up[0] * b) * 0.5,
                normal[1] * 0.5 + (right[1] * a + up[1] * b) * 0.5,
                normal[2] * 0.5 + (right[2] * a + up[2] * b) * 0.5,
            ]);
        }

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    (vertices, indices)
}

/// A unit ball, as rings of quads, which is the engine's own sphere.
fn ball(around: u32, up: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    use std::f32::consts::{PI, TAU};

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=up {
        let down = ring as f32 / up as f32 * PI;

        for step in 0..=around {
            let round = step as f32 / around as f32 * TAU;

            vertices.push([
                down.sin() * round.cos() * 0.5,
                down.cos() * 0.5,
                down.sin() * round.sin() * 0.5,
            ]);
        }
    }

    let across = around + 1;

    for ring in 0..up {
        for step in 0..around {
            let a = ring * across + step;
            let b = a + across;

            indices.extend_from_slice(&[a, b, b + 1, a, b + 1, a + 1]);
        }
    }

    (vertices, indices)
}
