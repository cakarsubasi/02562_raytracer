/// Types for sending messages from the UI thread to the rendering thread
/// Based on code shared by A.B. Sørensen in
/// https://github.com/absorensen/the-guide/tree/main/m2_concurrency/code/egui-winit-wgpu-template
/// Apache License 2.0

use winit::{dpi::PhysicalSize, event::{VirtualKeyCode, ElementState}};
use strum_macros::{EnumIter, IntoStaticStr};

#[derive(Debug)]
pub enum Command {
    Resize { new_size: PhysicalSize<u32> },
    Render { value: bool },
    LoadScene { idx: usize },
    SetCameraConstant { constant: f32 },
    SetSphereMaterial { material: ShaderType },
    SetOtherMaterial { material: ShaderType },
    SetPixelSubdivision { level: u32 },
    SetSamples { samples: u32, enabled: bool },
    SetTexture { use_texture: TextureUse, uv_scale: (f32, f32) },
    SetResolution { resolution: (u32, u32), display_mode: DisplayMode },
    KeyEvent {key: VirtualKeyCode, state: ElementState },
    Shutdown { value: bool },
}

#[derive(Copy, Clone, Default, Debug, EnumIter, IntoStaticStr, PartialEq)]
pub enum DisplayMode {
    /// window size has 1-to-1 correspondance with the rendering resolution
    #[default]
    Exact,
    /// rendering resolution is independent from window size
    Stretch,
    /// Window is automatically adjusted to fit either the horizontal or vertical maximum
    FitAuto,
    /// Render as high as the window size!
    Window,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, EnumIter, IntoStaticStr)]
pub enum ShaderType {
    Lambertian = 0,
    Phong = 1,
    Mirror = 2,
    Transmit = 3,
    Glossy = 4,
    Normal = 5,
    BaseColor = 6,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, EnumIter, IntoStaticStr)]
pub enum TextureUse {
    NoTexture = 0,
    Default = 1,
    Bilinear = 2,
    Nearest = 3,
}

