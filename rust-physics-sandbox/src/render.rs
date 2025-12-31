use std::iter;
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use bytemuck::{Pod, Zeroable};
use crate::physics::PhysicsWorld;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct InstanceRaw {
    model_pos: [f32; 3],
    model_rot: [f32; 4],
    obj_type: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

const VERTICES: &[Vertex] = &[
    // Front face
    Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [0.5, 0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    // Back face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [0.5, 0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    // Top face
    Vertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0] },
    // Bottom face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
    Vertex { position: [0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0] },
    // Right face
    Vertex { position: [0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [0.5, 0.5, -0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [0.5, 0.5, 0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.5], normal: [1.0, 0.0, 0.0] },
    // Left face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, 0.5, 0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, 0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // Front
    4, 5, 6, 6, 7, 4, // Back
    8, 9, 10, 10, 11, 8, // Top
    12, 13, 14, 14, 15, 12, // Bottom
    16, 17, 18, 18, 19, 16, // Right
    20, 21, 22, 22, 23, 20, // Left
];

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (u32, u32),
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::Texture,
    instance_capacity: usize,
    
    // Camera state
    camera_azimuth: f32,
    camera_altitude: f32,
    camera_radius: f32,
    camera_target: [f32; 3],
}

impl Renderer {
    pub async fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let canvas = document.get_element_by_id(canvas_id).ok_or("no canvas")?;
        let canvas: HtmlCanvasElement = canvas.dyn_into().map_err(|_| "not a canvas")?;

        let width = canvas.width();
        let height = canvas.height();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // WGPU 0.19+ surface creation for canvas
        let surface_target = wgpu::SurfaceTarget::Canvas(canvas);
        let surface = instance.create_surface(surface_target).map_err(|e| e.to_string())?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or("No adapter found")?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None,
        ).await.map_err(|e| e.to_string())?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Camera Uniform
        let camera_uniform = CameraUniform {
            view_proj: [[0.0; 4]; 4], // Updated in render
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // Pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3, // position
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x3, // normal
                            },
                        ],
                    },
                    // Instance buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x3, // model_pos
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4, // model_rot
                            },
                            wgpu::VertexAttribute {
                                offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                                shader_location: 7,
                                format: wgpu::VertexFormat::Float32, // obj_type
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Initial instance buffer (empty or capacity 100)
        let instance_capacity = 100;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (instance_capacity * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("depth_texture"),
            view_formats: &[],
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size: (width, height),
            render_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            instance_capacity,
            
            camera_azimuth: 0.0,
            camera_altitude: 0.5, // radians, slightly looking down
            camera_radius: 20.0,
            camera_target: [0.0, 0.0, 0.0],
        })
    }
    
    pub fn update_camera(&mut self, dx: f32, dy: f32, zoom: f32) {
        // Sensitivity factors
        let rotate_speed = 0.005;
        let zoom_speed = 0.1;
        
        self.camera_azimuth -= dx * rotate_speed;
        self.camera_altitude = (self.camera_altitude + dy * rotate_speed).clamp(0.1, 1.5); // Prevent flipping
        self.camera_radius = (self.camera_radius + zoom * zoom_speed).clamp(2.0, 100.0);
    }

    pub fn get_ray_from_screen(&self, x: f32, y: f32) -> (nalgebra::Point3<f32>, nalgebra::Vector3<f32>) {
        let width = self.config.width as f32;
        let height = self.config.height as f32;
        
        // NDC coordinates (-1 to 1)
        let ndc_x = (2.0 * x / width) - 1.0;
        let ndc_y = 1.0 - (2.0 * y / height);
        
        let aspect = width / height;
        let proj = nalgebra::Perspective3::new(aspect, 45.0_f32.to_radians(), 0.1, 100.0);
        let target = nalgebra::Point3::new(self.camera_target[0], self.camera_target[1], self.camera_target[2]);
        
        let cx = self.camera_radius * self.camera_altitude.cos() * self.camera_azimuth.sin();
        let cy = self.camera_radius * self.camera_altitude.sin();
        let cz = self.camera_radius * self.camera_altitude.cos() * self.camera_azimuth.cos();
        
        let eye = nalgebra::Point3::new(cx, cy, cz) + target.coords;
        let view = nalgebra::Isometry3::look_at_rh(&eye, &target, &nalgebra::Vector3::y());
        
        let view_proj = proj.as_matrix() * view.to_homogeneous();
        let inv_view_proj = view_proj.try_inverse().unwrap_or_else(nalgebra::Matrix4::identity);
        
        // Near plane point (z = -1 in NDC for wgpu? No, wgpu is 0 to 1 for z. But clip space is often -1 to 1 or 0 to 1 depending on API)
        // WGPU uses 0 to 1 depth in NDC.
        // Let's use 0.0 and 1.0
        let point_ndc_near = nalgebra::Vector4::new(ndc_x, ndc_y, 0.0, 1.0);
        let point_ndc_far = nalgebra::Vector4::new(ndc_x, ndc_y, 1.0, 1.0);
        
        let point_world_near = inv_view_proj * point_ndc_near;
        let point_world_far = inv_view_proj * point_ndc_far;
        
        let near = nalgebra::Point3::from_homogeneous(point_world_near).unwrap();
        let far = nalgebra::Point3::from_homogeneous(point_world_far).unwrap();
        
        let dir = (far - near).normalize();
        
        (near, dir)
    }

    pub fn render(&mut self, physics: &PhysicsWorld) {
        // 1. Prepare Instance Data
        let render_data = physics.get_render_data(); // [x,y,z, qx,qy,qz,qw, type, ...]
        let instance_count = render_data.len() / 8;
        
        let mut instances = Vec::with_capacity(instance_count);
        for i in 0..instance_count {
            let base = i * 8;
            instances.push(InstanceRaw {
                model_pos: [render_data[base], render_data[base+1], render_data[base+2]],
                model_rot: [render_data[base+3], render_data[base+4], render_data[base+5], render_data[base+6]],
                obj_type: render_data[base+7],
            });
        }
        
        // Resize instance buffer if needed
        if instances.len() > self.instance_capacity {
             self.instance_capacity = instances.len() * 2;
             self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (self.instance_capacity * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        
        if !instances.is_empty() {
             self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }

        // 2. Update Camera (Orbital)
        let aspect = self.config.width as f32 / self.config.height as f32;
        let proj = nalgebra::Perspective3::new(aspect, 45.0_f32.to_radians(), 0.1, 100.0);
        let target = nalgebra::Point3::new(self.camera_target[0], self.camera_target[1], self.camera_target[2]);
        
        let x = self.camera_radius * self.camera_altitude.cos() * self.camera_azimuth.sin();
        let y = self.camera_radius * self.camera_altitude.sin();
        let z = self.camera_radius * self.camera_altitude.cos() * self.camera_azimuth.cos();
        
        let eye = nalgebra::Point3::new(x, y, z) + target.coords;
        let view = nalgebra::Isometry3::look_at_rh(&eye, &target, &nalgebra::Vector3::y());
        
        let view_proj = (proj.as_matrix() * view.to_homogeneous()).transpose(); // Transpose for WGSL (column-major)
        let view_proj_array: [[f32; 4]; 4] = view_proj.into();
        
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[CameraUniform { view_proj: view_proj_array }]));

        // 3. Render Pass
        let output = match self.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(_) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.1, // Deep Blue/Black Night Sky
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(0..(instances.len() * std::mem::size_of::<InstanceRaw>()) as u64));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..instances.len() as u32);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
    }
}
