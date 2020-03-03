#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum VertexKey {
    Seed(u32),
    Midpoint(u32, u32),
    Centroid(u32),
}

impl VertexKey {
    pub fn midpoint(first: u32, second: u32) -> VertexKey {
        if first < second {
            VertexKey::Midpoint(first, second)
        } else {
            VertexKey::Midpoint(second, first)
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum FaceKey {
    Seed(u32),
    Vertex(u32),
    PyramidFace(u32, VertexKey),
}
