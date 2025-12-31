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
    @location(2) world_pos: vec3<f32>,
    @location(3) obj_type: f32,
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
    let world_pos = (model_matrix * vec4<f32>(in.position, 1.0)).xyz;

    var out: VertexOutput;
    // We rotate the normal by the model rotation
    let world_normal = (rotation_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    
    // Color based on object type
    var base_color = vec3<f32>(0.8, 0.2, 0.2); // Default Red
    if (model.obj_type > 1.9 && model.obj_type < 2.1) { 
         // Type 2: Liquid (Blue)
         base_color = vec3<f32>(0.2, 0.5, 0.9);
    } else if (model.obj_type > 2.9 && model.obj_type < 3.1) {
         // Type 3: Soft Body (Purple)
         base_color = vec3<f32>(0.7, 0.2, 0.8);
    } else if (model.obj_type > 3.9 && model.obj_type < 4.1) { 
         // Type 4: Floor (Dark Grey base, grid handled in fragment)
         base_color = vec3<f32>(0.1, 0.1, 0.12);
    } else {
         // Random-ish color based on position for variety
         let r = abs(sin(model.model_pos.x));
         let g = abs(cos(model.model_pos.y));
         let b = abs(sin(model.model_pos.z));
         base_color = vec3<f32>(r, g, b);
    }
    
    out.color = base_color;
    out.normal = world_normal;
    out.world_pos = world_pos;
    out.obj_type = model.obj_type;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 3.0));
    let view_dir = normalize(vec3<f32>(0.0, 10.0, 20.0) - in.world_pos); // Approx view dir
    
    // Grid Floor Logic
    if (in.obj_type > 3.9 && in.obj_type < 4.1) {
        let grid_size = 1.0;
        let line_width = 0.05;
        let x = abs(in.world_pos.x % grid_size);
        let z = abs(in.world_pos.z % grid_size);
        
        // Draw grid lines
        if (x < line_width || z < line_width) {
            return vec4<f32>(0.3, 0.8, 0.8, 1.0); // Neon cyan grid
        }
        
        // Check for axis lines
        if (abs(in.world_pos.x) < line_width * 2.0) {
             return vec4<f32>(0.2, 0.2, 0.8, 1.0); // Z axis (Blue)
        }
        if (abs(in.world_pos.z) < line_width * 2.0) {
             return vec4<f32>(0.8, 0.2, 0.2, 1.0); // X axis (Red)
        }
        
        return vec4<f32>(in.color, 1.0);
    }

    // Standard Lighting
    let diffuse = max(dot(in.normal, light_dir), 0.2);
    
    // Specular / Rim
    let reflect_dir = reflect(-light_dir, in.normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let rim = 1.0 - max(dot(view_dir, in.normal), 0.0);
    let rim_color = vec3<f32>(0.2, 0.2, 0.3) * pow(rim, 3.0);
    
    let final_color = in.color * diffuse + vec3<f32>(spec * 0.3) + rim_color;
    
    return vec4<f32>(final_color, 1.0);
}
