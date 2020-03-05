mod render;

use render::State;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mesh = gen_polyhedron();
    let mut state = State::new(&window, mesh);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if state.input(event) {
                *control_flow = ControlFlow::Wait;
            } else {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => *control_flow = ControlFlow::Wait,
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                        *control_flow = ControlFlow::Wait;
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                        *control_flow = ControlFlow::Wait;
                    }
                    _ => *control_flow = ControlFlow::Wait,
                }
            }
        }
        Event::MainEventsCleared => {
            state.update();
            state.render();
            *control_flow = ControlFlow::Wait;
        }
        _ => *control_flow = ControlFlow::Wait,
    });
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
