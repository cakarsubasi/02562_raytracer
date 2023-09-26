use winit::{dpi::PhysicalSize, event::VirtualKeyCode};

pub enum Command {
    Resize { new_size: PhysicalSize<u32> },
    Render { value: bool },
    KeyEvent {key: VirtualKeyCode },
    Shutdown { value: bool },
}
