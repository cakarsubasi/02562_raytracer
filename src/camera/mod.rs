use winit::event::*;

use crate::command::Command;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub constant: f32,
    pub znear: f32,
    pub zfar: f32,
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn handle_camera_commands(&mut self, command: &Command) -> bool {
        match command {
            Command::KeyEvent { key: VirtualKeyCode::W | VirtualKeyCode::Up } => {
                self.is_forward_pressed = true;
                true
            }
            Command::KeyEvent { key: VirtualKeyCode::A | VirtualKeyCode::Left } => {
                self.is_left_pressed = true;
                true
            }
            Command::KeyEvent { key: VirtualKeyCode::S | VirtualKeyCode::Down } => {
                self.is_backward_pressed = true;
                true
            }
            Command::KeyEvent { key: VirtualKeyCode::D | VirtualKeyCode::Right } => {
                self.is_right_pressed = true;
                true
            }
            _ => false
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the fowrard/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so 
            // that it doesn't change. The eye therefore still 
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
