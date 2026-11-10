// naga wants this before anything else, so a shader that traces is not
// the same source as one that does not.
enable wgpu_ray_query;

@group(0) @binding(0) var world: acceleration_structure;
@group(0) @binding(1) var<storage, read> rays: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> out: array<vec4<f32>>;

// Each ray starts above the triangle and points straight down, except the
// ones aimed to miss. Writes hit, t, kind, instance.
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let n = id.x;
    let start = rays[n * 2u].xyz;
    let along = rays[n * 2u + 1u].xyz;

    var q: ray_query;
    rayQueryInitialize(&q, world, RayDesc(0u, 0xFFu, 0.001, 1000.0, start, along));
    rayQueryProceed(&q);
    let hit = rayQueryGetCommittedIntersection(&q);

    out[n] = vec4<f32>(
        select(0.0, 1.0, hit.kind != RAY_QUERY_INTERSECTION_NONE),
        hit.t,
        f32(hit.kind),
        f32(hit.instance_custom_data),
    );
}
