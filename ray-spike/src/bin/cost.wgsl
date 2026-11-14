enable wgpu_ray_query;

@group(0) @binding(0) var world: acceleration_structure;
@group(0) @binding(1) var<storage, read_write> out: array<f32>;

// A ray a pixel, from a camera outside the scene looking into it. The point
// is the tracing rather than the picture: what comes out is how far it went,
// written so the dispatch cannot be optimised away.
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let wide = 1920.0;
    let tall = 1080.0;
    let at = vec2<f32>(f32(id.x) / wide - 0.5, f32(id.y) / tall - 0.5);

    // inside the room, looking across it, which is where a reflection is
    let eye = vec3<f32>(0.0, 5.0, 18.0);
    let along = normalize(vec3<f32>(at.x * 1.8, at.y * 1.0, -1.0));

    var q: ray_query;
    rayQueryInitialize(&q, world, RayDesc(0u, 0xFFu, 0.01, 200.0, eye, along));
    rayQueryProceed(&q);
    let hit = rayQueryGetCommittedIntersection(&q);

    out[id.y * u32(wide) + id.x] = select(0.0, hit.t, hit.kind != RAY_QUERY_INTERSECTION_NONE);
}
