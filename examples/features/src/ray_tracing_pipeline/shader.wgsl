enable wgpu_ray_tracing_pipeline;

struct HitPayload {
    color: vec3<f32>,
}

@group(0) @binding(0)
var acc_struct: acceleration_structure;

@group(0) @binding(1)
var output: texture_storage_2d<rgba8unorm, write>;

var<ray_payload> payload: HitPayload;

@ray_generation
fn raygen(@builtin(ray_invocation_id) id: vec3<u32>, @builtin(num_ray_invocations) dims: vec3<u32>) {
    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dims.xy);
    let d = uv * 2.0 - 1.0;

    let origin = vec3<f32>(0.0, 0.0, 2.5);
    let direction = normalize(vec3<f32>(d.x, -d.y, -1.0));

    payload = HitPayload(vec3<f32>(0.0));

    traceRay(acc_struct, RayDesc(0u, 0xFFu, 0.001, 100.0, origin, direction), &payload);

    textureStore(output, id.xy, vec4<f32>(payload.color, 1.0));
}

var<incoming_ray_payload> incoming: HitPayload;

@miss
@incoming_payload(incoming)
fn miss_main() {
    // Sky gradient
    incoming.color = vec3<f32>(0.2, 0.3, 0.5);
}

@closest_hit
@incoming_payload(incoming)
fn closest_hit_main(
    @builtin(object_ray_origin) origin: vec3<f32>,
    @builtin(object_ray_direction) dir: vec3<f32>,
) {
    // Simple coloring based on ray direction
    incoming.color = normalize(abs(dir));
}
