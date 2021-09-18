use std::collections::HashMap;

use cgmath::{Vector3, Zero};
use physics::box_collider::BoxCollider;
use wgpu::util::DeviceExt;

use renderer::vertex::Vertex;
use common::{block::Block, identifier::Identifier, registry::Registry};

use crate::{World, block_culling::{cull_neighbors, CullCode}};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_BIT_SIZE: usize = 4;

pub const BLOCK_Y_SHIFT: usize = 4;
pub const BLOCK_Z_SHIFT: usize = 8;

pub struct Chunk {
    is_first_build: bool,
    is_dirty: bool,
    is_surrounded: bool,

    chunk_data: ChunkData,

    //chunk_aabb_tree: Vec<Option<Aabb3<f32>>>,

    chunk_neighbors: Vec<Vector3<i32>>,
    //chunk_colliders: Vec<Aabb3<f32>>,
    chunk_colliders: Vec<BoxCollider>,

    chunk_verticies: Vec<Vertex>,
    chunk_indicies: Vec<u32>,
    
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
} 

impl Chunk {
    pub fn new(pos: Vector3<i32>) -> Self {
        
        //let a = dbvt::DynamicBoundingVolumeTree::<Aabb3<f32>>::new();

        //let b = Aabb3::new(Point3 { x: 0f32, y: 0f32, z: 0f32 }, Point3 { x: 1f32, y: 1f32, z: 1f32 });
        //a.insert(b);

        Self {
            is_first_build: true,
            is_dirty: true,
            is_surrounded: false,
            chunk_data: ChunkData::new(pos),

            //chunk_aabb_tree: vec![None; CHUNK_SIZE.pow(3)],
            chunk_colliders: Vec::new(),

            chunk_neighbors: vec![
                Vector3 { x: pos.x, y: pos.y + 1, z: pos.z }, // Up     0
                Vector3 { x: pos.x, y: pos.y - 1, z: pos.z }, // Down   1
                Vector3 { x: pos.x - 1, y: pos.y, z: pos.z }, // Left   2
                Vector3 { x: pos.x + 1, y: pos.y, z: pos.z }, // Right  3
                Vector3 { x: pos.x, y: pos.y, z: pos.z + 1 }, // Front  4
                Vector3 { x: pos.x, y: pos.y, z: pos.z - 1 }, // Back   5
            ],

            chunk_verticies: Vec::new(),
            chunk_indicies: Vec::new(),
            
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    // TODO: take into account player position
    pub fn is_visible(&self, world: &World) -> bool {
        let mut count = 0;

        for chunk_neighbor_pos in &self.chunk_neighbors {
            if let Some(chunk) = world.chunk_manager.get_chunk(*chunk_neighbor_pos) {
                if !chunk.is_empty() {
                    count += 1;
                }
            }
        }

        count < 6
    }

    pub fn is_first_build(&self) -> bool { self.is_first_build }

    pub fn is_empty(&self) -> bool { self.chunk_data.is_empty }

    pub fn build_mesh(&mut self, device: &wgpu::Device) {
        if !self.chunk_data.is_empty {
            let (verts, indies) = self.chunk_data.build_mesh();

            self.chunk_verticies = verts;
            self.chunk_indicies = indies;

            self.vertex_buffer = Some(
                device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(self.chunk_verticies.as_slice()),
                        usage: wgpu::BufferUsages::VERTEX
                    }
                )
            );
        
            self.index_buffer = Some(
                device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(self.chunk_indicies.as_slice()),
                        usage: wgpu::BufferUsages::INDEX
                    }
                )
            );

            let colliders = self.chunk_data.gen_collision_mesh();

            self.set_collision_mesh(colliders);
        }

        self.is_dirty = false;
    }

    fn set_collision_mesh(&mut self, box_data: HashMap<Vector3<usize>, Vector3<usize>>) {
        

        let mut collider_index = 0;
        let existing_collider_count = self.chunk_colliders.len();

        for (collider_pos, collider_size) in box_data {
            let collider_pos = Vector3::new(collider_pos.x as f32, collider_pos.y as f32, collider_pos.z as f32);
            let collider_size = Vector3::new(collider_size.x as f32, collider_size.y as f32, collider_size.z as f32);
            let position = self.chunk_data.to_world_pos_f32(collider_pos);

            if collider_index < existing_collider_count {
                self.chunk_colliders[collider_index].position = collider_pos;
                self.chunk_colliders[collider_index].size = collider_size;
            } else {
                self.chunk_colliders.push(BoxCollider::new(position, collider_size, Vector3::zero()));
            }
            
            collider_index += 1;
        }

        /*for (collider_start, collider_size) in box_data {
            let collider_end = collider_start + collider_size;

            let min = Point3 {
                x: collider_start.x as f32,
                y: collider_start.y as f32,
                z: collider_start.z as f32,
            };

            let max = Point3 {
                x: collider_end.x as f32,
                y: collider_end.y as f32,
                z: collider_end.z as f32,
            };

            if collider_index < existing_collider_count {
                self.chunk_colliders[collider_index] = Aabb3::new(min, max);
            } else {
                self.chunk_colliders.push(Aabb3::new(min, max));
            }
            
            collider_index += 1;
        }*/

        // delete all unused boxes if this mesh generation had less boxes than the previous one.
        if collider_index < existing_collider_count {
            for index in collider_index..existing_collider_count - 1 {
                self.chunk_colliders.remove(index);
            }
        }
    }

    pub fn has_collision(&self, aabb: &BoxCollider) -> Option<&BoxCollider> {
        for chunk_collider in &self.chunk_colliders {
            if aabb.collision_with(chunk_collider) {
                return Some(chunk_collider);
            }

            // TODO: return the block!
            /*if aabb.contains(chunk_aabb) {
                return true;
            }

            if let Some(swept_aabb) = aabb.swept_collision_with(block_aabb) {
                if let Some(block) = self.get_block(aabb.position.x as usize, aabb.position.y as usize, aabb.position.z as usize) {
                    return Some((
                        swept_aabb,
                        block
                    ));
                }
            }*/
        //}
        }

        None
    }

    pub fn is_dirty(&self) -> bool { self.is_dirty }

    pub fn get_pos(&self) -> Vector3<i32> { self.chunk_data.pos }

    pub fn add_block_from_world_pos(&mut self, world_x: f32, world_y: f32, world_z: f32, block: Option<Block>) -> bool {
        let local = self.chunk_data.world_to_local_pos(world_x, world_y, world_z);
        
        self.add_block(local.x, local.y, local.z, block)
    }

    pub fn add_block(&mut self, x: usize, y: usize, z: usize, block: Option<Block>) -> bool {
        if self.chunk_data.add_block(x, y, z, block) {
            self.is_dirty = true;

            true
        } else { false }
    }

    pub fn remove_block(&mut self, x: usize, y: usize, z: usize) -> bool {
        if self.chunk_data.remove_block(x, y, z) {
            self.is_dirty = true;

            true
        } else { false }
    }

    pub fn get_block_from_world_pos(&self, world_x: f32, world_y: f32, world_z: f32) -> Option<Block> {
        self.chunk_data.get_block_from_world_pos(world_x, world_y, world_z)
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        self.chunk_data.get_block(x, y, z)
    }

    pub fn get_buffers(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, u32)> {
        if let Some(vertex_buffer) = &self.vertex_buffer {
            if let Some(index_buffer) = &self.index_buffer {
                return Some((vertex_buffer, index_buffer, self.chunk_indicies.len() as u32))
            }
        }

        None
    }

    pub fn local_to_world_pos(&self, x: usize, y: usize, z: usize) -> Vector3<f32> {
        self.chunk_data.local_to_world_pos(x, y, z)
    }

    pub fn dispose(&mut self) {
        let _ = self.index_buffer.take();
        let _ = self.vertex_buffer.take();
        self.chunk_data.blocks.clear();
    }
}

#[derive(Debug, Clone)]
pub struct ChunkData {
    is_empty: bool,
    pos: Vector3<i32>,
    blocks: Vec<Option<Identifier>>,
    air_blocks: usize,
}

impl ChunkData {
    pub fn new(pos: Vector3<i32>) -> Self {
        Self {
            is_empty: true,
            pos,
            blocks: vec![None; CHUNK_SIZE.pow(3)],
            air_blocks: 0,
        }
    }

    pub fn get_blocks(&self) -> Vec<Option<Identifier>> {
        self.blocks.clone()
    }

    pub fn add_block(&mut self, x: usize, y: usize, z: usize, block: Option<Block>) -> bool {
        let index = pos_as_index(x, y, z);

        if index < self.blocks.len() && index > 0 {
            if let Some(_) = &block {
                self.is_empty = false;

                if self.air_blocks > 0 {
                    self.air_blocks -= 1;
                }
            } else {
                // only increment if we're replacing a block
                if let Some(_) = self.get_block(x, y, z) {
                    self.air_blocks += 1;
                }
            }

            self.is_empty = self.air_blocks > 0;

            if let Some(data) = block {
                self.blocks[index] = Some(data.get_identifier().clone());
            } else {
                self.blocks[index] = None;
            }

            true
        } else {
            false
        }
    }

    pub fn remove_block(&mut self, x: usize, y: usize, z: usize) -> bool {
        self.add_block(x, y, z, None)
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        let index = pos_as_index(x, y, z);

        if index > 0 && index < self.blocks.len() {
            let block_id = &self.blocks[index];

            if let Some(block_id) = block_id {
                return Registry::current().get_block(&block_id);
            }
        }

        None
    }

    pub fn get_block_from_world_pos(&self, world_x: f32, world_y: f32, world_z: f32) -> Option<Block> {
        let local = self.world_to_local_pos(world_x, world_y, world_z);

        self.get_block(local.x, local.y, local.z)
    }

    pub fn local_to_world_pos(&self, x: usize, y: usize, z: usize) -> Vector3<f32> {
        Vector3::new(
            self.pos.x as f32 * CHUNK_SIZE as f32 + x as f32,
            self.pos.y as f32 * CHUNK_SIZE as f32 + y as f32,
            self.pos.z as f32 * CHUNK_SIZE as f32 + z as f32
        )
    }

    pub fn to_world_pos_f32(&self, vec: Vector3<f32>) -> Vector3<f32> {
        Vector3::new(
            self.pos.x as f32 * CHUNK_SIZE as f32 + vec.x as f32,
            self.pos.y as f32 * CHUNK_SIZE as f32 + vec.y as f32,
            self.pos.z as f32 * CHUNK_SIZE as f32 + vec.z as f32
        )
    }

    pub fn world_to_local_pos(&self, x: f32, y: f32, z: f32) -> Vector3<usize> {
        cgmath::Vector3::new(
            (x - self.pos.x as f32 * CHUNK_SIZE as f32) as usize,
            (y - self.pos.y as f32 * CHUNK_SIZE as f32) as usize,
            (z - self.pos.z as f32 * CHUNK_SIZE as f32) as usize
        )
    }

    pub fn build_mesh(&mut self) -> (Vec<Vertex>, Vec<u32>) {
        if self.is_empty {
            return (Vec::new(), Vec::new())
        }

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indicies: Vec<u32> = Vec::new();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if let Some(block) = self.get_block(x, y, z) {
                        let block_pos = self.local_to_world_pos(x, y, z);

                        let cull_code = cull_neighbors(&self, x, y, z);
    
                        if (cull_code & (CullCode::F as u8)) == CullCode::F as u8 {
                            let (verts, indies) = self.build_face(
                                vertices, indicies, block.get_vert_front(),
                                &block_pos
                            );
    
                            vertices = verts;
                            indicies = indies;
                        }
    
                        if (cull_code & (CullCode::B as u8)) == CullCode::B as u8 {
                            let (verts, indies) = self.build_face(
                                vertices, indicies, block.get_vert_back(),
                                &block_pos
                            );
    
                            vertices = verts;
                            indicies = indies;
                        }
    
                        if (cull_code & (CullCode::U as u8)) == CullCode::U as u8 {
                            let (verts, indies) = self.build_face(
                                vertices, indicies, block.get_vert_top(),
                                &block_pos
                            );
    
                            vertices = verts;
                            indicies = indies;
                        }
    
                        if (cull_code & (CullCode::D as u8)) == CullCode::D as u8 {
                            let (verts, indies) = self.build_face(
                                vertices, indicies, block.get_vert_bottom(),
                                &block_pos
                            );
    
                            vertices = verts;
                            indicies = indies;
                        }
    
                        if (cull_code & (CullCode::L as u8)) == CullCode::L as u8 {
                            let (verts, indies) = self.build_face(
                                vertices, indicies, block.get_vert_left(),
                                &block_pos
                            );
    
                            vertices = verts;
                            indicies = indies;
                        }
    
                        if (cull_code & (CullCode::R as u8)) == CullCode::R as u8 {
                            let (verts, indies) = self.build_face(
                                vertices, indicies, block.get_vert_right(),
                                &block_pos
                            );
    
                            vertices = verts;
                            indicies = indies;
                        }
                    }
                }
            }
        }
        
        (vertices, indicies)
    }

    pub fn gen_collision_mesh(&self) -> HashMap<Vector3<usize>, Vector3<usize>> {
        let mut tested: Vec<bool> = [false; CHUNK_SIZE.pow(3)].into();
        let mut boxes: HashMap<Vector3<usize>, Vector3<usize>> = HashMap::new();

        for index in 0..tested.len() {
            if !tested[index] {
                tested[index] = true;

                if let Some(opt_block_id) = self.blocks.get(index) {
                    if let Some(_) = opt_block_id { // if the block contributes to the mesh
                        let box_start = index_as_pos(index);
                        let mut box_size = Vector3::new(1usize, 1, 1);

                        let mut can_spread_x = true;
                        let mut can_spread_y = true;
                        let mut can_spread_z = true;

                        //Attempts to expand in all directions and stops in each direction when it no longer can.
                        while can_spread_x || can_spread_y || can_spread_z {
                            can_spread_x = self.try_spread_x(can_spread_x, &mut tested, box_start, &mut box_size);
                            can_spread_y = self.try_spread_y(can_spread_y, &mut tested, box_start, &mut box_size);
                            can_spread_z = self.try_spread_z(can_spread_z, &mut tested, box_start, &mut box_size);
                        }

                        boxes.insert(box_start, box_size);
                    }
                }
            }
        }

        boxes
    }

    // returns whether the box can continue to spread along the positive x axis or not
    fn try_spread_x(&self, mut can_spread_x: bool, tested: &mut Vec<bool>, box_start: Vector3<usize>, box_size: &mut Vector3<usize>) -> bool {
        //Checks the square made by the Y and Z size on the X index one larger than the size of the
        //box
        let y_limit = box_start.y + box_size.y;
        let z_limit = box_start.z + box_size.z;

        for y in box_start.y..y_limit {
            if !can_spread_x { break; }

            for z in box_start.z..z_limit {
                let new_x = box_start.x + box_size.x;
                let new_index = pos_as_index(new_x, y, z);

                let block_contributes_to_mesh = match self.blocks.get(new_index) {
                    Some(opt_block_id) => {
                        if let Some(_) = opt_block_id {
                            true
                        } else {
                            false
                        }
                    },
                    None => false
                };

                if new_x >= CHUNK_SIZE || tested[new_index] || !block_contributes_to_mesh {
                    can_spread_x = false;
                }
            } 
        }

        //If the box can spread, mark it as tested and increase the box size in the X dimension.
        if can_spread_x {
            for y in box_start.y..y_limit {
                for z in box_start.z..z_limit {
                    let new_x = box_start.x + box_size.x;
                    let new_index = pos_as_index(new_x, y, z);
                    
                    tested[new_index] = true;
                }
            }

            box_size.x += 1;
        }
        
        can_spread_x
    }

    // returns whether the box can continue to spread along the positive y axis or not
    fn try_spread_y(&self, mut can_spread_y: bool, tested: &mut Vec<bool>, box_start: Vector3<usize>, box_size: &mut Vector3<usize>) -> bool {
        //Checks the square made by the Y and Z size on the X index one larger than the size of the
        //box
        let x_limit = box_start.x + box_size.x;
        let z_limit = box_start.z + box_size.z;

        for x in box_start.x..x_limit {
            if !can_spread_y { break; }

            for z in box_start.z..z_limit {
                let new_y = box_start.y + box_size.y;
                let new_index = pos_as_index(x, new_y, z);

                let block_contributes_to_mesh = match self.blocks.get(new_index) {
                    Some(opt_block_id) => {
                        if let Some(_) = opt_block_id {
                            true
                        } else {
                            false
                        }
                    },
                    None => false
                };

                if new_y >= CHUNK_SIZE || tested[new_index] || !block_contributes_to_mesh {
                    can_spread_y = false;
                }
            } 
        }

        //If the box can spread, mark it as tested and increase the box size in the X dimension.
        if can_spread_y {
            for x in box_start.x..x_limit {
                for z in box_start.z..z_limit {
                    let new_y = box_start.y + box_size.y;
                    let new_index = pos_as_index(x, new_y, z);
                    
                    tested[new_index] = true;
                }
            }

            box_size.y += 1;
        }
        
        can_spread_y
    }

    // returns whether the box can continue to spread along the positive z axis or not
    fn try_spread_z(&self, mut can_spread_z: bool, tested: &mut Vec<bool>, box_start: Vector3<usize>, box_size: &mut Vector3<usize>) -> bool {
        //Checks the square made by the Y and Z size on the X index one larger than the size of the
        //box
        let x_limit = box_start.x + box_size.x;
        let y_limit = box_start.y + box_size.y;

        for x in box_start.x..x_limit {
            if !can_spread_z { break; }

            for y in box_start.y..y_limit {
                let new_z = box_start.z + box_size.z;
                let new_index = pos_as_index(x, y, new_z);

                let block_contributes_to_mesh = match self.blocks.get(new_index) {
                    Some(opt_block_id) => {
                        if let Some(_) = opt_block_id {
                            true
                        } else {
                            false
                        }
                    },
                    None => false
                };

                if new_z >= CHUNK_SIZE || tested[new_index] || !block_contributes_to_mesh {
                    can_spread_z = false;
                }
            } 
        }

        //If the box can spread, mark it as tested and increase the box size in the X dimension.
        if can_spread_z {
            for x in box_start.x..x_limit {
                for y in box_start.y..y_limit {
                    let new_z = box_start.z + box_size.z;
                    let new_index = pos_as_index(x, y, new_z);
                    
                    tested[new_index] = true;
                }
            }

            box_size.z += 1;
        }
        
        can_spread_z
    }

    fn build_face(&self, mut vertices: Vec<Vertex>, mut indicies: Vec<u32>, block_verts: Vec<Vertex>,
        block_position: &Vector3<f32>) -> (Vec<Vertex>, Vec<u32>) {
        let block_indicies: Vec<u32> = vec![
            0, 1, 2, // triangle 1
            2, 3, 0, // triangle 2
        ];

        let index = vertices.len() as u32;
        
        for vert in block_verts {
            let vert_pos = [
                block_position.x + vert.position[0],
                block_position.y + vert.position[1],
                block_position.z + vert.position[2]
            ];
            
            vertices.push(Vertex { position: vert_pos, tex_coords: vert.tex_coords });
        }

        for f_index in &block_indicies {
            indicies.push(f_index.clone() + index);
        }

        (vertices, indicies)
    }
}

pub fn pos_as_index(local_x: usize, local_y: usize, local_z: usize) -> usize {
    //local_x + local_y * CHUNK_SIZE + local_z * CHUNK_SIZE * CHUNK_SIZE
    local_x | local_y << BLOCK_Y_SHIFT | local_z << BLOCK_Z_SHIFT
}

pub fn index_as_pos(index: usize) -> Vector3<usize> {
    let block_x = index & 0xF;
    let block_y = (index >> BLOCK_Y_SHIFT) & 0xF;
    let block_z = (index >> BLOCK_Z_SHIFT) & 0xF;

    Vector3::new(block_x, block_y, block_z)
}