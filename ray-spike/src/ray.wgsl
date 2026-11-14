// naga wants this before anything else, so a shader that traces is not
// the same source as one that does not.
enable wgpu_ray_query;

// The hit says which triangle and where on it. Everything else about the
// surface comes out of these, which is spec 0096's open question: whether
// the buffers that feed the acceleration structure can also be read beside
// it, at the hit, without EXPERIMENTAL_RAY_HIT_VERTEX_RETURN.
struct Material {
    colour: vec4<f32>,
    // shininess, specular, texture, spare
    rest: vec4<f32>,
}

@group(0) @binding(0) var world: acceleration_structure;
@group(0) @binding(1) var<storage, read> rays: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> out: array<vec4<f32>>;
// The same buffer the BLAS was built from, interleaved the way a renderer
// keeps one: position, normal, uv, eight floats to a vertex. Read as floats
// rather than as a struct, because an interleaved buffer has no struct.
@group(0) @binding(3) var<storage, read> vertices: array<f32>;
@group(0) @binding(4) var<storage, read> indices: array<u32>;
@group(0) @binding(5) var<storage, read> materials: array<Material>;

const STRIDE: u32 = 8u;

fn position_of(vertex: u32) -> vec3<f32> {
    let at = vertex * STRIDE;

    return vec3<f32>(vertices[at], vertices[at + 1u], vertices[at + 2u]);
}

fn normal_of(vertex: u32) -> vec3<f32> {
    let at = vertex * STRIDE + 3u;

    return vec3<f32>(vertices[at], vertices[at + 1u], vertices[at + 2u]);
}

fn uv_of(vertex: u32) -> vec2<f32> {
    let at = vertex * STRIDE + 6u;

    return vec2<f32>(vertices[at], vertices[at + 1u]);
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let n = id.x;
    let start = rays[n * 2u].xyz;
    let along = rays[n * 2u + 1u].xyz;

    var q: ray_query;
    rayQueryInitialize(&q, world, RayDesc(0u, 0xFFu, 0.001, 1000.0, start, along));
    rayQueryProceed(&q);
    let hit = rayQueryGetCommittedIntersection(&q);
    let struck = hit.kind != RAY_QUERY_INTERSECTION_NONE;

    out[n * 8u] = vec4<f32>(
        select(0.0, 1.0, struck),
        hit.t,
        f32(hit.primitive_index),
        f32(hit.instance_custom_data),
    );

    if !struck {
        return;
    }

    // Which three vertices. One geometry per structure here, so the
    // geometry's own offset into the index buffer is nought; a renderer with
    // several would add it, and `geometry_index` is reported below so that
    // the day it does, it can see what it is adding to.
    let first = hit.primitive_index * 3u;
    let a = indices[first];
    let b = indices[first + 1u];
    let c = indices[first + 2u];

    // 1 - u - v weights the first vertex, which the last spike measured.
    let w = vec3<f32>(
        1.0 - hit.barycentrics.x - hit.barycentrics.y,
        hit.barycentrics.x,
        hit.barycentrics.y,
    );

    let on_it = position_of(a) * w.x + position_of(b) * w.y + position_of(c) * w.z;
    let leaning = normal_of(a) * w.x + normal_of(b) * w.y + normal_of(c) * w.z;
    let uv = uv_of(a) * w.x + uv_of(b) * w.y + uv_of(c) * w.z;

    let o2w = hit.object_to_world;
    let w2o = hit.world_to_object;

    // **A normal is not a position and does not take the same matrix.** The
    // right one is the inverse transpose of the linear part, and a hit hands
    // over the inverse already, so this is a transpose and no inversion.
    let upright = transpose(mat3x3<f32>(w2o[0], w2o[1], w2o[2]));
    let naive = mat3x3<f32>(o2w[0], o2w[1], o2w[2]);

    out[n * 8u + 1u] = vec4<f32>(
        o2w * vec4<f32>(on_it, 1.0),
        select(0.0, 1.0, hit.front_face),
    );
    out[n * 8u + 2u] = vec4<f32>(normalize(upright * leaning), 0.0);
    out[n * 8u + 3u] = vec4<f32>(normalize(naive * leaning), 0.0);
    out[n * 8u + 4u] = vec4<f32>(uv, f32(hit.instance_index), f32(hit.geometry_index));

    // **Keyed by instance_index and not by custom_data.** The custom data is
    // whatever the builder put there, so it is an identity rather than a
    // subscript. The index is the row.
    let mine = materials[hit.instance_index];

    out[n * 8u + 5u] = mine.colour;
    out[n * 8u + 6u] = mine.rest;
    // handed over so the checking can weight the vertices itself rather
    // than trusting the three numbers above that came out of the same sum
    out[n * 8u + 7u] = vec4<f32>(hit.barycentrics, 0.0, 0.0);
}
