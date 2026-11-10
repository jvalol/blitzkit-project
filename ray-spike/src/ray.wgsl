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

    // Everything a reflection would need to shade the surface it hit,
    // without EXPERIMENTAL_RAY_HIT_VERTEX_RETURN: which triangle, where on
    // it, and how to get that into the world.
    out[n * 3u] = vec4<f32>(
        select(0.0, 1.0, hit.kind != RAY_QUERY_INTERSECTION_NONE),
        hit.t,
        f32(hit.primitive_index),
        f32(hit.instance_custom_data),
    );
    out[n * 3u + 1u] = vec4<f32>(
        hit.barycentrics.x,
        hit.barycentrics.y,
        select(0.0, 1.0, hit.front_face),
        f32(hit.geometry_index),
    );
    // the hit put back together from the barycentrics alone, in object
    // space, which is the whole question: 1 - u - v weights the first vertex
    let where_on_it = vec3<f32>(
        1.0 - hit.barycentrics.x - hit.barycentrics.y,
        hit.barycentrics.x,
        hit.barycentrics.y,
    );
    out[n * 3u + 2u] = vec4<f32>(where_on_it, f32(hit.instance_index));
}
