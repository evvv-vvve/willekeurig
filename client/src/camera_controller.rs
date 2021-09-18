use std::{f32::consts::FRAC_PI_2};

use cgmath::{Rad, Vector3, Zero};

use world::World;
use renderer::camera::Camera;

#[derive(Debug)]
pub struct CameraController {
    velocity: Vector3<f32>,

    rotate_horizontal: f32,
    rotate_vertical: f32,
}

pub enum Direction {
    North,
    South,
    East,
    West
}

impl Direction {
    pub const ROT_NORTH: f32 = 180.0;
    pub const ROT_EAST: f32 = -90.0;
    pub const ROT_SOUTH: f32 = 0.0;
    pub const ROT_WEST: f32 = 90.0;

    pub fn as_string(&self) -> String {
        match &self {
            Direction::North => String::from("North"),
            Direction::South => String::from("South"),
            Direction::East => String::from("East"),
            Direction::West => String::from("West"),
        }
    }
}

impl CameraController {
    pub fn get_facing_dir(&self, camera: &Camera) -> Direction {
        let dir = cgmath::Deg::from(camera.yaw).0;

        if dir > -45.0 && dir < 45.0 {
            Direction::East
        } else if dir > -135.0 && dir < -45.0 {
            Direction::North
        } else if dir > -135.0 && dir < 135.0 {
            Direction::South
        } else {
            Direction::West
        }
    }
}

impl CameraController {
    pub const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;
    pub const PI: f32 = 3.1415927 + 0.0001;
    pub const GRAVITY: f32 = 2.0;

    pub fn new() -> Self {
        Self {
            velocity: Vector3::zero(),

            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, _world: &mut World, cam_sensitivity: f32, dt: f32) {
        self.update_rot(camera, cam_sensitivity, dt);
    }

    fn update_rot(&mut self, camera: &mut Camera, cam_sensitivity: f32, dt: f32) {
        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * cam_sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * cam_sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(CameraController::SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(CameraController::SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(CameraController::SAFE_FRAC_PI_2) {
            camera.pitch = Rad(CameraController::SAFE_FRAC_PI_2);
        }

        if camera.yaw < -Rad(CameraController::PI) {
            camera.yaw = Rad(CameraController::PI);
        } else if camera.yaw > Rad(CameraController::PI) {
            camera.yaw = -Rad(CameraController::PI)
        }
    }
}