//! Can a shader read the surface it hit, out of the buffers the acceleration
//! structure was built from?
//!
//! Spec 0096's first spike asked whether a ray query works at all on wgpu 30
//! and Metal. It does, and the answer it left behind was that
//! `EXPERIMENTAL_RAY_HIT_VERTEX_RETURN` is false here, so a hit says which
//! triangle and where on it and nothing about the surface. The spec then said
//! the tables are the work and that binding them beside the structure was
//! untried.
//!
//! This is that. A quad of two triangles with a different normal at every
//! corner and a uv at every corner, two instances of it, one of them scaled
//! unevenly, and a per instance table of what the renderer calls a material.
//! Five rays. The shader reads the vertex and index buffers at the hit and
//! builds the surface; this file does the same arithmetic on the CPU, from
//! the same numbers, and prints how far apart they are.
//!
//! **Three questions, and the third was not in the spec.** Whether a buffer
//! can feed a BLAS and be read as storage at the same time. Whether the
//! lookup at the hit comes out right. And whether `object_to_world` carries
//! a normal, which the spec says in passing and which is false the moment
//! anything is scaled unevenly: a normal wants the inverse transpose, and a
//! hit hands over `world_to_object`, which is the inverse already.

use wgpu::util::DeviceExt;

/// A quad in the z = 0 plane, interleaved the way a renderer keeps a vertex
/// buffer: position, normal, uv, eight floats and 32 bytes to a vertex.
///
/// **The normals lean outward rather than all facing +z.** Four copies of
/// one normal interpolates to itself whatever the weights are, which is a
/// check that passes without reading the buffer at all.
const VERTICES: [[f32; 8]; 4] = [
    [-1.0, -1.0, 0.0, -0.4, -0.4, 1.0, 0.0, 0.0],
    [1.0, -1.0, 0.0, 0.4, -0.4, 1.0, 1.0, 0.0],
    [1.0, 1.0, 0.0, 0.4, 0.4, 1.0, 1.0, 1.0],
    [-1.0, 1.0, 0.0, -0.4, 0.4, 1.0, 0.0, 1.0],
];

/// Wound counter-clockwise seen from +z, so a ray from behind reports
/// `front_face` false. The second triangle is there to make
/// `primitive_index` mean something.
const INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

/// Row major 3x4, the layout `TlasInstance` takes.
///
/// The second is scaled three times in y, which is what makes the normal
/// question measurable: a uniform scale leaves a normal's direction alone.
const TRANSFORMS: [[f32; 12]; 2] = [
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0,
    ],
    [
        1.0, 0.0, 0.0, 4.0, //
        0.0, 3.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0,
    ],
];

/// Whatever the builder wants to find its instance by. Not a subscript: the
/// shader keys the material table by `instance_index` and reports these so
/// that both can be seen to arrive.
const CUSTOM_DATA: [u32; 2] = [11, 22];

/// A row per instance: colour, then shininess, specular, texture, spare.
/// This is spec 0012's `Instance` with the transform taken out, because the
/// transform is the one thing a hit already hands over.
const MATERIALS: [[f32; 8]; 2] = [
    [0.80, 0.20, 0.15, 1.0, 900.0, 0.35, 3.0, 0.0],
    [0.10, 0.45, 0.85, 1.0, 64.0, 0.90, 7.0, 0.0],
];

/// (from, along) pairs. Four hits and a miss.
///
/// Two at the unscaled instance, one in each triangle. Two at the scaled
/// one, the second of them from behind. The last goes past everything.
const RAYS: [[f32; 4]; 10] = [
    [0.0, -0.5, 5.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
    [-0.5, 0.0, 5.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
    [4.0, 1.5, 5.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
    [4.0, -1.5, -5.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [10.0, 10.0, 5.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
];

const RAY_COUNT: usize = RAYS.len() / 2;

/// Eight `vec4`s of answer per ray, which the shader writes and the printing
/// below reads back by the same names.
const SLOTS: usize = 8;

const ANSWER_BYTES: u64 = (RAY_COUNT * SLOTS * 4 * 4) as u64;

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

    // **One buffer, two jobs, which is the first question.** BLAS_INPUT so
    // the structure can be built over the positions at a stride, STORAGE so
    // the shader can read the normals and uvs beside them. If wgpu refuses
    // the pair, a renderer keeps two copies of every vertex it draws.
    let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertices"),
        contents: bytemuck::cast_slice(&VERTICES),
        usage: wgpu::BufferUsages::BLAS_INPUT | wgpu::BufferUsages::STORAGE,
    });
    let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indices"),
        contents: bytemuck::cast_slice(&INDICES),
        usage: wgpu::BufferUsages::BLAS_INPUT | wgpu::BufferUsages::STORAGE,
    });
    let materials = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("materials"),
        contents: bytemuck::cast_slice(&MATERIALS),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let size = wgpu::BlasTriangleGeometrySizeDescriptor {
        vertex_format: wgpu::VertexFormat::Float32x3,
        vertex_count: VERTICES.len() as u32,
        index_format: Some(wgpu::IndexFormat::Uint32),
        index_count: Some(INDICES.len() as u32),
        flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
    };

    let blas = device.create_blas(
        &wgpu::CreateBlasDescriptor {
            label: Some("a quad"),
            flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: wgpu::AccelerationStructureUpdateMode::Build,
        },
        wgpu::BlasGeometrySizeDescriptors::Triangles {
            descriptors: vec![size.clone()],
        },
    );

    let mut tlas = device.create_tlas(&wgpu::CreateTlasDescriptor {
        label: Some("the world"),
        max_instances: TRANSFORMS.len() as u32,
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
    });

    for (n, transform) in TRANSFORMS.iter().enumerate() {
        tlas[n] = Some(wgpu::TlasInstance::new(
            &blas,
            *transform,
            CUSTOM_DATA[n],
            0xFF,
        ));
    }

    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.build_acceleration_structures(
        std::iter::once(&wgpu::BlasBuildEntry {
            blas: &blas,
            geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
                size: &size,
                vertex_buffer: &vertices,
                first_vertex: 0,
                vertex_stride: 32,
                index_buffer: Some(&indices),
                first_index: Some(0),
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
        size: ANSWER_BYTES,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let read_back = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("read back"),
        size: ANSWER_BYTES,
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
            wgpu::BindGroupEntry {
                binding: 3,
                resource: vertices.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: indices.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: materials.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bound, &[]);
        pass.dispatch_workgroups(RAY_COUNT as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&answers, 0, &read_back, 0, ANSWER_BYTES);
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

    report(got);
}

/// Everything the shader said, against the same arithmetic done here.
fn report(got: &[f32]) {
    let should_hit = [true, true, true, true, false];
    let mut agreed = 0;
    let mut worst = 0.0f32;

    for n in 0..RAY_COUNT {
        let slot = |k: usize| {
            let at = (n * SLOTS + k) * 4;

            [got[at], got[at + 1], got[at + 2], got[at + 3]]
        };

        let head = slot(0);
        let hit = head[0] > 0.5;
        let t = head[1];
        let triangle = head[2] as usize;
        let custom = head[3] as u32;

        println!("\n== ray {n}");

        if hit != should_hit[n] {
            println!("   {hit}, and arithmetic says {}", should_hit[n]);
            continue;
        }

        if !hit {
            println!("   missed, as it should");
            agreed += 1;
            continue;
        }

        let there = slot(1);
        let upright = slot(2);
        let naive = slot(3);
        let uv = slot(4);
        let colour = slot(5);
        let rest = slot(6);
        let bary = slot(7);

        let which = uv[2] as usize;
        let weights = [1.0 - bary[0] - bary[1], bary[0], bary[1]];

        // the same sum, here, out of the same constants
        let mix = |field: usize| {
            [0, 1, 2].map(|k| {
                (0..3)
                    .map(|v| weights[v] * VERTICES[INDICES[triangle * 3 + v] as usize][field + k])
                    .sum::<f32>()
            })
        };

        let on_it = mix(0);
        let leaning = mix(3);
        let mine = MATERIALS[which];

        let to_world = |p: [f32; 3]| {
            let m = TRANSFORMS[which];

            [0, 1, 2]
                .map(|r| m[r * 4] * p[0] + m[r * 4 + 1] * p[1] + m[r * 4 + 2] * p[2] + m[r * 4 + 3])
        };

        let in_world = to_world(on_it);
        let from = RAYS[n * 2];
        let along = RAYS[n * 2 + 1];
        let along_the_ray = [0, 1, 2].map(|k| from[k] + along[k] * t);

        // **The normal, by the matrix a normal actually takes.** The inverse
        // transpose of the linear part, worked out here rather than taken
        // from the hit, so the shader's answer has something to be wrong
        // against.
        let upright_here = normalize(inverse_transpose(TRANSFORMS[which], leaning));
        let naive_here = normalize(linear(TRANSFORMS[which], leaning));

        let uv_here = [0, 1].map(|k| {
            (0..3)
                .map(|v| weights[v] * VERTICES[INDICES[triangle * 3 + v] as usize][6 + k])
                .sum::<f32>()
        });

        let apart = [
            ("where it landed", gap(&there[..3], &in_world)),
            ("and against the ray itself", gap(&in_world, &along_the_ray)),
            ("the normal", gap(&upright[..3], &upright_here)),
            (
                "the normal by object_to_world",
                gap(&naive[..3], &naive_here),
            ),
            ("the uv", gap(&uv[..2], &uv_here)),
            ("the material", gap(&colour[..3], &mine[..3])),
        ];

        println!(
            "   t {t:.3}, triangle {triangle}, instance {which}, custom_data {custom}, front {}",
            there[3] > 0.5
        );
        println!(
            "   at    {:>7.3} {:>7.3} {:>7.3}",
            there[0], there[1], there[2]
        );
        println!(
            "   normal{:>7.3} {:>7.3} {:>7.3}   by object_to_world {:>6.3} {:>6.3} {:>6.3}",
            upright[0], upright[1], upright[2], naive[0], naive[1], naive[2]
        );
        println!(
            "   uv    {:>7.3} {:>7.3}   colour {:.2} {:.2} {:.2}, shininess {}, specular {:.2}, texture {}",
            uv[0], uv[1], colour[0], colour[1], colour[2], rest[0], rest[1], rest[2] as i32
        );

        let mut right = true;

        for (what, off) in apart {
            if off > 1e-4 {
                println!("   {what}: apart by {off:.5}");
                right = false;
            }
            worst = worst.max(off);
        }

        let leaning_off = angle_between(&upright_here, &naive_here);

        if leaning_off > 0.5 {
            println!(
                "   object_to_world leans that normal {leaning_off:.1} degrees off, which is the scale"
            );
        }

        if right {
            agreed += 1;
        }
    }

    // **Every field, not just whether it hit.** This said 5 of 5 while
    // printing four rays of disagreement, when the weights were rotated to
    // see whether anything here could fail at all.
    println!("\n{agreed} of {RAY_COUNT} rays agreed with this file in every field");
    println!("worst disagreement between the shader and this file: {worst:.6}");
}

/// The linear part of a row major 3x4, applied to a vector.
fn linear(m: [f32; 12], v: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|r| m[r * 4] * v[0] + m[r * 4 + 1] * v[1] + m[r * 4 + 2] * v[2])
}

/// The matrix a normal takes: the inverse transpose of the linear part.
fn inverse_transpose(m: [f32; 12], v: [f32; 3]) -> [f32; 3] {
    let a = [[m[0], m[1], m[2]], [m[4], m[5], m[6]], [m[8], m[9], m[10]]];

    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);

    // the cofactor matrix over the determinant is the inverse transpose,
    // with no second transpose to undo: that is what makes this the short
    // way round rather than inverting and flipping
    let cofactor = [
        [
            a[1][1] * a[2][2] - a[1][2] * a[2][1],
            a[1][2] * a[2][0] - a[1][0] * a[2][2],
            a[1][0] * a[2][1] - a[1][1] * a[2][0],
        ],
        [
            a[0][2] * a[2][1] - a[0][1] * a[2][2],
            a[0][0] * a[2][2] - a[0][2] * a[2][0],
            a[0][1] * a[2][0] - a[0][0] * a[2][1],
        ],
        [
            a[0][1] * a[1][2] - a[0][2] * a[1][1],
            a[0][2] * a[1][0] - a[0][0] * a[1][2],
            a[0][0] * a[1][1] - a[0][1] * a[1][0],
        ],
    ];

    [0, 1, 2].map(|r| (cofactor[r][0] * v[0] + cofactor[r][1] * v[1] + cofactor[r][2] * v[2]) / det)
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();

    v.map(|k| k / len)
}

fn gap(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max)
}

fn angle_between(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dot = (0..3).map(|k| a[k] * b[k]).sum::<f32>().clamp(-1.0, 1.0);

    dot.acos().to_degrees()
}
