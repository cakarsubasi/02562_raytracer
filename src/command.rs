use winit::{dpi::PhysicalSize, event::{VirtualKeyCode, ElementState}};
use strum_macros::{EnumIter, IntoStaticStr};

use crate::scenes::SceneDescriptor;

#[derive(Debug)]
pub enum Command {
    Resize { new_size: PhysicalSize<u32> },
    Render { value: bool },
    LoadShader { shader_path: String },
    LoadScene { scene: SceneDescriptor },
    SetCameraConstant { constant: f32 },
    SetSphereMaterial { material: ShaderType },
    SetOtherMaterial { material: ShaderType },
    SetPixelSubdivision { level: u32 },
    SetSamples { samples: u32 },
    SetTexture { use_texture: u32, uv: (f32, f32) },
    SetResolution { resolution: (u32, u32), display_mode: DisplayMode },
    KeyEvent {key: VirtualKeyCode, state: ElementState },
    Shutdown { value: bool },
}

#[derive(Copy, Clone, Debug, EnumIter, IntoStaticStr, PartialEq)]
pub enum DisplayMode {
    /// window size has 1-to-1 correspondance with the rendering resolution
    Exact,
    /// rendering resolution is independent from window size
    Stretch,
    /// Window can be adjusted vertically, horizontal size is automatically set
    /// based on rendering resolution
    FitVerticalAuto,
    /// Window can be adjusted arbitrarily, renderer has to keep the required
    // aspect ratio by filling in black bars
    PillarBox,
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