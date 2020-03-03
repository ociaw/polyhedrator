use cgmath::Vector3;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: cgmath::Point3<f32>,
    tex_coords: [f32; 2],
    normal: cgmath::Vector3<f32>,
}

impl Vertex {
    pub fn new(
        position: cgmath::Point3<f32>,
        tex_coords: [f32; 2],
        normal: Vector3<f32>,
    ) -> Vertex {
        Vertex {
            position,
            tex_coords,
            normal,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    vertex_indices: [u32; 3],
}

impl Triangle {
    pub const fn new(vertex_indices: [u32; 3]) -> Triangle {
        Triangle { vertex_indices }
    }
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn index_count(&self) -> u32 {
        (self.triangles.len() * (std::mem::size_of::<Triangle>() / std::mem::size_of::<u32>()))
            as u32
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn new(vertices: Vec<Vertex>, triangles: Vec<Triangle>) -> Mesh {
        Mesh {
            vertices,
            triangles,
        }
    }

    pub fn vertex_buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: (mem::size_of::<[f32; 3]>() + mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }

    pub fn from_vertex_groups(iter: impl IntoIterator<Item = Vec<Vertex>>) -> Self {
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        for group in iter {
            let first_index = vertices.len() as u32;
            let count = group.len();
            for vertex in group {
                vertices.push(vertex);
            }
            for i in 1..(count - 1) as u32 {
                triangles.push(super::mesh::Triangle::new([
                    first_index,
                    first_index + i,
                    first_index + i + 1,
                ]));
            }
        }

        Self::new(vertices, triangles)
    }
}
