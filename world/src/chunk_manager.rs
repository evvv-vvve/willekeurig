use std::{collections::HashMap, sync::{Arc, RwLock}};

use cgmath::{Vector3, Zero};

use crate::{chunk::{self, Chunk}, generator};

pub struct ChunkManager {
    /* -== CHUNK LISTS ==- */
    chunk_list: HashMap<Vector3<i32>, Chunk>,
    chunk_render_list: HashMap<Vector3<i32>, Chunk>,
    
    chunks_to_build: HashMap<Vector3<i32>, Chunk>,
    chunks_to_load: HashMap<Vector3<i32>, Chunk>,
    chunks_to_unload: HashMap<Vector3<i32>, Chunk>,

    cam_pos: Vector3<f32>,
    force_visibility_update: bool,
}

const ASYNC_NUM_CHUNKS_PER_FRAME: usize = 2;

impl ChunkManager { 
    pub fn new() -> Self {
        Self {
            chunk_list: HashMap::new(),
            chunk_render_list: HashMap::new(),

            chunks_to_build: HashMap::new(),
            chunks_to_load: HashMap::new(),
            chunks_to_unload: HashMap::new(),

            cam_pos: Vector3::zero(),
            force_visibility_update: false,
        }
    }
}

impl ChunkManager {
    pub fn get_renderable_chunks(&self) -> &HashMap<Vector3<i32>, Chunk> { &self.chunk_render_list }

    pub fn world_to_chunk_coords(&self, world_pos: &Vector3<f32>) -> Vector3<i32> {
        Vector3::new(
            abs_ceil(world_pos.x / chunk::CHUNK_SIZE as f32) as i32,
            abs_ceil(world_pos.y / chunk::CHUNK_SIZE as f32) as i32,
            abs_ceil(world_pos.z / chunk::CHUNK_SIZE as f32) as i32
        )
    }

    pub fn add_chunk(&mut self, new_chunk: Chunk) -> bool {
        for (_, chunk) in &self.chunk_render_list {
            if chunk.get_pos() == new_chunk.get_pos() {
                return false;
            }
        }

        for (_, chunk) in &self.chunks_to_load {
            if chunk.get_pos() == new_chunk.get_pos() {
                return false;
            }
        }

        self.chunks_to_load.entry(new_chunk.get_pos()).or_insert(new_chunk);

        true
    }

    pub fn get_chunk_from_world(&self, world_pos: &Vector3<f32>) -> Option<&Chunk> {
        let pos = self.world_to_chunk_coords(world_pos);

        self.get_chunk(pos)
    }

    pub fn get_chunk_from_world_mut(&mut self, world_pos: &Vector3<f32>) -> Option<&mut Chunk> {
        self.get_chunk_mut(self.world_to_chunk_coords(world_pos))
    }

    pub fn get_chunk(&self, chunk_pos: Vector3<i32>) -> Option<&Chunk> {
        self.chunk_render_list.get(&chunk_pos)
    }

    pub fn get_chunk_mut(&mut self, chunk_pos: Vector3<i32>) -> Option<&mut Chunk> {
        self.chunk_render_list.get_mut(&chunk_pos)
    }
}

impl ChunkManager {
    pub fn update(&mut self, device: Arc<RwLock<wgpu::Device>>, player_pos: Vector3<f32>, seed: u32) {
        self.load_chunks();
        self.build_chunks(device, seed);
        // TODO: logic for detecting what chunks are visible
    }

    fn load_chunks(&mut self) {
        if self.chunks_to_load.len() < 1 {
            return;
        }

        let mut chunks_to_load = if self.chunks_to_load.len() < ASYNC_NUM_CHUNKS_PER_FRAME {
            self.chunks_to_load.len() - 1
        } else {
            ASYNC_NUM_CHUNKS_PER_FRAME
        };

        while chunks_to_load > 0 {
            let key = match self.chunks_to_load.keys().next() {
                Some(&chunk_key) => chunk_key,
                None => { break; }
            };

            let chunk = self.chunks_to_load.remove(&key);

            if let Some(chunk) = chunk {
                self.chunks_to_build.entry(key).or_insert(chunk);
            }

            chunks_to_load -= 1;
        }
    }

    fn build_chunks(&mut self, device: Arc<RwLock<wgpu::Device>>, seed: u32) {
        if self.chunks_to_build.len() < 1 {
            return;
        }
        
        let length = if self.chunks_to_build.len() < ASYNC_NUM_CHUNKS_PER_FRAME {
            self.chunks_to_build.len() - 1
        } else {
            ASYNC_NUM_CHUNKS_PER_FRAME
        };

        let mut nonempty_chunks = 0;
        let mut keys = Vec::new();

        for (chunk_key, chunk) in self.chunks_to_build.iter_mut() {
            if nonempty_chunks >= length {
                break;
            }

            if chunk.is_first_build() {
                let height_map = generator::gen_height_map(seed, chunk.get_pos());

                generator::gen_smooth_terrain(chunk, &height_map);
            }
            
            chunk.build_mesh(&device.clone().read().unwrap());

            if !chunk.is_empty() {
                nonempty_chunks += 1;
            }

            keys.push(*chunk_key);
        }

        keys.reverse();

        for key in keys {
            let chunk = self.chunks_to_build.remove(&key);

            if let Some(chunk) = chunk {
                self.chunk_render_list.entry(key).or_insert(chunk);
            }
        }
    } 
}

fn abs_ceil(f: f32) -> f32 {
    if f >= 0.0 {
        f
    } else {
        f.floor()
    }
}