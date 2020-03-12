#![allow(dead_code)]

mod controls;
mod render;

use render::State;
use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use iced_wgpu::{window::SwapChain, Primitive, Renderer, Settings, Target};
use iced_winit::{Cache, Clipboard, MouseCursor, Size, UserInterface};

const MSAA_SAMPLES: u32 = 4;
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut size = window.inner_size();
    let mut logical_size = size.to_logical(window.scale_factor());
    let mut modifiers = event::ModifiersState::default();
    let surface = wgpu::Surface::create(&window);
    let adapter = wgpu::Adapter::request(&Default::default()).unwrap();
    let (mut device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: Default::default(),
    });

    let mut ui_swap = SwapChain::new(&device, &surface, TEXTURE_FORMAT, size.width, size.height);
    let mut ui_swap_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: TEXTURE_FORMAT,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    };

    let mut resized = false;

    // Initialize iced
    let mut events = Vec::new();
    let mut cache = Some(Cache::default());
    let mut renderer = Renderer::new(&mut device, Settings::default());
    let mut output = (Primitive::None, MouseCursor::OutOfBounds);
    let clipboard = Clipboard::new(&window);

    let mut controls = controls::Controls::new();

    // let mut ui_framebuffer = create_multisampled_framebuffer(&device, &ui_swap_desc, MSAA_SAMPLES);

    let mesh = gen_polyhedron();
    let mut state = State::new(&device, &mut queue, &ui_swap_desc, mesh);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Wait
        };
        match event {
            event::Event::WindowEvent { event, .. } => {
                match event {
                    event::WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    // Recreate swapchain when window is resized.
                    event::WindowEvent::Resized(new_size) => {
                        if new_size.width == 0 || new_size.height == 0 {
                            return;
                        }
                        size = new_size;
                        logical_size = size.to_logical(window.scale_factor());
                        resized = true;
                        /*
                        ui_framebuffer =
                            create_multisampled_framebuffer(&device, &ui_swap_desc, MSAA_SAMPLES);
                            state.resize(&device, &ui_swap_desc);
                            */
                    }

                    // Close on request or on Escape.
                    event::WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(event::VirtualKeyCode::Escape),
                                state: event::ElementState::Pressed,
                                ..
                            },
                        ..
                    }
                    | event::WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) =
                    iced_winit::conversion::window_event(event, window.scale_factor(), modifiers)
                {
                    events.push(event);
                }
            }

            event::Event::MainEventsCleared => {
                // If no relevant events happened, we can simply skip this
                if events.is_empty() {
                    return;
                }

                // We need to:
                // 1. Process events of our user interface.
                // 2. Update state as a result of any interaction.
                // 3. Generate a new output for our renderer.

                // First, we build our user interface.
                let mut user_interface = UserInterface::build(
                    controls.view(&state),
                    Size::new(logical_size.width, logical_size.height),
                    cache.take().unwrap(),
                    &mut renderer,
                );

                // Then, we process the events, obtaining messages in return.
                let messages = user_interface.update(
                    events.drain(..),
                    clipboard.as_ref().map(|c| c as _),
                    &renderer,
                );

                let user_interface = if messages.is_empty() {
                    // If there are no messages, no interactions we care about have
                    // happened. We can simply leave our user interface as it is.
                    user_interface
                } else {
                    // If there are messages, we need to update our state
                    // accordingly and rebuild our user interface.
                    // We can only do this if we drop our user interface first
                    // by turning it into its cache.
                    cache = Some(user_interface.into_cache());

                    // In this example, `Controls` is the only part that cares
                    // about messages, so updating our state is pretty
                    // straightforward.
                    for message in messages {
                        controls.update(message, &mut state);
                    }

                    // Once the state has been changed, we rebuild our updated
                    // user interface.
                    UserInterface::build(
                        controls.view(&state),
                        Size::new(logical_size.width, logical_size.height),
                        cache.take().unwrap(),
                        &mut renderer,
                    )
                };

                // Finally, we just need to draw a new output for our renderer,
                output = user_interface.draw(&mut renderer);

                // update our cache,
                cache = Some(user_interface.into_cache());

                // and request a redraw
                window.request_redraw();
            }
            event::Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();

                    ui_swap =
                        SwapChain::new(&device, &surface, TEXTURE_FORMAT, size.width, size.height);
                    ui_swap_desc = wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        format: TEXTURE_FORMAT,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::Vsync,
                    };
                    state.resize(&device, &ui_swap_desc);
                    resized = false;
                }

                let (frame, viewport) = ui_swap.next_frame();

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

                state.update(&mut encoder, &device);
                state.render(&frame.view, None, &mut encoder);

                // And then iced on top
                let mouse_cursor = renderer.draw(
                    &mut device,
                    &mut encoder,
                    Target {
                        texture: &frame.view,
                        viewport,
                    },
                    &output,
                    window.scale_factor(),
                    &["Some debug information!"],
                );

                queue.submit(&[encoder.finish()]);

                // And update the mouse cursor
                window.set_cursor_icon(iced_winit::conversion::mouse_cursor(mouse_cursor));
            }
            _ => (),
        }
    });
}

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: sc_desc.width,
        height: sc_desc.height,
        depth: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: sc_desc.format,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    device
        .create_texture(multisampled_frame_descriptor)
        .create_default_view()
}

fn gen_polyhedron() -> render::Mesh {
    use super::operators::Kis;
    use super::seeds::Platonic;
    use super::Operator;
    use render::Mesh;
    use std::iter::FromIterator;

    type MeshVertex = render::Vertex;

    let seed = Platonic::dodecahedron(2.0);
    let kis = Kis::scale_apex(0.0);
    let operations = vec![
        Operator::Kis(kis),
        Operator::Dual,
        Operator::Kis(kis),
        Operator::Dual,
        Operator::Kis(kis),
        Operator::Dual,
        Operator::Kis(kis),
        Operator::Dual,
    ];
    let start = std::time::SystemTime::now();
    let polyhedron = seed.apply_iter(operations);
    let end = std::time::SystemTime::now();
    eprintln!(
        "Polyhedron generation took {} ms.",
        end.duration_since(start).unwrap().as_millis()
    );

    let faces = polyhedron.faces();
    let classes = polyhedron.classify_faces();

    let mesh = Mesh::from_vertex_groups(faces.iter().enumerate().map(
        |(i, face)| -> Vec<MeshVertex> {
            let class = classes[i];
            let coord_x = ((class % 8) as f32 + 0.5) / 8.0;
            let coord_y = ((class / 8) as f32 + 0.5) as f32 / 8.0;

            let vertices = polyhedron.face_vertices(face);
            let normal = normal(vertices.clone()).cast::<f32>().unwrap();

            Vec::from_iter(vertices.map(|vertex| -> MeshVertex {
                MeshVertex::new(vertex.cast::<f32>().unwrap(), [coord_x, coord_y], normal)
            }))
        },
    ));

    eprintln!(
        "faces: {}, triangles: {}, verts: {}",
        faces.len(),
        mesh.triangles().len(),
        mesh.vertices().len()
    );
    mesh
}

fn normal(mut vertices: impl Iterator<Item = polyhedrator::Vertex>) -> cgmath::Vector3<f64> {
    use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};

    // Using a vertex near the polygon reduces error for polygons far from the origin
    let origin = Point3::origin();
    let first = vertices.next().unwrap_or(origin);
    let normalizer = first;

    let mut normal = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut previous = first - normalizer;
    for vertex in vertices {
        let current = vertex - normalizer;
        normal += previous.cross(current);
        previous = current;
    }
    normal += previous.cross(first - normalizer);

    normal.normalize()
}
