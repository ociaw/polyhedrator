#![allow(dead_code)]

mod camera;
mod mesh;
mod shader;
mod texture;
mod uniforms;
mod update;

use camera::Camera;
use camera::CameraController;
use shader::Shader;
use texture::Texture;
use uniforms::Uniforms;

pub use mesh::{Mesh, Vertex};
pub use update::Update;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn create_depth_texture(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> wgpu::Texture {
    let desc = wgpu::TextureDescriptor {
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        ..sc_desc.to_texture_desc()
    };
    device.create_texture(&desc)
}

struct Geometry {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl Geometry {
    pub fn from_mesh(mesh: &Mesh, device: &wgpu::Device) -> Geometry {
        let vertex_buffer = device
            .create_buffer_mapped(mesh.vertices().len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&mesh.vertices());

        let index_buffer = device
            .create_buffer_mapped(mesh.triangles().len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&mesh.triangles());

        Geometry {
            vertex_buffer,
            index_buffer,
            index_count: mesh.index_count(),
        }
    }
}

pub struct State {
    render_pipeline: wgpu::RenderPipeline,
    geometry: Geometry,
    texture: Texture,
    camera: Camera,
    camera_controller: CameraController,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl State {
    pub fn new(
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        swap_desc: &wgpu::SwapChainDescriptor,
        mesh: Mesh,
    ) -> Self {
        let texture_bind_group_layout = Texture::create_bind_group_layout(device);

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
            });

        let texture = Texture::load_from_file(
            "res/texture/palette.png",
            device,
            queue,
            &texture_bind_group_layout,
        )
        .unwrap();

        let vs_module = Shader::load_glsl_from_path(
            "res/shader/shader.vert",
            glsl_to_spirv::ShaderType::Vertex,
            &device,
        )
        .unwrap();
        let fs_module = Shader::load_glsl_from_path(
            "res/shader/shader.frag",
            glsl_to_spirv::ShaderType::Fragment,
            &device,
        )
        .unwrap();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            });

        let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: swap_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[Mesh::vertex_buffer_descriptor()],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };
        let render_pipeline = device.create_render_pipeline(&pipeline_descriptor);

        let geometry = Geometry::from_mesh(&mesh, &device);

        let mut camera = Camera::default(swap_desc.width as f32 / swap_desc.height as f32);
        camera.move_eye((2.0, 0.0, 0.0).into());

        let camera_controller = CameraController::new(std::f32::consts::FRAC_PI_8 / 8.0);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[uniforms]);

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..std::mem::size_of_val(&uniforms) as wgpu::BufferAddress,
                },
            }],
        });

        let depth_texture = create_depth_texture(&device, &swap_desc);
        let depth_texture_view = depth_texture.create_default_view();

        Self {
            render_pipeline,
            geometry,
            texture,
            camera,
            camera_controller,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            depth_texture,
            depth_texture_view,
        }
    }

    pub fn apply_update(&mut self, device: &wgpu::Device, update: update::Update) {
        if let Some(new_mesh) = update.mesh {
            self.geometry = Geometry::from_mesh(&new_mesh, device);
        }
        if let Some(swap_desc) = update.swap_desc {
            self.depth_texture = create_depth_texture(&device, swap_desc);
            self.depth_texture_view = self.depth_texture.create_default_view();
            self.camera.set_aspect_ratio(swap_desc.width as f32 / swap_desc.height as f32);
        }
    }

    pub fn update(&mut self, encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device) {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);

        let staging_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&[self.uniforms]);

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
        );
    }

    pub fn render<'a>(
        &self,
        attachment: &wgpu::TextureView,
        resolve_target: Option<&wgpu::TextureView>,
        encoder: &'a mut wgpu::CommandEncoder,
    ) {
        use wgpu::{LoadOp, StoreOp};

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment,
                resolve_target,
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_color: wgpu::Color::WHITE,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture_view,
                depth_load_op: LoadOp::Clear,
                depth_store_op: StoreOp::Clear,
                clear_depth: 1.0,
                stencil_load_op: LoadOp::Clear,
                stencil_store_op: StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture.bind_group(), &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        render_pass.set_vertex_buffers(0, &[(&self.geometry.vertex_buffer, 0)]);
        render_pass.set_index_buffer(&self.geometry.index_buffer, 0);
        render_pass.draw_indexed(0..self.geometry.index_count, 0, 0..1);
    }
}
