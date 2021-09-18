use std::{collections::HashMap, sync::{Arc, RwLock}};

use cgmath::Vector3;
use chunk::Chunk;
use wgpu::Device;

use self::chunk_manager::ChunkManager;

/*  -== MODULES START ==-  */

pub mod generator;
pub mod chunk;
pub mod chunk_manager;
pub mod block_culling;
pub mod transform;

/*  -== MODULES END ==-  */

pub struct World {
    chunk_manager: ChunkManager,
    seed: u32,

    player_last_chunk: Vector3<i32>,
    
    //spawn_pos: Vector3<i32>,
    render_distance: usize,
}

pub const MAP_W: usize = 256;
pub const MAP_H: usize = 64;

impl World {
    pub fn new(seed: u32, render_distance: usize) -> Self {
        //let mut rng = StdRng::seed_from_u64(seed as u64);

        //let x = rng.gen_range(0..255);
        //let z = rng.gen_range(0..255);

        Self {
            chunk_manager: ChunkManager::new(),
            seed,

            player_last_chunk: Vector3::new(0, 0, 0),
            //spawn_pos: Vector3::new(x, 0, z),
            render_distance,
        }
    }

    pub fn create_or_destroy_chunks(&mut self, player_pos: &Vector3<f32>) {
        //println!("[create_destroy_chunks] test");
        
        let dist_min = -(self.render_distance as i32);
        let dist_max = self.render_distance as i32;

        let player_chunk = self.chunk_manager.world_to_chunk_coords(player_pos);
        
        if player_chunk != self.player_last_chunk {
            for c_z in dist_min..dist_max {
                for c_x in dist_min..dist_max {
                    for c_y in dist_min..dist_max {
                        let chunk_pos = Vector3::new(
                            c_x + player_chunk.x,
                            c_y + player_chunk.y,
                            c_z + player_chunk.z
                        );

                        self.chunk_manager.add_chunk(Chunk::new(chunk_pos));
                    }
                }
            }
        }

        self.player_last_chunk = player_chunk;
    }

    /*pub fn get_chunks_loading(&self) -> usize {
        self.chunk_manager.get_chunks_loading()
    }

    pub fn get_max_chunks(&self) -> usize {
        self.chunk_manager.get_max_chunks()
    }

    pub fn load_chunks(&mut self, device: Arc<RwLock<Device>>) {
        //println!("[build_chunk_meshes] testset");

        self.chunk_manager.load_chunks(device, self.seed).unwrap();
    }

    pub fn fetch_chunks(&mut self) {
        self.chunk_manager.receive_chunks();
    }
    
    pub fn update_chunks(&mut self, player_pos: &Vector3<f32>, device: Arc<RwLock<Device>>) {
        self.chunk_manager.update(device, player_pos, self.render_distance);
    }*/

    pub fn update(&mut self, device: Arc<RwLock<Device>>, player_pos: Vector3<f32>) {
        self.chunk_manager.update(device, player_pos, self.seed);
    }
}

// getters
impl World {
    /*pub fn get_chunks(&self) -> &HashMap<Vector3<i32>, Chunk> {
        &self.chunk_manager.chunks
    }*/

    pub fn get_renderable_chunks(&self) -> &HashMap<Vector3<i32>, Chunk> {
        self.chunk_manager.get_renderable_chunks()
    }

    pub fn get_chunk_from_world(&self, world_pos: &Vector3<f32>) -> Option<&Chunk> {
        self.chunk_manager.get_chunk_from_world(world_pos)
    }

    pub fn get_chunk_from_world_mut(&mut self, world_pos: &Vector3<f32>) -> Option<&mut Chunk> {
        self.chunk_manager.get_chunk_from_world_mut(world_pos)
    }

    /*pub fn get_chunk_queue_count(&self) -> usize {
        self.chunk_manager.get_chunk_queue_count()
    }*/
}