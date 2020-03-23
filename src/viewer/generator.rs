use super::render;
use crate::{Operator, Polyhedron};

pub struct Generator {
    polyhedron: Polyhedron,
}

impl Generator {
    pub fn seed(polyhedron: Polyhedron) -> Generator {
        Generator {
            polyhedron
        }
    }

    pub fn apply_operator(&mut self, operator: Operator) {
        let temp_value = crate::seeds::Platonic::Tetrahedron.polyhedron(1.0);
        let old = std::mem::replace(&mut self.polyhedron, temp_value);
        let new = old.apply(operator);
        std::mem::replace(&mut self.polyhedron, new);
    }

    pub fn apply_iter(&mut self, operators: impl IntoIterator<Item = Operator>) {
        let temp_value = crate::seeds::Platonic::Tetrahedron.polyhedron(1.0);
        let mut polyhedron = std::mem::replace(&mut self.polyhedron, temp_value);
        for op in operators.into_iter() {
            polyhedron = polyhedron.apply(op);
        }
        std::mem::replace(&mut self.polyhedron, polyhedron);
    }

    pub fn to_mesh(&self) -> render::Mesh {
        use std::iter::FromIterator;
        use render::Mesh;

        type MeshVertex = render::Vertex;

        let polyhedron = &self.polyhedron;

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
