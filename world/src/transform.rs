use cgmath::Vector3;

use crate::chunk;

pub struct Transform {
    chunk_pos: Vector3<i32>,
    local_pos: Vector3<f32>,
}

impl Transform {
    pub fn change_chunk(&mut self) {
        // update x
        if self.local_pos.x > chunk::CHUNK_SIZE as f32 {
            self.chunk_pos.x += 1;
        } else if self.local_pos.x < 0. {
            self.chunk_pos.x -= 1;
        }

        // update y
        if self.local_pos.y > chunk::CHUNK_SIZE as f32 {
            self.chunk_pos.y += 1;
        } else if self.local_pos.y < 0. {
            self.chunk_pos.y -= 1;
        }

        // update z
        if self.local_pos.z > chunk::CHUNK_SIZE as f32 {
            self.chunk_pos.z += 1;
        } else if self.local_pos.z < 0. {
            self.chunk_pos.z -= 1;
        }
        
        // set pos back to 0
        self.local_pos = Vector3 {
            x: floor_mod(self.local_pos.x, chunk::CHUNK_SIZE as f32),
            y: floor_mod(self.local_pos.y, chunk::CHUNK_SIZE as f32),
            z: floor_mod(self.local_pos.z, chunk::CHUNK_SIZE as f32),
        };
    }
}

fn floor_mod(a: f32, b: f32) -> f32 {
    (a % b + b) % b
}