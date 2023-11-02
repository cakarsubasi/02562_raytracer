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
    SetTexture { use_texture: bool },
    SetOtherMaterial { material: ShaderType },
    KeyEvent {key: VirtualKeyCode, state: ElementState },
    Shutdown { value: bool },
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