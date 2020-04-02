mod builder;
mod keys;
pub mod operators;
pub mod seeds;

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

    pub fn center_on_origin(&mut self) {
        let mut center = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
        for vert in self.vertices.iter() {
            center += vert.to_vec();
        }

        for vert in self.vertices.iter_mut() {
            *vert = *vert - center;
        }
        eprintln!("Recentered with adjustment of {:?}", center);
    }

    pub fn scale(&mut self, max_radius: f64) {
        let mut furthest = Point3::origin();
        let mut furthest_mag = 0.0;
        for vert in self.vertices.iter() {
            let mag = vert.to_vec().magnitude2();
            if mag > furthest_mag {
                furthest = *vert;
                furthest_mag = mag;
            }
        }

        let scale = max_radius / furthest.to_vec().magnitude();

        for vert in self.vertices.iter_mut() {
            *vert = *vert * scale;
        }
        eprintln!("Scaled {}", scale);
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
