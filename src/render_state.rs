use crate::bindings::bsp_tree::TraversalStructure;
use crate::bindings::create_bind_group_layouts;
use crate::bindings::storage_mesh::StorageMeshGpu;
use crate::bindings::texture::{RenderSource, TextureInfo};
use crate::command::DisplayMode;
use crate::mesh::Mesh;
use crate::SceneDescriptor;
use crate::{
    bindings::{
        create_bind_groups, create_shader_definitions,
        mesh::MeshGpu,
        texture::{RenderDestination, Texture},
        uniform::UniformGpu,
        vertex::{self, Vertex},
        Bindable, BufferOwner, IntoGpu,
    },
    camera::{Camera, CameraController},
    command::Command,
};
use wgpu;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowId},
};

use anyhow::*;

use std::fs::File;
use std::io::prelude::*;

const CAMERA_SPEED: f32 = 0.05;

pub struct RenderState {
    surface: wgpu::Surface,
    render_source: RenderSource,
    render_destination: RenderDestination,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    display_mode: DisplayMode,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
    mesh_direct: MeshGpu,
    camera: Camera,
    pub uniform: UniformGpu,
    textures: Vec<Texture>,
    mesh_handle: Option<StorageMeshGpu>,
    traversal_structure_handle: TraversalStructure,
    bind_groups: Vec<wgpu::BindGroup>,
    camera_controller: CameraController,
}

impl RenderState {
    pub async fn new(
        _event_loop: &EventLoop<()>,
        window: winit::window::Window,
        scene: &SceneDescriptor,
    ) -> Self {
        //let window = WindowBuilder::new().build(&event_loop).unwrap();
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let camera_controller = CameraController::new(CAMERA_SPEED);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits {
                            // lol, lmao
                            max_storage_buffers_per_shader_stage: 16,
                            ..Default::default()
                        }
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // This assumes sRGB surface
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: scene.res.0,
            height: scene.res.1,
            present_mode: wgpu::PresentMode::Immediate, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let camera = scene.camera.to_owned();

        let mesh_direct = MeshGpu::new(&device, vertex::VERTICES, vertex::INDICES);

        let render_source = RenderSource::new(&device, scene.res);
        let render_destination = RenderDestination::new(&device, scene.res);

        let handles = Self::setup_rendering(&device, &queue, &config, &scene, &render_destination)
            .await
            .unwrap();

        Self {
            window,
            surface,
            render_source,
            render_destination,
            display_mode: DisplayMode::Exact, // this should be fairly safe
            device,
            queue,
            config,
            size,
            render_pipeline_layout: handles.0,
            render_pipeline: handles.1,
            mesh_direct,
            camera,
            bind_groups: handles.2,
            uniform: handles.3,
            textures: handles.4,
            mesh_handle: handles.5,
            traversal_structure_handle: handles.6,
            camera_controller,
        }
    }

    async fn setup_rendering(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        scene: &SceneDescriptor,
        render_destination: &RenderDestination,
    ) -> Result<(
        wgpu::PipelineLayout,
        wgpu::RenderPipeline,
        Vec<wgpu::BindGroup>,
        UniformGpu,
        Vec<Texture>,
        Option<StorageMeshGpu>,
        TraversalStructure,
    )> {
        // Uniform variables
        let uniform = UniformGpu::new(&device);
        // load texture
        let texture_bytes = include_bytes!("../res/textures/grass.jpg");
        let mut textures = vec![Texture::from_bytes(
            TextureInfo {
                name: "texture0".into(),
                sampler_name: "sampler0".into(),
                samplers: [true, true, true],
            },
            &device,
            &queue,
            texture_bytes
        )
        .unwrap()];
        // load background
        if let Some(path) = &scene.background_hdri {
            
            let background = Texture::from_file(
                TextureInfo {
                    name: "hdri0".into(),
                    sampler_name: "hdri0_sampler".into(),
                    samplers: [true, false, false],
                },
                &device,
                &queue,
                path
            )?;

            textures.push(background);
        }

        // load model
        let model = &scene.model.as_ref().and_then(|m| Mesh::from_obj(m).ok());
        let mesh_handle = model.as_ref().and_then(|m| match scene.vertex_type {
            crate::scenes::VertexType::Split => Some(m.into_gpu_split(&device)),
            crate::scenes::VertexType::Combined => Some(m.into_gpu_combined(&device)),
        });
        // create and load the BSP
        // TODO: allow BVHs
        let traversal_structure = if let Some(model) = model {
            match scene.traverse_type {
                crate::scenes::TraverseType::Bsp => TraversalStructure::Bsp(model.bsp_tree().into_gpu(device)),
                crate::scenes::TraverseType::Bvh => TraversalStructure::Bvh(model.bvh().into_gpu(device)),
            }
        } else {
            TraversalStructure::None
        };

        // generate bind group layouts
        let handles = [
            Some(&uniform as &dyn Bindable),
            mesh_handle
                .as_ref()
                .and_then(|mesh| Some(mesh as &dyn Bindable)),
            Some(&traversal_structure as &dyn Bindable),
            Some(render_destination as &dyn Bindable),
        ]
        .into_iter()
        .flatten()
        .chain(textures.iter().map(|texture| texture as &dyn Bindable))
        .collect::<Vec<&dyn Bindable>>();

        let (render_pipeline_layout, bind_groups) =
            Self::recreate_bind_groups_impl(device, &handles);

        let mut shader_defs = Self::create_shader_defs_impl(&handles);

        let mut file = File::open(&scene.shader)?;
        let mut shader_source = String::new();
        file.read_to_string(&mut shader_source)?;

        let shader = Self::create_shader_module(&device, &mut shader_defs, &shader_source).await?;

        let render_pipeline = RenderState::create_render_pipeline(
            &device,
            Some(&render_pipeline_layout),
            &shader,
            &config,
        );

        Ok((
            render_pipeline_layout,
            render_pipeline,
            bind_groups,
            uniform,
            textures,
            mesh_handle,
            traversal_structure,
        ))
    }

    fn get_handles(&self) -> Vec<&dyn Bindable> {
        [
            Some(&self.uniform as &dyn Bindable),
            self.mesh_handle
                .as_ref()
                .and_then(|mesh| Some(mesh as &dyn Bindable)),
            Some(&self.traversal_structure_handle as &dyn Bindable),
            Some(&self.render_destination as &dyn Bindable),
        ]
        .into_iter()
        .flatten()
        .chain(self.textures.iter().map(|texture| texture as &dyn Bindable))
        .collect::<Vec<_>>()
    }

    fn recreate_bind_groups(&mut self) {
        (self.render_pipeline_layout, self.bind_groups) = Self::recreate_bind_groups_impl(&self.device, &self.get_handles());
    }

    fn recreate_bind_groups_impl(
        device: &wgpu::Device,
        handles: &Vec<&dyn Bindable>,
    ) -> (wgpu::PipelineLayout, Vec<wgpu::BindGroup>) {
        let bind_group_layout_entries = handles
            .iter()
            .flat_map(|&handle| handle.get_layout_entries())
            .collect::<Vec<_>>();
        let mut bind_group_entries = handles
            .iter()
            .flat_map(|&handle| handle.get_bind_group_entries())
            .collect::<Vec<_>>();
        let bind_group_layout = create_bind_group_layouts(&device, bind_group_layout_entries);
        let bind_group = create_bind_groups(device, &mut bind_group_entries, &bind_group_layout);
        let render_pipeline_layout =
            Self::create_render_pipeline_layout(&device, &vec![bind_group_layout]);
        (render_pipeline_layout, vec![bind_group])
    }

    fn create_shader_defs_impl(handles: &Vec<&dyn Bindable>) -> String {
        create_shader_definitions(
            &handles
                .iter()
                .map(|&handle| handle.get_bind_descriptor())
                .collect::<Vec<_>>(),
        )
    }

    pub fn load_scene(&mut self, scene: &SceneDescriptor) -> Result<()> {
        let handles = pollster::block_on(Self::setup_rendering(
            &self.device,
            &self.queue,
            &self.config,
            scene,
            &self.render_destination,
        ))?;
        self.render_pipeline_layout = handles.0;
        self.render_pipeline = handles.1;
        self.bind_groups = handles.2;
        self.uniform = handles.3;
        self.textures = handles.4;
        self.mesh_handle = handles.5;
        self.traversal_structure_handle = handles.6;
        // update uniforms
        self.camera = scene.camera.to_owned();
        // update resolution
        self.set_display_mode(scene.res, crate::command::DisplayMode::Stretch)?;
        Ok(())
    }

    pub fn recreate_render_pipeline<'a>(&'a mut self, shader: &wgpu::ShaderModule) {
        self.render_pipeline = Self::create_render_pipeline(
            &self.device,
            Some(&self.render_pipeline_layout),
            shader,
            &self.config,
        );
    }

    pub async fn create_shader_module_from_file(
        &self,
        shader_location: &std::path::Path,
    ) -> Result<wgpu::ShaderModule> {
        let mut file = File::open(shader_location)?;
        let mut shader_source = String::new();
        file.read_to_string(&mut shader_source)?;
        let shader_defs = Self::create_shader_defs_impl(&self.get_handles());
        Self::create_shader_module(&self.device, shader_defs.as_str(), &mut shader_source).await
    }

    async fn create_shader_module(
        device: &wgpu::Device,
        shader_defs: &str,
        shader_source: &str,
    ) -> Result<wgpu::ShaderModule> {
        let mut everything = String::from(shader_source);
        everything.push_str(shader_defs);
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader_maybe = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(everything.as_str().into()),
        });
        let error_maybe = device.pop_error_scope().await;
        if let Some(err) = error_maybe {
            return Err(anyhow!(err.to_string()));
        }

        Ok(shader_maybe)
    }

    fn create_render_pipeline_layout(
        device: &wgpu::Device,
        bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
    ) -> wgpu::PipelineLayout {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: bind_group_layouts
                    .iter()
                    .map(|v| v)
                    .collect::<Vec<_>>()
                    .as_ref(),
                push_constant_ranges: &[],
            });
        render_pipeline_layout
    }

    pub fn create_render_pipeline(
        device: &wgpu::Device,
        render_pipeline_layout: Option<&wgpu::PipelineLayout>,
        shader: &wgpu::ShaderModule,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: render_pipeline_layout,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        render_pipeline
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_id(&self) -> WindowId {
        self.window().id()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if self.display_mode == DisplayMode::Window {
            self.set_render_resolution((new_size.width, new_size.height));
        }
    }

    // return true to stop processing events, right now, always return false
    pub fn input_alt(&mut self, command: &Command) -> bool {
        self.camera_controller.handle_camera_commands(command);
        false
    }

    pub fn update(&mut self) {
        self.camera.aspect = self.aspect_ratio();
        self.camera_controller.update_camera(&mut self.camera);
        self.uniform.update(
            Some(&self.camera),
            None,
            None,
            None, // TODO
            Some((self.config.width, self.config.height)),
        );
        self.uniform.update_buffer(&self.queue);
        self.render_destination.update_view();
        self.render_source.update_view();
        self.recreate_bind_groups();
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let source_view = self
            .render_source
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &source_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, self.mesh_direct.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.mesh_direct.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        for (idx, group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(idx as u32, group, &[]);
        }
        render_pass.draw_indexed(0..self.mesh_direct.num_indices, 0, 0..1);

        drop(render_pass);

        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &self.render_source.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &self.render_destination.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            self.render_source.texture.size(),
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        std::result::Result::Ok(())
    }

    pub fn aspect_ratio(&self) -> f32 {
        //self.config.width as f32 / self.config.height as f32
        self.window.inner_size().width as f32 / self.window.inner_size().height as f32
    }

    pub fn set_display_mode(
        &mut self,
        resolution: (u32, u32),
        display_mode: DisplayMode,
    ) -> Result<()> {
        self.set_render_resolution(resolution);
        // TODO: use display_mode to do funny things
        self.display_mode = display_mode;
        Ok(())
    }

    fn set_render_resolution(&mut self, resolution: (u32, u32)) {
        if resolution.0 > 0 && resolution.1 > 0 {
            self.config.width = resolution.0;
            self.config.height = resolution.1;
            self.surface.configure(&self.device, &self.config);
            self.render_destination
                .change_dimension(&self.device, (self.config.width, self.config.height));
            self.render_source
                .change_dimension(&self.device, (self.config.width, self.config.height));
        }
    }

    pub fn update_camera_constant(&mut self, constant: f32) {
        self.camera.constant = constant;
    }
}
