use cgmath::{InnerSpace, Point3, Vector3, Zero};
use physics::box_collider::BoxCollider;
use renderer::camera::Camera;
use winit::event::VirtualKeyCode;
use world::World;

use crate::camera_controller::CameraController;

pub struct Player {
    //rotation: Vector3<f32>,
    velocity: Vector3<f32>, // temporary vel vector

    speed: f32,
    cam_sensitivity: f32,
    is_flying: bool,

    collider: BoxCollider, 

    //aabb: AABB,
    
    camera: Camera,
    camera_controller: CameraController,
}

const PLAYER_COLLIDER_SIZE: Vector3<f32> = Vector3::new(1., 2., 1.);
const GRAVITY: f32 = -2.;

impl Player {
    pub fn new(position: Vector3<f32>, /*rotation: Vector3<f32>,*/ cam_sensitivity: f32, speed: f32) -> Self {
        let camera = Camera::new(
            (position.x, position.y + 1.0, position.z),
            cgmath::Deg(-90.0),
            cgmath::Deg(-20.0)
        );

        Self {
            //rotation,
            velocity: Vector3::zero(),

            is_flying: false,
            cam_sensitivity,
            speed,

            collider: BoxCollider::new(position, PLAYER_COLLIDER_SIZE, Vector3::zero()),

            /*aabb: AABB::new(
                position,
                Vector3 { x: 0.5, y: 2.0, z: 0.5 },
                Vector3::zero(),
            ),*/
            
            camera,
            camera_controller: CameraController::new()
        }
    }

    //pub fn position(&self) -> Vector3<f32> { self.aabb.position }
}

impl Player {
    pub fn is_flying(&self) -> bool { self.is_flying }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_camera_controller(&self) -> &CameraController {
        &self.camera_controller
    } 

    pub fn get_camera_controller_mut(&mut self) -> &mut CameraController {
        &mut self.camera_controller
    } 

    pub fn process_keyboard(&mut self, input_manager: &renderer::input_manager::InputManager) -> bool {
        let amount = 1.0;

        let mut key_pressed = false;

        if input_manager.key_down(VirtualKeyCode::W) {
            self.velocity.z = amount;
            key_pressed = true;
        } else if input_manager.key_down(VirtualKeyCode::S) {
            self.velocity.z = -amount;
            key_pressed = true;
        } else {
            self.velocity.z = 0.0;
        }

        if input_manager.key_down(VirtualKeyCode::A) {
            self.velocity.x = -amount;
            key_pressed = true;
        } else if input_manager.key_down(VirtualKeyCode::D) {
            self.velocity.x = amount;
            key_pressed = true;
        } else {
            self.velocity.x = 0.0;
        }

        if !self.is_flying {
            if input_manager.key_just_pressed(VirtualKeyCode::Space) {
                self.velocity.y = amount * 1.25;
                
                key_pressed = true;
            } else {
                self.velocity.y = 0.;
            }
        } else {
            if input_manager.key_down(VirtualKeyCode::Space) {
                self.velocity.y = amount * 1.25;
                
                key_pressed = true;
            } else if input_manager.key_down(VirtualKeyCode::LShift) {
                if self.is_flying {
                    self.velocity.y = -(amount * 1.25);
                }
                
                key_pressed = true;
            } else {
                self.velocity.y = 0.0;
            }
        }

        if input_manager.key_just_pressed(VirtualKeyCode::P) {
            self.is_flying = !self.is_flying;
            
            key_pressed = true;
        }

        key_pressed
    }

    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        let mut camera = self.camera.clone();
        self.handle_movement(&camera, world, delta_time);

        // adjust camera
        let camera_position = Point3::new(self.collider.position.x, self.collider.position.y, self.collider.position.z);
                
        camera.position = camera_position;
        camera.position.y += 1.;

        self.camera = camera;

        self.camera_controller.update_camera(&mut self.camera, world, self.cam_sensitivity, delta_time);
    }

    // TODO: cleanup?
    fn handle_movement(&mut self, camera: &Camera, world: &mut World, delta_time: f32) {
        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        
        let mut velocity = Vector3::zero();
        velocity += forward * self.velocity.z * self.speed * delta_time;
        velocity += right * self.velocity.x * self.speed * delta_time;
        velocity += Vector3::unit_y() * self.velocity.y * self.speed * delta_time;

        self.collider.velocity = velocity;

        // do some collision detection
        if let Some(chunk) = world.get_chunk_from_world(&self.collider.position) {
            if let Some(collider) = chunk.has_collision(&self.collider) {
                let swept_result = self.collider.swept_aabb(collider);

                self.collider.position += self.collider.velocity * swept_result.time;
            } else {
                if !self.is_flying {
                    // no collision, assume we can fall
                    self.collider.velocity += Vector3::unit_y() * GRAVITY * delta_time;
                }

                self.collider.position += self.collider.velocity;
            }
        }
    }
}