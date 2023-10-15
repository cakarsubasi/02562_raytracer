use wgpu;
use winit::{window::{Window, WindowId}, event_loop::EventLoop};
use crate::{
    texture,
    camera::{Camera, CameraController},
    uniform::{self, Uniform, Vertex, BindGroup}, command::Command, mesh::{Mesh, MeshGPU}, data_structures::bsp_tree::BspTreeGpu};

use anyhow::*;

use std::fs::File;
use std::io::prelude::*;

use wgpu::util::DeviceExt;

const CAMERA_SPEED: f32 = 0.05;

pub struct RenderState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    camera: Camera,
    uniform: Uniform,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    mesh_handle: MeshGPU,
    bsp_tree_handle: BspTreeGpu,
    time_of_last_render: std::time::Instant,
    camera_controller: CameraController,
}

impl RenderState {
    pub async fn new(_event_loop: &EventLoop<()>, window: winit::window::Window) -> Self {

        //let window = WindowBuilder::new().build(&event_loop).unwrap();
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::BROWSER_WEBGPU,
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
                        wgpu::Limits::default()
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
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (2.0, 1.5, 2.0).into(),
            target: (0.0, 0.5, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            constant: 1.0,
            aspect: config.width as f32 / config.height as f32,
            znear: 0.1,
            zfar: 100.0,
        };

        // Uniform variables
        let uniform = Uniform::new();
        let uniform_bg = uniform.create_bind_group(&device);
        let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) = uniform_bg;

        // load texture
        let texture_bytes = include_bytes!("../res/textures/grass.jpg");
        let texture = texture::Texture::from_bytes(&device, &queue, texture_bytes, "grass.jpg").unwrap();
        let texture_bg = texture.create_bind_group(&device);
        let (texture_bind_group_layout, texture_bind_group) = texture_bg;

        // load model
        let mut model = Mesh::from_obj("res/models/CornellBox.obj").expect("Failed to load model");
        //eprintln!("{model}");
        model.scale(1f32 / 300f32);
        let mesh_handle = model.into_gpu(&device);

        // create and load the BSP
        let bsp_tree = model.bsp_tree();
        let bsp_tree_handle = bsp_tree.into_gpu(&device);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout, &mesh_handle.layout, &bsp_tree_handle.layout],
            push_constant_ranges: &[],
        }
        );

        let render_pipeline = RenderState::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &shader,
            &config);

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(uniform::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(uniform::INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = uniform::INDICES.len() as u32;

        let time_of_last_render = std::time::Instant::now();

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            uniform,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group,
            mesh_handle,
            bsp_tree_handle,
            camera_controller,
            time_of_last_render,
        }
    }

    pub fn recreate_render_pipeline<'a>(&'a mut self, shader: &wgpu::ShaderModule) {
        self.render_pipeline = Self::create_render_pipeline(&self.device, &self.render_pipeline_layout, shader, &self.config);
    }

    pub async fn create_shader_module(&self, shader_location: &str) -> Result<wgpu::ShaderModule> {
        let mut file = File::open::<std::path::PathBuf>(shader_location.into())?;
        let mut shader_source = String::new();
        file.read_to_string(&mut shader_source)?;

        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader_maybe = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.as_str().into()),
        });
        let error_maybe = self.device.pop_error_scope().await;
        if let Some(err) = error_maybe {
            return Err(anyhow!(err.to_string()));
        }

        Ok(shader_maybe)
    }

    pub fn create_render_pipeline(
        device: &wgpu::Device, 
        render_pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
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
        self.uniform.update(&self.camera);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let current_time: std::time::Instant = std::time::Instant::now();
        let _time_delta: std::time::Duration = current_time - self.time_of_last_render;
        self.time_of_last_render = current_time;

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        
        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(2, &self.mesh_handle.bind_group, &[]);
        render_pass.set_bind_group(3, &self.bsp_tree_handle.bind_group, &[]);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        std::result::Result::Ok(())
    }

    //pub fn delta_time(&self) -> std::time::Duration {
    //    let now = std::time::Instant::now();
    //    return now - self.time_of_last_render;
    //}

    pub fn aspect_ratio(&self) -> f32 {
        //self.config.width as f32 / self.config.height as f32
        self.window.inner_size().width as f32 / self.window.inner_size().height as f32
    }
}