mod builder;
mod keys;
pub mod operators;

pub use operators::Operator;
pub type Vertex = Point3<f64>;

use builder::Builder;
use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};
use fnv::FnvHashMap;
use keys::{FaceKey, VertexKey};

#[derive(Clone, Debug)]
pub struct Face {
    indices: Vec<u32>,
}

impl Face {
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn last(&self) -> (u32, u32) {
        assert!(self.indices.len() > 1);
        let last_index = self.indices.len() - 1;
        (self.indices[last_index - 1], self.indices[last_index])
    }
}

#[derive(Clone, Debug)]
pub struct Polyhedron {
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
}

impl Polyhedron {
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn classify_faces(&self) -> Vec<usize> {
        let mut face_classes = Vec::new();
        let mut classes = FnvHashMap::default();

        for face in &self.faces {
            let sig = signature(self.face_vertices(face));
            let new_index = classes.len();
            let class = classes.entry(sig).or_insert(new_index);
            face_classes.push(*class);
        }

        eprintln!("Face class count: {}", classes.len());
        face_classes
    }

    pub fn face_vertices<'a>(
        &'a self,
        face: &'a Face,
    ) -> impl ExactSizeIterator<Item = Vertex> + 'a + Clone {
        face.indices
            .iter()
            .map(move |index| -> Vertex { self.vertices[*index as usize] })
    }

    /// Applies the operator and returns the resulting polyhedron.
    pub fn apply(self, operator: Operator) -> Polyhedron {
        use operators::*;

        match operator {
            Operator::Ambo => self.ambo(),
            Operator::Dual => self.dual(),
            Operator::Kis(kis) => self.kis(kis),
        }
    }

    /// Applies each operator in order and returns the resulting polyhedron.
    pub fn apply_iter(self, operators: impl IntoIterator<Item = Operator>) -> Polyhedron {
        let mut polyhedron = self;
        for op in operators.into_iter() {
            polyhedron = polyhedron.apply(op);
        }
        polyhedron
    }

    /// Applies the `kis` operator and returns the resulting polyhedron.
    pub fn kis(self, kis: operators::Kis) -> Polyhedron {
        let mut builder = Builder::new();

        for i in 0..self.vertices.len() {
            let vertex = self.vertices[i];
            builder.add_vertex(VertexKey::Seed(i as u32), vertex);
        }

        for face_index in 0..self.faces.len() {
            let face = &self.faces[face_index];
            let face_index = face_index as u32;
            let is_identity =
                kis.side_count() != 0 && kis.side_count() as usize != face.indices.len();
            let mut v1_key = VertexKey::Seed(*face.indices.last().unwrap());

            let apex_position = {
                let vertices = self.face_vertices(face);
                let center = center(vertices.clone());
                let normal = normal(vertices.clone());
                let dist_to_center = mean_distance(vertices.clone(), center);
                center + normal * (kis.apex_scale() * dist_to_center)
            };

            for index in &face.indices {
                let v2_key = VertexKey::Seed(*index);
                if is_identity {
                    builder.add_flag(FaceKey::Seed(face_index), v1_key, v2_key);
                    v1_key = v2_key;
                    continue;
                }

                let apex_key = VertexKey::Centroid(face_index);
                let face_key = FaceKey::PyramidFace(face_index, v1_key);

                builder.add_vertex(apex_key, apex_position);
                builder.add_flag(face_key, v1_key, v2_key);
                builder.add_flag(face_key, v2_key, apex_key);
                builder.add_flag(face_key, apex_key, v1_key);
                v1_key = v2_key;
            }
        }

        builder.build_polyhedron()
    }

    /// Applies the `dual` operator and returns the resulting polyhedron.
    pub fn dual(self) -> Polyhedron {
        let mut builder = Builder::new();

        let mut face_map = Vec::with_capacity(self.vertices.len());
        face_map.resize(face_map.capacity(), FnvHashMap::default());
        for i in 0..self.faces.len() {
            let center = center(self.face_vertices(&self.faces[i]));
            builder.add_vertex(VertexKey::Centroid(i as u32), center);
        }

        for i in 0..self.faces.len() {
            let face = &self.faces[i];
            let mut v1 = face.last().1;

            let face_vertex_key = VertexKey::Centroid(i as u32);
            for v2 in &face.indices {
                let map = &mut face_map[v1 as usize];
                map.insert(*v2, face_vertex_key);
                v1 = *v2;
            }
        }

        for i in 0..self.faces.len() {
            let face = &self.faces[i];
            let mut v1 = face.last().1;

            for v2 in &face.indices {
                let map = &mut face_map[*v2 as usize];
                let vertex = map.get(&v1).expect("Should be present");
                builder.add_flag(FaceKey::Vertex(v1), *vertex, VertexKey::Centroid(i as u32));
                v1 = *v2;
            }
        }

        builder.build_polyhedron()
    }

    /// Applies the `ambo` operator and returns the resulting polyhedron.
    pub fn ambo(self) -> Polyhedron {
        let mut builder = Builder::new();
        for i in 0..self.faces.len() {
            let face = &self.faces[i];
            let (mut v1, mut v2) = face.last();

            for v3 in &face.indices {
                if v1 < v2 {
                    let midpoint = self.vertices[v1 as usize].midpoint(self.vertices[v2 as usize]);
                    builder.add_vertex(VertexKey::midpoint(v1, v2), midpoint);
                }

                builder.add_flag(
                    FaceKey::Seed(i as u32),
                    VertexKey::midpoint(v1, v2),
                    VertexKey::midpoint(v2, *v3),
                );
                builder.add_flag(
                    FaceKey::Vertex(v2),
                    VertexKey::midpoint(v2, *v3),
                    VertexKey::midpoint(v1, v2),
                );
                v1 = v2;
                v2 = *v3;
            }
        }
        builder.build_polyhedron()
    }

    pub fn cube(edge_length: f64) -> Polyhedron {
        let scalar = edge_length / 2.0;

        Polyhedron {
            vertices: vec![
                [scalar, scalar, scalar].into(),
                [scalar, scalar, -scalar].into(),
                [scalar, -scalar, scalar].into(),
                [scalar, -scalar, -scalar].into(),
                [-scalar, scalar, scalar].into(),
                [-scalar, scalar, -scalar].into(),
                [-scalar, -scalar, scalar].into(),
                [-scalar, -scalar, -scalar].into(),
            ],
            faces: vec![
                Face {
                    indices: vec![0, 2, 3, 1],
                },
                Face {
                    indices: vec![4, 5, 7, 6],
                },
                Face {
                    indices: vec![0, 1, 5, 4],
                },
                Face {
                    indices: vec![2, 6, 7, 3],
                },
                Face {
                    indices: vec![0, 4, 6, 2],
                },
                Face {
                    indices: vec![1, 3, 7, 5],
                },
            ],
        }
    }

    pub fn regular_dodecahedron(edge_length: f64) -> Polyhedron {
        // Thank goodness for CAS
        let scalar = (edge_length * (5.0f64.sqrt() + 1.0)) / 4.0;
        let phi = (edge_length * (5.0f64.sqrt() + 3.0)) / 4.0;
        let inverse = edge_length / 2.0;

        Polyhedron {
            vertices: vec![
                [scalar, scalar, scalar].into(),
                [scalar, scalar, -scalar].into(),
                [scalar, -scalar, scalar].into(),
                [scalar, -scalar, -scalar].into(),
                [-scalar, scalar, scalar].into(),
                [-scalar, scalar, -scalar].into(),
                [-scalar, -scalar, scalar].into(),
                [-scalar, -scalar, -scalar].into(),
                [0.0, phi, inverse].into(),
                [0.0, phi, -inverse].into(),
                [0.0, -phi, inverse].into(),
                [0.0, -phi, -inverse].into(),
                [phi, inverse, 0.0].into(),
                [phi, -inverse, 0.0].into(),
                [-phi, inverse, 0.0].into(),
                [-phi, -inverse, 0.0].into(),
                [inverse, 0.0, phi].into(),
                [-inverse, 0.0, phi].into(),
                [inverse, 0.0, -phi].into(),
                [-inverse, 0.0, -phi].into(),
            ],
            faces: vec![
                Face {
                    indices: vec![0, 8, 4, 17, 16],
                },
                Face {
                    indices: vec![0, 12, 1, 9, 8],
                },
                Face {
                    indices: vec![0, 16, 2, 13, 12],
                },
                Face {
                    indices: vec![1, 12, 13, 3, 18],
                },
                Face {
                    indices: vec![1, 18, 19, 5, 9],
                },
                Face {
                    indices: vec![2, 10, 11, 3, 13],
                },
                Face {
                    indices: vec![3, 11, 7, 19, 18],
                },
                Face {
                    indices: vec![4, 8, 9, 5, 14],
                },
                Face {
                    indices: vec![4, 14, 15, 6, 17],
                },
                Face {
                    indices: vec![5, 19, 7, 15, 14],
                },
                Face {
                    indices: vec![6, 10, 2, 16, 17],
                },
                Face {
                    indices: vec![6, 15, 7, 11, 10],
                },
            ],
        }
    }

    pub fn regular_icosahedron(edge_length: f64) -> Polyhedron {
        let scalar = edge_length / 2.0;
        let phi = Self::golden_ratio() * scalar;

        Polyhedron {
            vertices: vec![
                [-scalar, phi, 0.0].into(),
                [scalar, phi, 0.0].into(),
                [-scalar, -phi, 0.0].into(),
                [scalar, -phi, 0.0].into(),
                [0.0, -scalar, phi].into(),
                [0.0, scalar, phi].into(),
                [0.0, -scalar, -phi].into(),
                [0.0, scalar, -phi].into(),
                [phi, 0.0, -scalar].into(),
                [phi, 0.0, scalar].into(),
                [-phi, 0.0, -scalar].into(),
                [-phi, 0.0, scalar].into(),
            ],
            faces: vec![
                Face {
                    indices: vec![0, 1, 7],
                },
                Face {
                    indices: vec![0, 5, 1],
                },
                Face {
                    indices: vec![0, 7, 10],
                },
                Face {
                    indices: vec![0, 10, 11],
                },
                Face {
                    indices: vec![0, 11, 5],
                },
                Face {
                    indices: vec![1, 5, 9],
                },
                Face {
                    indices: vec![1, 8, 7],
                },
                Face {
                    indices: vec![1, 9, 8],
                },
                Face {
                    indices: vec![2, 3, 4],
                },
                Face {
                    indices: vec![2, 4, 11],
                },
                Face {
                    indices: vec![2, 6, 3],
                },
                Face {
                    indices: vec![2, 10, 6],
                },
                Face {
                    indices: vec![2, 11, 10],
                },
                Face {
                    indices: vec![3, 6, 8],
                },
                Face {
                    indices: vec![3, 8, 9],
                },
                Face {
                    indices: vec![3, 9, 4],
                },
                Face {
                    indices: vec![4, 5, 11],
                },
                Face {
                    indices: vec![4, 9, 5],
                },
                Face {
                    indices: vec![6, 7, 8],
                },
                Face {
                    indices: vec![6, 10, 7],
                },
            ],
        }
    }

    fn golden_ratio() -> f64 {
        (1.0 + 5.0f64.sqrt()) / 2.0
    }
}

fn normal(mut vertices: impl Iterator<Item = Vertex>) -> Vector3<f64> {
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

fn center(vertices: impl Iterator<Item = Vertex>) -> Point3<f64> {
    let mut center = Point3::<f64>::origin();
    let mut count = 0u32;
    for vertex in vertices {
        center += vertex.to_vec();
        count += 1;
    }
    center / count as f64
}

fn mean_distance(vertices: impl Iterator<Item = Vertex>, point: Point3<f64>) -> f64 {
    use cgmath::MetricSpace;

    let mut distance = 0.0;
    let mut count = 0;
    for vertex in vertices {
        distance += point.distance(vertex);
        count += 1;
    }
    distance / count as f64
}

fn signature(mut vertices: impl ExactSizeIterator<Item = Vertex>) -> Vec<u64> {
    if vertices.len() < 3 {
        return Vec::new();
    }

    let mut crosses = Vec::with_capacity(vertices.len());
    let first = vertices.next().unwrap();
    let second = vertices.next().unwrap();
    let mut v1 = first;
    let mut v2 = second;
    for v3 in vertices {
        crosses.push((v1 - v2).cross(v3 - v2).magnitude());
        v1 = v2;
        v2 = v3;
    }
    crosses.push((v1 - v2).cross(first - v2).magnitude());
    crosses.push((v2 - first).cross(first - second).magnitude());

    crosses.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    crosses
        .into_iter()
        .map(|f| -> u64 { truncate_mantissa(f, 7) })
        .collect()
}

fn truncate_mantissa(f: f64, bits: u32) -> u64 {
    use cgmath::num_traits::Float;
    const ZERO_COUNT: u32 = 0u64.leading_zeros();

    if bits >= ZERO_COUNT {
        panic!("bits must be less than {}.", ZERO_COUNT);
    }

    let (mantissa, _, _) = f.integer_decode();
    let original_bits = ZERO_COUNT - mantissa.leading_zeros();
    if bits >= original_bits {
        // The mantissa has the same or fewer significant bits than we want already,
        // so we can just return it outright.
        return mantissa;
    }

    // Truncate until we have the number of bits we want
    let shift = original_bits - bits;
    mantissa >> shift
}
