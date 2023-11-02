// Heavily based on: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs

use std::{env, iter, time::Instant};

use strum::IntoEnumIterator;

use crossbeam_channel::Sender;
use egui::{ClippedPrimitive, Context, FontDefinitions, FullOutput, Response, ScrollArea, Ui};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::{
    CommandEncoder, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceTexture,
    TextureFormat, TextureView,
};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowId},
};

use crate::{
    command::{Command, DisplayMode, ShaderType},
    gpu_handles::GPUHandles,
    scenes::{get_scenes, SceneDescriptor},
};

pub struct ControlPanel {
    pub window_id: WindowId,
    // Rendering state
    pub window: Window,
    pub surface: wgpu::Surface,
    pub platform: Platform,
    config: wgpu::SurfaceConfiguration,
    render_pass: RenderPass,
    // Scenes
    scenes: Vec<SceneDescriptor>,
    current_scene: String,
    // All of our buttons' state
    should_render: bool,
    camera_constant: f32,
    sphere_material: ShaderType,
    other_material: ShaderType,
    use_texture: bool,
    pixel_subdivision: u32,
    render_resolution: (u32, u32),
    display_mode: DisplayMode,
    max_samples: u32,
    scene_path: String,
    model_path: String,
}

impl ControlPanel {
    pub fn build(
        gpu_handles: &GPUHandles,
        event_loop: &EventLoop<()>,
        window_size: winit::dpi::PhysicalSize<u32>,
        window_padding: u32,
    ) -> Self {
        let window: Window = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title("control panel")
            .with_inner_size(window_size)
            .build(event_loop)
            .unwrap();

        window.set_outer_position(winit::dpi::PhysicalPosition::new(
            window_padding,
            window_padding,
        ));

        let surface: Surface = unsafe { gpu_handles.instance.create_surface(&window) }.unwrap();

        let size: PhysicalSize<u32> = window.inner_size();

        let caps: SurfaceCapabilities = surface.get_capabilities(&gpu_handles.adapter);
        let config: SurfaceConfiguration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&gpu_handles.device, &config);

        let platform: Platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let surface_format: TextureFormat =
            surface.get_capabilities(&gpu_handles.adapter).formats[0];
        let render_pass: RenderPass = RenderPass::new(&gpu_handles.device, surface_format, 1);

        let path = String::from("");
        let model = String::from("");
        let window_id: WindowId = window.id();

        ControlPanel {
            window,
            surface,
            config,
            render_pass,
            platform,
            should_render: true,
            camera_constant: 1.0,
            sphere_material: ShaderType::Lambertian,
            other_material: ShaderType::Lambertian,
            scene_path: path,
            model_path: model,
            use_texture: true,
            pixel_subdivision: 1,
            render_resolution: (800, 450),
            display_mode: DisplayMode::Stretch,
            max_samples: 4096,
            window_id,
            scenes: get_scenes(),
            current_scene: "Default".into(),
        }
    }

    // The control panel needs to send all of the relevant initial
    // values to the render engine, otherwise the values won't
    // be used until the buttons are used.
    pub fn initialize(&self, commands: &Sender<Command>) {
        commands
            .send(Command::Render {
                value: self.should_render,
            })
            .unwrap();
    }

    pub fn resize(&mut self, device: &wgpu::Device, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(device, &self.config);
            self.window.request_redraw();
        }
    }

    pub fn get_current_texture(&mut self) -> wgpu::SurfaceTexture {
        self.surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture")
    }

    pub fn redraw(
        &mut self,
        commands: &Sender<Command>,
        has_focus: &mut bool,
        redraw_gui: &mut bool,
        start_time: &Instant,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.platform
            .update_time(start_time.elapsed().as_secs_f64());

        let output_frame: SurfaceTexture = self.get_current_texture();
        let output_view: TextureView = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        self.platform.begin_frame();

        // Draw the control panel.
        self.ui(&self.platform.context(), commands, has_focus, redraw_gui);

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let full_output: FullOutput = self.platform.end_frame(Some(&self.window));
        let paint_jobs: Vec<ClippedPrimitive> =
            self.platform.context().tessellate(full_output.shapes);

        let mut encoder: CommandEncoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // Upload all resources for the GPU.
        let screen_descriptor: ScreenDescriptor = ScreenDescriptor {
            physical_width: self.config.width,
            physical_height: self.config.height,
            scale_factor: self.window.scale_factor() as f32,
        };
        let tdelta: egui::TexturesDelta = full_output.textures_delta;
        self.render_pass
            .add_textures(device, queue, &tdelta)
            .expect("add texture ok");
        self.render_pass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.render_pass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            )
            .unwrap();

        // Submit the commands.
        queue.submit(iter::once(encoder.finish()));

        // Redraw egui
        output_frame.present();

        // Cleanup
        self.render_pass
            .remove_textures(tdelta)
            .expect("remove texture ok");
    }

    fn ui(
        &mut self,
        context: &Context,
        commands: &Sender<Command>,
        has_focus: &mut bool,
        redraw_gui: &mut bool,
    ) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("control panel");

            // Basically all of our buttons
            ScrollArea::vertical().show(ui, |ui: &mut Ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    // Organize window button
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory_mut(|mem| mem.reset_areas());
                    }

                    // Dark/Light mode button
                    ui.horizontal(|ui: &mut Ui| {
                        ui.label("egui theme:");
                        egui::widgets::global_dark_light_mode_buttons(ui);
                    });

                    // Render button
                    // If not checked, the renderer won't redraw
                    ui.horizontal(|ui: &mut Ui| {
                        if ui.checkbox(&mut self.should_render, "Render").changed() {
                            commands
                                .send(Command::Render {
                                    value: self.should_render,
                                })
                                .unwrap()
                        };
                    });
                    self.create_resolution_ui(ui, commands);
                    self.create_scene_selection_ui(ui, commands);
                    self.create_path_ui(ui, commands, has_focus, redraw_gui);
                    self.create_basic_scene_ui(ui, commands);
                    self.create_texture_ui(ui, commands);
                    self.create_pixel_subdivision_ui(ui, commands);
                    self.create_max_sample_ui(ui, commands);
                });
            });
        });
    }

    fn create_path_ui(
        &mut self,
        ui: &mut Ui,
        commands: &Sender<Command>,
        has_focus: &mut bool,
        redraw_gui: &mut bool,
    ) {
        // Load different shaders
        ui.horizontal(|ui: &mut Ui| {
            let load_shader_button = ui.button("Load Shader");

            if load_shader_button.changed() {
                *redraw_gui = true;
            };

            if load_shader_button.clicked() {
                commands
                    .send(Command::LoadShader {
                        shader_path: self.scene_path.clone(),
                    })
                    .unwrap();
            }

            ui.label("Path");

            let text_edit_singleline_response: Response =
                ui.text_edit_singleline(&mut self.scene_path);
            if text_edit_singleline_response.gained_focus() {
                *has_focus = true;
                *redraw_gui = true;
            }
            if text_edit_singleline_response.lost_focus() {
                *has_focus = false;
            }
        });

        // This button opens a file dialog and
        // sets the scene_path to that path.
        ui.horizontal(|ui: &mut Ui| {
            if ui.button("Open file..").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(env::current_dir().unwrap())
                    .add_filter("WGSL Shaders (*.wgsl)", &["wgsl"])
                    .pick_file()
                {
                    self.scene_path = path.display().to_string();
                }
            }
        });

        // load different models
        ui.horizontal(|ui: &mut Ui| {
            let load_model_button = ui.button("Load Model");

            if load_model_button.changed() {
                *redraw_gui = true;
            };

            if load_model_button.clicked() {
                eprintln!("Load model not implemented yet.");
                //commands
                //    .send(Command::LoadModel {
                //        shader_path: self.scene_path.clone(),
                //    })
                //    .unwrap();
            }

            ui.label("Path");

            let text_edit_singleline_response: Response =
                ui.text_edit_singleline(&mut self.scene_path);
            if text_edit_singleline_response.gained_focus() {
                *has_focus = true;
                *redraw_gui = true;
            }
            if text_edit_singleline_response.lost_focus() {
                *has_focus = false;
            }
        });

        // This button opens a file dialog and
        // sets the model_path to that path.
        ui.horizontal(|ui: &mut Ui| {
            if ui.button("Open file..").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(env::current_dir().unwrap())
                    .add_filter("Wavefront OBJ (*.obj)", &["obj"])
                    .pick_file()
                {
                    self.model_path = path.display().to_string();
                }
            }
        });
    }

    fn create_basic_scene_ui(&mut self, ui: &mut Ui, commands: &Sender<Command>) {
        ui.horizontal(|ui: &mut Ui| {
            ui.label("Camera constant");
            let camera_constant: Response = ui.add(
                egui::widgets::DragValue::new(&mut self.camera_constant)
                    .clamp_range(0.1..=10.0)
                    .fixed_decimals(1)
                    .speed(0.1),
            );
            if camera_constant.changed() {
                commands
                    .send(Command::SetCameraConstant {
                        constant: self.camera_constant,
                    })
                    .unwrap();
            }
        });

        ui.horizontal(|ui: &mut Ui| {
            egui::ComboBox::from_label("Sphere Material")
                .selected_text(format!("{:?}", self.sphere_material))
                .show_ui(ui, |ui| {
                    for material_type in ShaderType::iter() {
                        let type_str: &'static str = material_type.into();
                        if ui
                            .selectable_value(&mut self.sphere_material, material_type, type_str)
                            .clicked()
                        {
                            commands
                                .send(Command::SetSphereMaterial {
                                    material: self.sphere_material,
                                })
                                .unwrap();
                        }
                    }
                });
        });

        ui.horizontal(|ui: &mut Ui| {
            egui::ComboBox::from_label("Other Material")
                .selected_text(format!("{:?}", self.other_material))
                .show_ui(ui, |ui| {
                    for material_type in ShaderType::iter() {
                        let type_str: &'static str = material_type.into();
                        if ui
                            .selectable_value(&mut self.other_material, material_type, type_str)
                            .clicked()
                        {
                            commands
                                .send(Command::SetOtherMaterial {
                                    material: self.other_material,
                                })
                                .unwrap();
                        }
                    }
                });
        });
    }

    fn create_scene_selection_ui(&mut self, ui: &mut Ui, commands: &Sender<Command>) {
        ui.horizontal(|ui: &mut Ui| {
            egui::ComboBox::from_label("Scene")
                .selected_text(format!("{}", self.current_scene))
                .show_ui(ui, |ui| {
                    for scene in &self.scenes {
                        if ui
                            .selectable_value(
                                &mut self.current_scene,
                                scene.name.clone(),
                                &scene.name,
                            )
                            .clicked()
                        {
                            commands
                                .send(Command::LoadScene {
                                    scene: scene.clone(),
                                })
                                .unwrap()
                        }
                    }
                })
        });
    }

    fn create_texture_ui(&mut self, ui: &mut Ui, commands: &Sender<Command>) {
        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.use_texture, "Use Texture").changed() {
                commands
                    .send(Command::SetTexture {
                        use_texture: self.use_texture as u32,
                        uv: (1.0, 1.0),
                    })
                    .unwrap()
            };
        });
    }

    fn create_pixel_subdivision_ui(&mut self, ui: &mut Ui, commands: &Sender<Command>) {
        ui.horizontal(|ui| {
            let slider = egui::Slider::new(&mut self.pixel_subdivision, 1..=crate::bindings::uniform::MAX_SUBDIVISION)
                .text("Pixel Subdivision")
                .clamp_to_range(true);
            if ui.add(slider).changed() {
                commands
                    .send(Command::SetPixelSubdivision {
                        level: self.pixel_subdivision,
                    })
                    .unwrap();
            }
        });
    }

    fn create_resolution_ui(&mut self, ui: &mut Ui, commands: &Sender<Command>) {
        ui.horizontal(|ui: &mut Ui| {
            ui.label("Resolution");
            let resolution_x: Response = ui.add(
                egui::widgets::DragValue::new(&mut self.render_resolution.0)
                    .clamp_range(256..=2000)
                    .speed(1),
            );
            let resolution_y: Response = ui.add(
                egui::widgets::DragValue::new(&mut self.render_resolution.1)
                    .clamp_range(256..=2000)
                    .speed(1),
            );
            let display_mode_changed = egui::ComboBox::from_label("Mode")
                .selected_text(format!("{:?}", self.display_mode))
                .show_ui(ui, |ui| {  
                    DisplayMode::iter().map(|display_mode| {
                        let type_str: &'static str = display_mode.into();
                        ui.selectable_value(&mut self.display_mode, display_mode, type_str).changed()
                }).fold(false, |acc, elem| acc || elem)
            }).inner.unwrap_or(false);

            if resolution_x.changed() || resolution_y.changed() || display_mode_changed {
                commands
                    .send(Command::SetResolution {
                        resolution: self.render_resolution,
                        display_mode: self.display_mode,
                    })
                    .unwrap();
            }
        });
    }

    fn create_max_sample_ui(&mut self, ui: &mut Ui, commands: &Sender<Command>) {
        ui.horizontal(|ui: &mut Ui| {
            ui.label("Max Samples");
            let samples = ui.add(
                egui::widgets::DragValue::new(&mut self.max_samples)
                    .clamp_range(1..=4096)
                    .fixed_decimals(1)
                    .speed(1),
            );

            if samples.changed() {
                commands
                .send(Command::SetSamples {
                    samples: self.max_samples
                })
                .unwrap();
            }
        });
    }
}
