// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(5) model_pos: vec3<f32>,
    @location(6) model_rot: vec4<f32>, // Quaternion
    @location(7) obj_type: f32,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

fn quat_to_mat4(q: vec4<f32>) -> mat4x4<f32> {
    let x = q.x; let y = q.y; let z = q.z; let w = q.w;
    let x2 = x + x; let y2 = y + y; let z2 = z + z;
    let xx = x * x2; let xy = x * y2; let xz = x * z2;
    let yy = y * y2; let yz = y * z2; let zz = z * z2;
    let wx = w * x2; let wy = w * y2; let wz = w * z2;

    return mat4x4<f32>(
        vec4<f32>(1.0 - (yy + zz), xy + wz, xz - wy, 0.0),
        vec4<f32>(xy - wz, 1.0 - (xx + zz), yz + wx, 0.0),
        vec4<f32>(xz + wy, yz - wx, 1.0 - (xx + yy), 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

@vertex
fn vs_main(
    model: InstanceInput,
    in: VertexInput,
) -> VertexOutput {
    let rotation_matrix = quat_to_mat4(model.model_rot);
    let translation_matrix = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(model.model_pos, 1.0)
    );
    let model_matrix = translation_matrix * rotation_matrix;

    var out: VertexOutput;
    // Simple directional lighting based on normal
    // We rotate the normal by the model rotation
    let world_normal = (rotation_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 3.0));
    let diffuse = max(dot(world_normal, light_dir), 0.2);
    
    // Color based on object type
    var base_color = vec3<f32>(0.8, 0.2, 0.2); // Default Red
    if (model.model_pos.y < -1.0) { // Hack: if it's the floor (usually low y)
         base_color = vec3<f32>(0.3, 0.3, 0.3);
    } else {
         // Random-ish color based on position for variety
         let r = abs(sin(model.model_pos.x));
         let g = abs(cos(model.model_pos.y));
         let b = abs(sin(model.model_pos.z));
         base_color = vec3<f32>(r, g, b);
    }
    
    out.color = base_color * diffuse;
    out.normal = world_normal;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(in.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
