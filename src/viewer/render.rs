#![allow(dead_code)]

mod camera;
mod mesh;
mod texture;
mod uniforms;

use winit::{event::*, window::Window};

use camera::Camera;
use camera::CameraController;
use texture::Texture;
use uniforms::Uniforms;

pub use mesh::{Mesh, Vertex};

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
    surface: wgpu::Surface,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
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
    pub fn _new(window: &Window, mesh: Mesh) -> Self {
        let size = window.inner_size();
        let surface = wgpu::Surface::create(window);
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            ..Default::default()
        })
        .unwrap();

        let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: Default::default(),
        });

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Vsync,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler,
                    },
                ],
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
            });

        let texture_bytes = include_bytes!("res/texture/palette.png");
        let texture = Texture::from_buffer(
            &device,
            &mut queue,
            &texture_bind_group_layout,
            texture_bytes,
        );

        let vs_src = include_str!("res/shader/shader.vert");
        let fs_src = include_str!("res/shader/shader.frag");

        let vs_spirv = glsl_to_spirv::compile(vs_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
        let fs_spirv = glsl_to_spirv::compile(fs_src, glsl_to_spirv::ShaderType::Fragment).unwrap();

        let vs_data = wgpu::read_spirv(vs_spirv).unwrap();
        let fs_data = wgpu::read_spirv(fs_spirv).unwrap();

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

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
                format: sc_desc.format,
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

        let mut camera = Camera::default(sc_desc.width as f32 / sc_desc.height as f32);
        camera.move_eye((2.0, 0.0, 0.0).into());

        let camera_controller = CameraController::new(std::f32::consts::FRAC_PI_8 / 8.0);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device
            // The COPY_DST part will be important later
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[uniforms]);

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    // FYI: you can share a single buffer between bindings.
                    range: 0..std::mem::size_of_val(&uniforms) as wgpu::BufferAddress,
                },
            }],
        });

        let depth_texture = create_depth_texture(&device, &sc_desc);
        let depth_texture_view = depth_texture.create_default_view();

        Self {
            surface,
            _adapter: adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
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

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        self.depth_texture = create_depth_texture(&self.device, &self.sc_desc);
        self.depth_texture_view = self.depth_texture.create_default_view();
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);

        // Copy operation's are performed on the gpu, so we'll need
        // a CommandEncoder for that
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let staging_buffer = self
            .device
            .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&[self.uniforms]);

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
        );

        // We need to remember to submit our CommandEncoder's output
        // otherwise we won't see any change.
        self.queue.submit(&[encoder.finish()]);
    }

    pub fn _render(&mut self) {
        let frame = self.swap_chain.get_next_texture();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            use wgpu::{LoadOp, StoreOp};

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    },
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

        self.queue.submit(&[encoder.finish()]);
    }
}
