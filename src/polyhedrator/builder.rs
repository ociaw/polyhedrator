use super::{Face, FaceKey, Polyhedron, Vertex, VertexKey};
use cgmath::Point3;
use fnv::FnvHashMap;
use std::hash::Hash;

pub struct Builder {
    flags: FnvHashMap<FaceKey, BuilderFace>,
    indices: FnvHashMap<VertexKey, u32>,
    vertices: Vec<Vertex>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            flags: FnvHashMap::default(),
            indices: FnvHashMap::default(),
            vertices: Vec::new(),
        }
    }

    pub fn build_polyhedron(self) -> Polyhedron {
        let mut faces = Vec::with_capacity(self.flags.len());

        for flag in self.flags {
            let face = flag.1;
            // Start at an arbitrary vertex
            let start = match face.first() {
                Some(v) => v,
                None => continue,
            };

            let mut indices = Vec::with_capacity(face.len());

            let mut current = start;
            loop {
                indices.push(self.indices[&current]);

                current = match face.find_next(current) {
                    Some(v) => v,
                    None => break, // TODO: Log error or panic
                };
                if current == start {
                    break;
                }
            }

            faces.push(Face { indices });
        }

        Polyhedron {
            vertices: self.vertices,
            faces,
        }
    }

    pub fn add_vertex(&mut self, key: VertexKey, position: Point3<f64>) {
        if self.indices.contains_key(&key) {
            // TODO: Either panic or return a Result with an Error
            return;
        }
        if self.indices.len() == u32::max_value() as usize {
            // TODO: Either panic or return a Result with an Error
            return;
        }

        let index = self.indices.len() as u32;
        let old = self.indices.insert(key, index);
        assert!(old.is_none());
        self.vertices.push(position);
        assert_eq!(self.vertices.len(), self.indices.len());
    }

    pub fn add_flag(&mut self, face: FaceKey, source: VertexKey, destination: VertexKey) {
        assert_ne!(source, destination);
        match self.flags.get_mut(&face) {
            Some(vertex_list) => {
                vertex_list.add_edge(source, destination);
            }
            None => {
                let mut vertex_list = BuilderFace::new();
                vertex_list.add_edge(source, destination);
                self.flags.insert(face, vertex_list);
            }
        }
    }
}

#[derive(Clone)]
struct BuilderFace {
    edges: Vec<Edge>,
}

impl BuilderFace {
    fn new() -> BuilderFace {
        Self::with_capacity(6)
    }

    fn with_capacity(capacity: usize) -> BuilderFace {
        BuilderFace {
            edges: Vec::with_capacity(capacity),
        }
    }

    fn find_next(&self, source: VertexKey) -> Option<VertexKey> {
        self.edges
            .iter()
            .find(|edge| edge.source == source)
            .map(|edge| edge.destination)
    }

    fn _find_previous(&self, destination: VertexKey) -> Option<VertexKey> {
        self.edges
            .iter()
            .find(|edge| edge.destination == destination)
            .map(|edge| edge.source)
    }

    fn first(&self) -> Option<VertexKey> {
        self.edges.first().map(|edge| edge.destination)
    }

    fn len(&self) -> usize {
        self.edges.len()
    }

    fn add_edge(&mut self, source: VertexKey, destination: VertexKey) {
        assert_ne!(source, destination);
        match self.find_next(source) {
            Some(existing) => {
                assert_eq!(destination, existing);
            }
            None => {
                self.edges.push(Edge::new(source, destination));
            }
        };
        self.sort();
    }

    fn sort(&mut self) {
        if self.edges.len() < 3 {
            // Well that was easy
            return;
        }

        for i in 1..self.edges.len() {
            let previous = self.edges[i - 1];
            let current = self.edges[i];
            if previous.destination != current.source {
                // The current item is not the previous item's destination, so find it and swap them
                for j in i + 1..self.edges.len() {
                    let candidate = self.edges[j];
                    if previous.destination == candidate.source {
                        // Candidate follows previous, so swap it with current
                        self.edges.swap(i, j);
                        break;
                    }
                }
            }
        }
        // If the list was complete, it is now sorted.
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
struct Edge {
    source: VertexKey,
    destination: VertexKey,
}

impl Edge {
    pub fn new(source: VertexKey, destination: VertexKey) -> Edge {
        assert_ne!(source, destination);
        Edge {
            source,
            destination,
        }
    }
}
