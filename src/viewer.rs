#![allow(dead_code)]

mod render;

use render::State;
use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use conrod_core::widget_ids;

// Generate the winit <-> conrod_core type conversion fns.
conrod_winit::v021_conversion_fns!();

const UI_MSAA_SAMPLES: u32 = 4;
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut size = window.inner_size();
    let surface = wgpu::Surface::create(&window);
    let adapter = wgpu::Adapter::request(&Default::default()).unwrap();
    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: Default::default(),
    });
    let mut ui_swap_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: TEXTURE_FORMAT,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    };
    let mut ui_swap = device.create_swap_chain(&surface, &ui_swap_desc);
    let mut ui_renderer = conrod_wgpu::Renderer::new(&device, UI_MSAA_SAMPLES, TEXTURE_FORMAT);
    let mut ui_framebuffer = create_multisampled_framebuffer(&device, &ui_swap_desc, UI_MSAA_SAMPLES);
    let mut ui = conrod_core::UiBuilder::new([size.width as f64, size.height as f64])
        .theme(theme())
        .build();

    let id_generator = ui.widget_id_generator();
    let ids = Ids::new(id_generator);

    let bytes = &include_bytes!("viewer/res/NotoSans-Regular.ttf")[..];
    let font = conrod_core::text::Font::from_bytes(bytes);
    ui.fonts.insert(font.unwrap());

    let image_map = conrod_core::image::Map::new();

    let mesh = gen_polyhedron();
    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = convert_event(&event, &window) {
            ui.handle_event(event);
        }

        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Wait
        };
        match event {
            event::Event::WindowEvent { event, .. } => match event {
                // Recreate swapchain when window is resized.
                event::WindowEvent::Resized(new_size) => {
                    size = new_size;
                    ui_swap_desc.width = new_size.width;
                    ui_swap_desc.height = new_size.height;
                    ui_swap = device.create_swap_chain(&surface, &ui_swap_desc);
                    ui_framebuffer =
                        create_multisampled_framebuffer(&device, &ui_swap_desc, UI_MSAA_SAMPLES);
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
            },

            event::Event::MainEventsCleared => {
                // Update widgets if any event has happened
                if ui.global_input().events().next().is_some() {
                    let mut ui = ui.set_widgets();
                    gui(&mut ui, &ids);
                    window.request_redraw();
                }
            }

            event::Event::RedrawRequested(_) => {
                // If the view has changed at all, it's time to draw.
                let primitives = match ui.draw_if_changed() {
                    None => return,
                    Some(ps) => ps,
                };

                // The window frame that we will draw to.
                let frame = ui_swap.get_next_texture();

                // Begin encoding commands.
                let cmd_encoder_desc = wgpu::CommandEncoderDescriptor { todo: 0 };
                let mut encoder = device.create_command_encoder(&cmd_encoder_desc);

                // Feed the renderer primitives and update glyph cache texture if necessary.
                let scale_factor = window.scale_factor();
                let [win_w, win_h]: [f32; 2] = [size.width as f32, size.height as f32];
                let viewport = [0.0, 0.0, win_w, win_h];
                if let Some(cmd) = ui_renderer
                    .fill(&image_map, viewport, scale_factor, primitives)
                    .unwrap()
                {
                    cmd.load_buffer_and_encode(&device, &mut encoder);
                }

                // Begin the render pass and add the draw commands.
                {
                    // This condition allows to more easily tweak the UI_MSAA_SAMPLES constant.
                    let (attachment, resolve_target) = match UI_MSAA_SAMPLES {
                        1 => (&frame.view, None),
                        _ => (&ui_framebuffer, Some(&frame.view)),
                    };
                    let color_attachment_desc = wgpu::RenderPassColorAttachmentDescriptor {
                        attachment,
                        resolve_target,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::BLACK,
                    };

                    let render_pass_desc = wgpu::RenderPassDescriptor {
                        color_attachments: &[color_attachment_desc],
                        depth_stencil_attachment: None,
                    };
                    let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

                    let render = ui_renderer.render(&device, &image_map);
                    render_pass.set_pipeline(render.pipeline);
                    render_pass.set_vertex_buffers(0, &[(&render.vertex_buffer, 0)]);
                    let instance_range = 0..1;
                    for cmd in render.commands {
                        match cmd {
                            conrod_wgpu::RenderPassCommand::SetBindGroup { bind_group } => {
                                render_pass.set_bind_group(0, bind_group, &[]);
                            }
                            conrod_wgpu::RenderPassCommand::SetScissor {
                                top_left,
                                dimensions,
                            } => {
                                let [x, y] = top_left;
                                let [w, h] = dimensions;
                                render_pass.set_scissor_rect(x, y, w, h);
                            }
                            conrod_wgpu::RenderPassCommand::Draw { vertex_range } => {
                                render_pass.draw(vertex_range, instance_range.clone());
                            }
                        }
                    }
                }

                queue.submit(&[encoder.finish()]);
            }
            _ => (),
        }
    });
}

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    pub struct Ids {
        // The scrollable canvas.
        canvas,
        // The title and introduction widgets.
        title,
        introduction,
        // Shapes.
        shapes_canvas,
        rounded_rectangle,
        shapes_left_col,
        shapes_right_col,
        shapes_title,
        line,
        point_path,
        rectangle_fill,
        rectangle_outline,
        trapezoid,
        oval_fill,
        oval_outline,
        circle,
        // Image.
        image_title,
        rust_logo,
        // Button, XyPad, Toggle.
        button_title,
        button,
        xy_pad,
        toggle,
        ball,
        // NumberDialer, PlotPath
        dialer_title,
        number_dialer,
        plot_path,
        // Scrollbar
        canvas_scrollbar,
    }
}

fn theme() -> conrod_core::Theme {
    use conrod_core::position::{Align, Direction, Padding, Position, Relative};
    conrod_core::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod_core::color::TRANSPARENT,
        shape_color: conrod_core::color::LIGHT_CHARCOAL,
        border_color: conrod_core::color::BLACK,
        border_width: 0.0,
        label_color: conrod_core::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod_core::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}

fn gui(ui: &mut conrod_core::UiCell, ids: &Ids) {
    use conrod_core::{widget, Positionable, Sizeable, Widget};

    const MARGIN: conrod_core::Scalar = 30.0;
    const SHAPE_GAP: conrod_core::Scalar = 50.0;
    const TITLE_SIZE: conrod_core::FontSize = 42;
    const SUBTITLE_SIZE: conrod_core::FontSize = 32;

    // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
    // By default, its size is the size of the window. We'll use this as a background for the
    // following widgets, as well as a scrollable container for the children widgets.
    const TITLE: &'static str = "All Widgets";
    widget::Canvas::new()
        .pad(MARGIN)
        .set(ids.canvas, ui);

    ////////////////
    ///// TEXT /////
    ////////////////

    // We'll demonstrate the `Text` primitive widget by using it to draw a title and an
    // introduction to the example.
    widget::Text::new(TITLE)
        .font_size(TITLE_SIZE)
        .mid_top_of(ids.canvas)
        .set(ids.title, ui);

    const INTRODUCTION: &'static str =
        "This example aims to demonstrate all widgets that are provided by conrod.\
         \n\nThe widget that you are currently looking at is the Text widget. The Text widget \
         is one of several special \"primitive\" widget types which are used to construct \
         all other widget types. These types are \"special\" in the sense that conrod knows \
         how to render them via `conrod_core::render::Primitive`s.\
         \n\nScroll down to see more widgets!";
    widget::Text::new(INTRODUCTION)
        .padded_w_of(ids.canvas, MARGIN)
        .down(60.0)
        .align_middle_x_of(ids.canvas)
        .center_justify()
        .line_spacing(5.0)
        .set(ids.introduction, ui);
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
