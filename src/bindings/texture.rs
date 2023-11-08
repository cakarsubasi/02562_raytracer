use std::{fs::File, path::Path};

use anyhow::*;
use image::{GenericImageView, io::Reader};

use super::{Bindable, WgslBindDescriptor};

pub struct TextureInfo {
    pub name: String,
    pub sampler_name: String,
    pub samplers: [bool; 3],
}

pub struct Texture {
    pub name: String,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler_default: Option<(String, wgpu::Sampler)>,
    sampler_bilinear: Option<(String, wgpu::Sampler)>,
    sampler_no_filtering: Option<(String, wgpu::Sampler)>,
}

impl Texture {
    pub fn from_file<P>(
        info: TextureInfo,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        file_name: P,
    ) -> Result<Self>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let file = File::open(file_name.as_ref())?;
        let image = Reader::open(file_name)?.decode()?;
        Self::from_image(info, device, queue, &image)
    }

    pub fn from_bytes(
        info: TextureInfo,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(info, device, queue, &img)
    }

    pub fn from_image(
        info: TextureInfo,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage
    ) -> Result<Self> {
        let (texture, view, sampler_default, sampler_bilinear, sampler_no_filtering) =
            Self::build(device, queue, img, &info.name);

        Ok(Self {
            name: info.name,
            texture,
            view,
            sampler_default: if info.samplers[0] {
                Some((format!("{}", info.sampler_name), sampler_default))
            } else {
                None
            },
            sampler_bilinear: if info.samplers[0] {
                Some((format!("{}_bilinear", info.sampler_name), sampler_bilinear))
            } else {
                None
            },
            sampler_no_filtering: if info.samplers[0] {
                Some((
                    format!("{}_nearest", info.sampler_name),
                    sampler_no_filtering,
                ))
            } else {
                None
            },
        })
    }

    fn build(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::DynamicImage,
        label: &str,
    ) -> (
        wgpu::Texture,
        wgpu::TextureView,
        wgpu::Sampler,
        wgpu::Sampler,
        wgpu::Sampler,
    ) {
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler_default = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let sampler_bilinear = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let sampler_no_filtering = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        (
            texture,
            view,
            sampler_default,
            sampler_bilinear,
            sampler_no_filtering,
        )
    }
}

impl Bindable for Texture {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut entries = vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        }];

        if let Some((_, _)) = &self.sampler_default {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        if let Some((_, _)) = &self.sampler_bilinear {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        if let Some((_, _)) = &self.sampler_bilinear {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            });
        }

        entries
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }];
        if let Some((_, sampler)) = &self.sampler_default {
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            });
        }
        if let Some((_, sampler)) = &self.sampler_bilinear {
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            });
        }
        if let Some((_, sampler)) = &self.sampler_no_filtering {
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            });
        }
        entries
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        let mut bind_descriptors = vec![WgslBindDescriptor {
            struct_def: None,
            bind_type: None,
            var_name: self.name.as_str(),
            var_type: "texture_2d<f32>",
            extra_code: None,
        }];
        if let Some((name, _)) = &self.sampler_default {
            bind_descriptors.push(WgslBindDescriptor {
                struct_def: None,
                bind_type: None,
                var_name: name.as_str(),
                var_type: "sampler",
                extra_code: None,
            });
        }
        if let Some((name, _)) = &self.sampler_bilinear {
            bind_descriptors.push(WgslBindDescriptor {
                struct_def: None,
                bind_type: None,
                var_name: name.as_str(),
                var_type: "sampler",
                extra_code: None,
            });
        }
        if let Some((name, _)) = &self.sampler_no_filtering {
            bind_descriptors.push(WgslBindDescriptor {
                struct_def: None,
                bind_type: None,
                var_name: name.as_str(),
                var_type: "sampler",
                extra_code: None,
            });
        }

        bind_descriptors
    }
}

pub struct RenderDestination {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl RenderDestination {
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let (texture, view) = Self::build(device, size);

        Self { texture, view }
    }

    pub fn change_dimension(&mut self, device: &wgpu::Device, new_size: (u32, u32)) {
        let (new_texture, view) = Self::build(device, new_size);
        let old_texture = std::mem::replace(&mut self.texture, new_texture);
        //self.texture = texture;
        drop(old_texture);
        self.view = view;
    }

    fn build(device: &wgpu::Device, size: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Ping Pong Destination"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        return (texture, view);
    }

    pub fn update_view(&mut self) {
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }
}

impl Bindable for RenderDestination {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        }]
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        vec![WgslBindDescriptor {
            struct_def: None,
            bind_type: None,
            var_name: "renderTexture",
            var_type: "texture_2d<f32>",
            extra_code: None,
        }]
    }
}

pub struct RenderSource {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl RenderSource {
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> RenderSource {
        let (texture, view) = Self::build(device, size);

        Self { texture, view }
    }

    fn build(device: &wgpu::Device, size: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Ping Pong Source"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        return (texture, view);
    }

    pub fn change_dimension(&mut self, device: &wgpu::Device, new_size: (u32, u32)) {
        let (texture, view) = Self::build(device, new_size);
        self.texture = texture;
        self.view = view;
    }

    pub fn update_view(&mut self) {
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }
}
