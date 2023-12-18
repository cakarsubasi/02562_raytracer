/// UI and Rendering thread code for a shader based path tracer
/// Based on code shared by A.B. Sørensen in
/// https://github.com/absorensen/the-guide/tree/main/m2_concurrency/code/egui-winit-wgpu-template
/// Apache License 2.0

mod bindings;
mod camera;
mod command;
mod control_panel;
pub mod data_structures;
mod gpu_handles;
pub mod mesh;
mod render_state;
mod scenes;
mod tools;

use std::{path::Path, sync::Arc, thread, time::Instant};

use crate::{
    control_panel::ControlPanel,
    render_state::RenderState,
    scenes::{get_scenes, SceneDescriptor},
};

/*
Boilerplate code from https://sotrh.github.io/learn-wgpu/
*/

use command::Command;
use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use gpu_handles::GPUHandles;
use tools::RenderStats;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowId,
};

// Simple wrapper to handle different window ids.
struct WindowSelector {
    control_panel_id: u64,
    render_panel_id: u64,
}

impl WindowSelector {
    pub fn new(control_panel_id: WindowId, render_panel_id: WindowId) -> Self {
        let control_panel_id: u64 = control_panel_id.into();
        let render_panel_id: u64 = render_panel_id.into();
        WindowSelector {
            control_panel_id,
            render_panel_id,
        }
    }

    #[inline(always)]
    pub fn select_window(&self, id: &WindowId) -> usize {
        let id: u64 = (*id).into();
        if id == self.control_panel_id {
            0
        } else if id == self.render_panel_id {
            1
        } else {
            1000
        }
    }
}

fn main_thread(
    gpu_handles: GPUHandles,
    event_loop: EventLoop<()>,
    window_selector: WindowSelector,
    mut control_panel: ControlPanel,
    transmitter: Sender<Command>,
) {
    let start_time: Instant = Instant::now();

    // For handling constant redrawing of specific control panel widgets.
    let mut redraw_gui: bool = false;
    let mut gui_has_focus: bool = false;

    control_panel.initialize(&transmitter);

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&gpu_handles.instance, &gpu_handles.adapter);
        let transmitter: &Sender<Command> = &transmitter;

        control_panel.platform.handle_event(&event); // In doubt about this one
                                                     // *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::MouseInput { state, .. } => {
                    match state {
                        // Always redraw the control panel when a button has been pressed
                        // or released.
                        ElementState::Pressed => {
                            if window_selector.select_window(&window_id) == 0 {
                                control_panel.redraw(
                                    transmitter,
                                    &mut gui_has_focus,
                                    &mut redraw_gui,
                                    &start_time,
                                    &gpu_handles.device,
                                    &gpu_handles.queue,
                                );
                            }
                        }
                        ElementState::Released => {
                            if window_selector.select_window(&window_id) == 0 {
                                control_panel.redraw(
                                    transmitter,
                                    &mut gui_has_focus,
                                    &mut redraw_gui,
                                    &start_time,
                                    &gpu_handles.device,
                                    &gpu_handles.queue,
                                );
                            }
                        }
                    }
                }

                // Redraw the control panel when the cursor moves on it.
                // The render engine will always redraw anyway.
                WindowEvent::CursorMoved { .. } => {
                    if window_selector.select_window(&window_id) == 0 {
                        control_panel.redraw(
                            transmitter,
                            &mut gui_has_focus,
                            &mut redraw_gui,
                            &start_time,
                            &gpu_handles.device,
                            &gpu_handles.queue,
                        );
                    }
                }

                // Handle resizing of the specific window.
                WindowEvent::Resized(size) => match window_selector.select_window(&window_id) {
                    0 => control_panel.resize(&gpu_handles.device, size),
                    1 => transmitter
                        .send(Command::Resize { new_size: size })
                        .unwrap(),
                    _ => (),
                },

                // Handle shutdown by clicking the close button in the upper right.
                WindowEvent::CloseRequested => {
                    transmitter.send(Command::Shutdown { value: true }).unwrap();
                    *control_flow = ControlFlow::Exit;
                }

                // Most of the keyboard events are sent directly to the render engine
                // through the transmitter channel. In general, it is assumed that
                // you don't control the GUI through the keyboard.
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    use VirtualKeyCode::*;
                    match key {
                        Escape => {
                            transmitter.send(Command::Shutdown { value: true }).unwrap();
                            *control_flow = ControlFlow::Exit;
                        }
                        virtual_key_code => transmitter
                            .send(Command::KeyEvent {
                                key: virtual_key_code,
                                state: state,
                            })
                            .unwrap(),
                    }
                }
                _ => (),
            },

            // Only redraw the control panel for specific redraw requests.
            // This is to keep the control panel light on processing.
            // The render engine is running on its own thread and redraws
            // every single frame, so no redraw request needed.
            Event::RedrawRequested(window_id) => {
                if window_selector.select_window(&window_id) == 0 {
                    control_panel.redraw(
                        transmitter,
                        &mut gui_has_focus,
                        &mut redraw_gui,
                        &start_time,
                        &gpu_handles.device,
                        &gpu_handles.queue,
                    )
                }
            }

            // This event happens once all the other events have been cleared.
            // The additional redraws are for when a GUI element has focus and
            // needs to be constantly redrawn. It could for example be the
            // text entry widget.
            Event::MainEventsCleared => {
                if redraw_gui || gui_has_focus {
                    redraw_gui = false;
                    control_panel.redraw(
                        transmitter,
                        &mut gui_has_focus,
                        &mut redraw_gui,
                        &start_time,
                        &gpu_handles.device,
                        &gpu_handles.queue,
                    );
                }
            }

            _ => {}
        }
    });
}

// Our render and control window sizes, and space between them.
const RENDER_WINDOW_SIZE: winit::dpi::PhysicalSize<u32> = winit::dpi::PhysicalSize::new(1420, 1080);
const CONTROL_WINDOW_SIZE: winit::dpi::PhysicalSize<u32> = winit::dpi::PhysicalSize::new(400, 1080);
const WINDOW_PADDING: u32 = 16;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    if !gpu_handles::self_test() {
        panic!("Unable to find a GPU adapter");
    }

    let event_loop = EventLoop::new();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    let gpu_handles = GPUHandles::new();

    let scenes = get_scenes();


    let render_state_window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("engine panel")
        .with_inner_size(RENDER_WINDOW_SIZE)
        .build(&event_loop)
        .unwrap();

    render_state_window.set_outer_position(winit::dpi::PhysicalPosition::new(
        2 * WINDOW_PADDING + CONTROL_WINDOW_SIZE.width,
        WINDOW_PADDING,
    ));

        // Create control panel
        let control_panel: ControlPanel = ControlPanel::build(
            &gpu_handles,
            &event_loop,
            CONTROL_WINDOW_SIZE,
            WINDOW_PADDING,
            scenes.clone(),
        );
    

    let mut render_state = RenderState::new(&event_loop, render_state_window, &scenes[0]).await;

    let (transmitter, receiver): (Sender<Command>, Receiver<Command>) = unbounded::<Command>();
    // Create the window selector which will be used for
    // matching events to the relevant window.
    let window_selector: WindowSelector =
        WindowSelector::new(control_panel.window_id, render_state.window_id());

    let _render_thread = thread::Builder::new()
        .name("Render Thread".into())
        .spawn(move || rendering_thread(&mut render_state, receiver, scenes.clone()));

    // Set initial state
    control_panel.force_send_all(&transmitter);

    main_thread(
        gpu_handles,
        event_loop,
        window_selector,
        control_panel,
        transmitter,
    );
}

fn rendering_thread(
    render_state: &mut RenderState,
    receiver: Receiver<Command>,
    scenes: Arc<[SceneDescriptor]>,
) {
    let mut should_render = true;
    let mut progressive = false;

    let mut render_statistics = RenderStats::new();

    loop {
        let current_iter = render_state.uniform.get_iteration();
        let max_iter = render_state.uniform.max_iterations;
        if should_render && (progressive && current_iter < max_iter) || (should_render && !progressive) {
            render_statistics.begin_capture();
            thread::scope(|s| {
                s.spawn(|| {
                    match render_state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            panic!("out of memory")
                        }
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(_) => {},
                    }
                    render_statistics.end_capture();

                    if progressive {
                        println!("Current iter: {}/{}", current_iter, max_iter);
                        render_state.uniform.increase_iteration();
                    }

                    if render_statistics.total > std::time::Duration::from_secs(5) {
                        println!("{render_statistics}");
                        render_statistics.reset();
                    }
                });
            });
            render_state.update();
        }

        loop {
            // This will be the end of me with how much it actually affects performance
            match receiver.recv_timeout(std::time::Duration::from_millis(1)) {
                Err(RecvTimeoutError::Timeout) => break,
                Err(_err) => break,
                Ok(command) => {
                    render_state.input_alt(&command);
                    match command {
                        Command::Resize { new_size } => {
                            render_state.resize(new_size);
                        }
                        Command::KeyEvent {
                            key,
                            state: ElementState::Pressed,
                        } => match key {
                            VirtualKeyCode::Space => {
                                match render_state.load_scene(&scenes[0]) {
                                    Ok(_) => eprintln!("Successfully loaded default scene."),
                                    Err(err) => eprintln!("Failed to load default scene: {}", err),
                                }
                            }
                            _ => {}
                        },
                        // Dont do anything on key release because we are lazy
                        Command::KeyEvent {
                            key: _,
                            state: ElementState::Released,
                        } => {}
                        Command::Shutdown { value } => {
                            if value {
                                break;
                            };
                        }
                        Command::Render { value } => {
                            should_render = value;
                        }
                        Command::SetCameraConstant { constant } => {
                            render_state.update_camera_constant(constant);
                        }
                        Command::SetSphereMaterial { material } => {
                            render_state
                                .uniform
                                .update_sphere_selection(material as u32);
                        }
                        Command::SetOtherMaterial { material } => {
                            render_state.uniform.update_other_selection(material as u32);
                        }
                        Command::SetPixelSubdivision { level } => {
                            render_state.uniform.update_subdivision_level(level);
                        }
                        Command::SetTexture {
                            use_texture,
                            uv_scale,
                        } => {
                            render_state.uniform.update_use_texture(use_texture as u32);
                            render_state.uniform.update_uv_scale(uv_scale);

                        }
                        Command::SetResolution {
                            resolution,
                            display_mode,
                        } => {
                            let (new_window_size, new_resolution) = match display_mode {
                                command::DisplayMode::Exact => {
                                    (PhysicalSize::new(resolution.0, resolution.1), resolution)
                                }
                                command::DisplayMode::Stretch => (RENDER_WINDOW_SIZE, resolution),
                                command::DisplayMode::FitAuto => {
                                    let max_aspect_ratio = RENDER_WINDOW_SIZE.width as f32
                                        / RENDER_WINDOW_SIZE.height as f32;
                                    let current_aspect_ratio =
                                        resolution.0 as f32 / resolution.1 as f32;
                                    let (width, height) = if current_aspect_ratio > max_aspect_ratio
                                    {
                                        // wider, fix the size horizontally
                                        let width = RENDER_WINDOW_SIZE.width;
                                        let height = (width as f32 / current_aspect_ratio) as u32;
                                        (width, height)
                                    } else {
                                        // taller, fix the size vertically
                                        let height = RENDER_WINDOW_SIZE.height;
                                        let width = (height as f32 * current_aspect_ratio) as u32;
                                        (width, height)
                                    };
                                    (PhysicalSize::new(width, height), resolution)
                                }
                                command::DisplayMode::Window => {
                                    let current_size = render_state.window().inner_size();
                                    let resolution = (current_size.width, current_size.height);
                                    (current_size, resolution)
                                }
                            };
                            render_state
                            .set_display_mode(new_resolution, display_mode)
                            .unwrap();

                            render_state.window().set_inner_size(new_window_size);
                        }
                        Command::LoadScene { idx } => match render_state.load_scene(&scenes[idx]) {
                            Ok(_) => 
                            {
                                render_statistics.reset();
                                render_state.uniform.reset_iteration();
                                render_state.uniform.max_iterations = max_iter;
                                eprintln!("Successfully loaded new scene: {:?}", scenes[idx])
                            }
                            Err(err) => eprintln!("{err}"),
                        },
                        Command::SetSamples { samples, enabled } => {
                            progressive = enabled;
                            if enabled {
                                render_state.uniform.max_iterations = samples;
                            } else {
                                render_state.uniform.max_iterations = 2;
                            }
                        }
                        #[allow(unreachable_patterns)]
                        _other => {
                            eprintln!("Detected and dropped command {_other:?}");
                        }
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn load_shader(render_state: &mut RenderState, shader_path: &str) {
    let shader_module =
        pollster::block_on(render_state.create_shader_module_from_file(Path::new(shader_path)));
    match shader_module {
        Ok(module) => {
            render_state.recreate_render_pipeline(&module);
            eprintln!("Successfully loaded shader.")
        }
        Err(err) => {
            eprintln!("{err}")
        }
    }
}
