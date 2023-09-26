use winit::{dpi::PhysicalSize, event::{VirtualKeyCode, ElementState}};

pub enum Command {
    Resize { new_size: PhysicalSize<u32> },
    Render { value: bool },
    KeyEvent {key: VirtualKeyCode, state: ElementState },
    Shutdown { value: bool },
}
