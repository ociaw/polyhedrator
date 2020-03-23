use super::Mesh;

pub struct Update<'a> {
    pub mesh: Option<Mesh>,
    pub swap_desc: Option<&'a wgpu::SwapChainDescriptor>,
}

impl<'a> Default for Update<'a> {
    fn default() -> Self {
        Update {
            mesh: None,
            swap_desc: None,
        }
    }
}
