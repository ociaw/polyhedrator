pub struct Shader {}

impl Shader {
    pub fn load_glsl_from_path<P: AsRef<std::path::Path>>(
        path: P,
        ty: glsl_to_spirv::ShaderType,
        device: &wgpu::Device,
    ) -> Result<wgpu::ShaderModule, std::io::Error> {
        let source = std::fs::read_to_string(path)?;
        let spirv_file = match glsl_to_spirv::compile(&source, ty) {
            Ok(file) => file,
            Err(e) => {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
            }
        };
        Self::load_spirv_from_file(spirv_file, device)
    }

    pub fn load_spirv_from_file(
        file: std::fs::File,
        device: &wgpu::Device,
    ) -> Result<wgpu::ShaderModule, std::io::Error> {
        let spirv = wgpu::read_spirv(file)?;
        Ok(device.create_shader_module(&spirv))
    }
}
