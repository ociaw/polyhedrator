pub struct Texture {
    _diffuse_texture: wgpu::Texture,
    _diffuse_texture_view: wgpu::TextureView,
    _diffuse_sampler: wgpu::Sampler,
    diffuse_bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn from_buffer(
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
    ) -> Texture {
        let diffuse_image = image::load_from_memory(bytes).unwrap();
        let diffuse_rgba = diffuse_image.as_rgba8().unwrap();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let diffuse_buffer = device
            .create_buffer_mapped(diffuse_rgba.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&diffuse_rgba);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &diffuse_buffer,
                offset: 0,
                row_pitch: 4 * dimensions.0,
                image_height: dimensions.1,
            },
            wgpu::TextureCopyView {
                texture: &diffuse_texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            size,
        );

        queue.submit(&[encoder.finish()]);

        let diffuse_texture_view = diffuse_texture.create_default_view();
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
        });

        Texture {
            _diffuse_texture: diffuse_texture,
            _diffuse_texture_view: diffuse_texture_view,
            _diffuse_sampler: diffuse_sampler,
            diffuse_bind_group,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.diffuse_bind_group
    }
}
