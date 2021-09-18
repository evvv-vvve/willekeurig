/*use std::{collections::HashMap, io::Error, sync::{Arc, RwLock, mpsc::{Receiver, Sender, channel}}};

use cgmath::Vector3;
use rand::prelude::SliceRandom;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::{chunk::{self, Chunk}, generator};

pub struct ChunkManager {
    pub chunks: HashMap<Vector3<i32>, Chunk>,
    
    chunks_to_load: Vec<Vector3<i32>>,
    chunks_loading: Vec<Vector3<i32>>,

    chunk_sender: Option<Sender<(Chunk, Vector3<i32>)>>,
    chunk_receiver: Option<Receiver<(Chunk, Vector3<i32>)>>,
}

impl ChunkManager {
    pub const CHUNK_GEN_MAX: usize = 10;

    pub fn new() -> Self {
        ChunkManager {
            chunks: HashMap::new(),
            chunks_to_load: Vec::new(),
            chunks_loading: Vec::new(),

            chunk_sender: None,
            chunk_receiver: None,
        }
    }

    pub fn get_chunk_queue_count(&self) -> usize {
        self.chunks_to_load.len()
    }

    pub fn world_to_chunk_coords(&self, world_pos: &Vector3<f32>) -> Vector3<i32> {
        Vector3::new(
            abs_ceil(world_pos.x / chunk::CHUNK_SIZE as f32) as i32,
            abs_ceil(world_pos.y / chunk::CHUNK_SIZE as f32) as i32,
            abs_ceil(world_pos.z / chunk::CHUNK_SIZE as f32) as i32
        )
    }

    pub fn add_chunk(&mut self, chunk_pos: Vector3<i32>) -> bool {
        if self.chunks_to_load.iter().any(|&pos| pos == chunk_pos) {
            return false;
        }

        if self.chunks.contains_key(&chunk_pos) {
            return false;
        }

        self.chunks_to_load.push(chunk_pos);
        true
    }

    pub fn get_chunk_from_world(&self, world_pos: &Vector3<f32>) -> Option<&Chunk> {
        let pos = self.world_to_chunk_coords(world_pos);

        self.get_chunk(&pos)
    }

    pub fn get_chunk_from_world_mut(&mut self, world_pos: &Vector3<f32>) -> Option<&mut Chunk> {
        self.get_chunk_mut(&self.world_to_chunk_coords(world_pos))
    }

    pub fn get_chunk(&self, chunk_pos: &Vector3<i32>) -> Option<&Chunk> {
        self.chunks.get(chunk_pos)
    }

    pub fn get_chunk_mut(&mut self, chunk_pos: &Vector3<i32>) -> Option<&mut Chunk> {
        self.chunks.get_mut(chunk_pos)
    }

    pub fn rem_chunk(&mut self, chunk_pos: &Vector3<i32>) -> bool {
        let had_chunk = if let Some(mut chunk) = self.chunks.remove(chunk_pos) {
            chunk.dispose();
            true
        } else {
            false
        };

        let was_queued = if let Some(index) = self.chunks_to_load.iter().position(|pos| *pos == *chunk_pos) {
            self.chunks_to_load.remove(index);
            true
        } else {
            false
        };

        had_chunk || was_queued
    }

    /// Receive built chunks from the chunk_receiver
    pub fn receive_chunks(&mut self) {
        if let Some(channel) = &self.chunk_receiver {
            if let Ok((chunk, chunk_pos)) = channel.try_recv() {
                if let Some(index) = self.chunks_to_load.iter().position(|pos| *pos == chunk_pos) {
                    self.chunks_to_load.remove(index);
                }

                self.chunks.entry(chunk_pos).or_insert(chunk);

                if let Some(index) = self.chunks_loading.iter().position(|pos| *pos == chunk_pos) {
                    self.chunks_loading.remove(index);
                }
            }
        }
    }
    
    pub fn update(&mut self, device: Arc<RwLock<wgpu::Device>>, player_pos: &Vector3<f32>, draw_distance: usize) {
        let player_chunk = self.world_to_chunk_coords(player_pos);

        let mut old_chunks = Vec::new();

        {
            for (chunk_pos, chunk) in &mut self.chunks {
                if chunk_pos.x < player_chunk.x - draw_distance as i32 || chunk_pos.x > player_chunk.x + draw_distance as i32 ||
                   chunk_pos.y < player_chunk.y - draw_distance as i32 || chunk_pos.y > player_chunk.y + draw_distance as i32 ||
                   chunk_pos.z < player_chunk.z - draw_distance as i32 || chunk_pos.z > player_chunk.z + draw_distance as i32 {
                       old_chunks.push(chunk_pos.clone());
                       continue;
                }
    
                if chunk.is_dirty() {
                    chunk.build_mesh(&device.clone().read().unwrap());
                }
            }
        }

        for chunk_pos in old_chunks {
            self.rem_chunk(&chunk_pos);
        }
    }

    pub fn shuffle_load(&mut self) {
        self.chunks_to_load.shuffle(&mut rand::thread_rng());
    }

    pub fn get_chunks_loading(&self) -> usize {
        self.chunks_loading.len()
    }

    pub fn get_max_chunks(&self) -> usize {
        Self::CHUNK_GEN_MAX
    }

    pub fn load_chunks(&mut self, device: Arc<RwLock<wgpu::Device>>, seed: u32) -> Result<(), Error> {
        //println!("{}", self.chunks_to_load.len());
        
        // if theres nothing to load, or we're currently at our chunkgen limit, return
        if self.chunks_to_load.len() < 1 || self.get_chunks_loading() > Self::CHUNK_GEN_MAX - 1 {
            return Ok(());
        }

        let mut chunk_count = if self.chunks_to_load.len() < Self::CHUNK_GEN_MAX {
            self.chunks_to_load.len()
        } else {
            Self::CHUNK_GEN_MAX - self.get_chunks_loading()
        };

        if self.get_chunks_loading() + chunk_count > Self::CHUNK_GEN_MAX {
            let sub = (self.get_chunks_loading() + chunk_count) - Self::CHUNK_GEN_MAX;

            chunk_count -= sub;
        }

        // make the chunk loading appear less uniform
        //self.shuffle_load();

        /*let chunks_to_load = (0..chunk_count).map(|index| {
            self.chunks_to_load[index]
        }).collect::<Vec<_>>();
        
        for index in 0..chunk_count {
            let chunk_pos = chunks_to_load[index];
            
            if self.chunks_loading.contains(&chunk_pos) {
                if let Some(index) = self.chunks_to_load.iter().position(|pos| *pos == chunk_pos) {
                    self.chunks_to_load.remove(index);
                }
                
                continue;
            }

            // Receiver and Sender should both have the same value (Some or None)
            if let None = self.chunk_receiver {
                let (sender, receiver) = mpsc::channel::<(Chunk, Vector3<i32>)>();

                self.chunk_sender = Some(sender);
                self.chunk_receiver = Some(receiver);
            }

            if let Some(sender) = &self.chunk_sender {
                build_chunk(device.clone(), seed, chunk_pos, sender.clone());

                self.chunks_loading.push(chunk_pos);
            }
        }*/

        if let None = self.chunk_receiver {
            let (sender, receiver) = channel();

            self.chunk_sender = Some(sender);
            self.chunk_receiver = Some(receiver);
        }

        if let Some(sender_ref) = &self.chunk_sender {
            let sender = sender_ref.clone();

            let built = (0..chunk_count)
                .map(|index| {
                    let chunk_pos = self.chunks_to_load[index];

                    if self.chunks_loading.contains(&chunk_pos) {
                        if let Some(index) = self.chunks_to_load.iter().position(|pos| *pos == chunk_pos) {
                            self.chunks_to_load.remove(index);
                        }

                        None 
                    } else {
                        Some(chunk_pos)
                    }
                })
                .collect::<Vec<_>>()
                .into_par_iter()
                .map_with(sender, | s, chunk_pos_option| {
                    if let Some(chunk_pos) = chunk_pos_option {
                        let built_chunk = build_chunk(device.clone(), seed, chunk_pos);
                        
                        if let Err(e) = s.send(built_chunk) {
                            eprintln!("Could not send built chunk: {}", e);
                            None
                        } else {
                            Some(chunk_pos)
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            
            for chunk_pos_opt in built {
                if let Some(chunk_pos) = chunk_pos_opt {
                    self.chunks_loading.push(chunk_pos);
                }
            }
        }

        Ok(())
    }
}

fn build_chunk(device: Arc<RwLock<wgpu::Device>>, seed: u32, chunk_pos: Vector3<i32>) -> (Chunk, Vector3<i32>) {
    let mut chunk = Chunk::new(chunk_pos);

    let height_map = generator::gen_height_map(seed, chunk_pos);

    generator::gen_smooth_terrain(&mut chunk, &height_map);
        
    chunk.build_mesh(&device.read().unwrap());

    (chunk, chunk_pos)
}

fn abs_ceil(f: f32) -> f32 {
    if f >= 0.0 {
        f
    } else {
        f.floor()
    }
}*/