use cgmath::InnerSpace;

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    eye_unit_vector: cgmath::Vector3<f32>,
    magnitude: f32,
    aspect_ratio: f32,
    vertical_fov: f32,
    near_depth: f32,
    far_depth: f32,
}

impl Camera {
    pub fn default(aspect_ratio: f32) -> Camera {
        Camera {
            eye_unit_vector: (1.0, 0.0, 0.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            magnitude: 3.0,
            aspect_ratio,
            vertical_fov: 50.0,
            near_depth: 0.1,
            far_depth: 100.0,
        }
    }

    pub fn up(&self) -> cgmath::Vector3<f32> {
        self.up
    }

    pub fn eye(&self) -> cgmath::Vector3<f32> {
        self.eye_unit_vector
    }

    pub fn orbit_horizontal(&mut self, angle: cgmath::Rad<f32>) {
        let axis = self.up;
        let rotation_matrix = cgmath::Matrix3::from_axis_angle(axis, angle);
        self.eye_unit_vector = (rotation_matrix * self.eye_unit_vector).normalize();
    }

    pub fn orbit_vertical(&mut self, angle: cgmath::Rad<f32>) {
        let axis = self.eye_unit_vector.cross(self.up).normalize();
        let rotation_matrix = cgmath::Matrix3::from_axis_angle(axis, angle);
        self.eye_unit_vector = (rotation_matrix * self.eye_unit_vector).normalize();
        self.up = (rotation_matrix * self.up).normalize();
    }

    pub fn rotate_in_place(&mut self, angle: cgmath::Rad<f32>) {
        let axis = self.eye_unit_vector;
        let rotation_matrix = cgmath::Matrix3::from_axis_angle(axis, angle);
        self.up = (rotation_matrix * self.up).normalize();
    }

    pub fn set_zoom(&mut self, magnitude: f32) {
        if magnitude <= 0.0 {
            return;
        }
        self.magnitude = magnitude;
    }

    pub fn move_eye(&mut self, eye: cgmath::Point3<f32>) {
        let vector = eye - self.target;
        self.magnitude = vector.magnitude();
        self.eye_unit_vector = vector.normalize();
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let eye = self.target + self.eye_unit_vector * self.magnitude;
        let view = cgmath::Matrix4::look_at(eye, self.target, self.up);
        let proj = cgmath::perspective(
            cgmath::Deg(self.vertical_fov),
            self.aspect_ratio,
            self.near_depth,
            self.far_depth,
        );

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

pub struct CameraController {
    speed: f32,
    zoom_factor: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_cw_pressed: bool,
    is_ccw_pressed: bool,
}

use winit::event::*;

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            zoom_factor: 10.0,
            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_cw_pressed: false,
            is_ccw_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_x, y) => {
                    self.zoom_factor = f32::max(1.0, self.zoom_factor - *y);
                    true
                }
                _ => false,
            },
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        scancode,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                let mut cont = match scancode {
                    0x10 => {
                        // Q
                        self.is_cw_pressed = is_pressed;
                        true
                    }
                    0x12 => {
                        // E
                        self.is_ccw_pressed = is_pressed;
                        true
                    }
                    0x11 => {
                        // W
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    0x1e => {
                        // A
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    0x1f => {
                        // S
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    0x20 => {
                        // D
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                };
                cont |= match keycode {
                    VirtualKeyCode::Up => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Down => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                };
                cont
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        camera.set_zoom(self.zoom_factor / 2.0);

        let min_zoom = 4.0;
        let max_zoom = 20.0;
        let zoom_range = max_zoom - min_zoom;

        let min_speed = self.speed / 8.0;
        let max_speed = self.speed;
        let speed_range = max_speed - min_speed;

        let conversion_factor = speed_range / zoom_range;

        let zoom = self.zoom_factor.min(max_zoom).max(min_zoom);
        let speed = (zoom - min_zoom) * conversion_factor + min_speed;
        let move_angle = cgmath::Rad(speed);
        let rotate_angle = cgmath::Rad(self.speed);

        if self.is_cw_pressed {
            camera.rotate_in_place(rotate_angle);
        }
        if self.is_ccw_pressed {
            camera.rotate_in_place(-rotate_angle);
        }

        if self.is_up_pressed {
            camera.orbit_vertical(move_angle);
        }
        if self.is_down_pressed {
            camera.orbit_vertical(-move_angle);
        }

        if self.is_right_pressed {
            camera.orbit_horizontal(move_angle);
        }
        if self.is_left_pressed {
            camera.orbit_horizontal(-move_angle);
        }
    }
}
