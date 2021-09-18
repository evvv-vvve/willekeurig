use super::chunk;
use noise::{OpenSimplex, Seedable, NoiseFn};
use common::{registry::Registry, identifier::Identifier};

pub struct TerrainType {
    pub name: String,
    pub height: f64,
    pub color: Color
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

/*pub fn generate(map_w: usize, map_h: usize, seed: u32, scale: f64, octaves: u32,
    persistance: f64, lacunarity: f64, offset: cgmath::Vector2<f64>, position: cgmath::Vector2<f64>) -> Self {
          
        Self {
            map_w,
            map_h,
            seed,
            scale,
            octaves,
            persistance,
            lacunarity,
            offset,
            position,

            noise_map: Self::generate_noise_map(map_w, map_h, seed, scale,
            octaves, persistance, lacunarity, offset, position),
            regions: vec![
                TerrainType {
                    name: String::from("Water Deep"),
                    color: Color::new(39, 50, 107),
                    height: 0.3
                },
                TerrainType {
                    name: String::from("Water Shallow"),
                    color: Color::new(49, 71, 178),
                    height: 0.4
                },
                TerrainType {
                    name: String::from("Sand"),
                    color: Color::new(224, 229, 149),
                    height: 0.45
                },
                TerrainType {
                    name: String::from("Grass"),
                    color: Color::new(89, 201, 52),
                    height: 0.5
                },
                TerrainType {
                    name: String::from("Grass 2"),
                    color: Color::new(63, 132, 41),
                    height: 0.6
                },
                TerrainType {
                    name: String::from("Rock"),
                    color: Color::new(73, 38, 44),
                    height: 0.7
                },
                TerrainType {
                    name: String::from("Rock 2"),
                    color: Color::new(56, 30, 34),
                    height: 0.9
                },
                TerrainType {
                    name: String::from("Snow"),
                    color: Color::new(255, 255, 255),
                    height: 1.0
                }
              ]
          }
    }*/

pub fn gen_height_map(seed: u32, chunk_pos: cgmath::Vector3<i32>) -> Vec<i32> {
    let mut height_map = vec![0; chunk::CHUNK_SIZE.pow(2)];

    let noise = OpenSimplex::new();
    noise.set_seed(seed);

    for z in 0..chunk::CHUNK_SIZE {
        for x in 0..chunk::CHUNK_SIZE {
            let block_x = (x as i32 + chunk_pos.x * chunk::CHUNK_SIZE as i32) as f64;
            let block_z = (z as i32 + chunk_pos.z * chunk::CHUNK_SIZE as i32) as f64;

            let mut value = noise.get([block_x / chunk::CHUNK_SIZE as f64, block_z / chunk::CHUNK_SIZE as f64]);

            value = (value + 1.0) / 2.0;
            value *= 16.0;

            height_map[z * chunk::CHUNK_SIZE + x] = value as i32;
        }
    } 

    height_map
}

pub fn gen_smooth_terrain(chunk: &mut chunk::Chunk, height_map: &Vec<i32>) {
    let chunk_y = chunk.get_pos().y;
    let registry = Registry::current();
    
    for z in 0..chunk::CHUNK_SIZE {
        for x in 0..chunk::CHUNK_SIZE {
            let height = height_map[z * chunk::CHUNK_SIZE + x];
            
            for y in 0..chunk::CHUNK_SIZE {
                let block_y = chunk_y * chunk::CHUNK_SIZE as i32 + y as i32;// - 65;

                if block_y <= height {
                    if block_y == height {
                        if let Some(block) = registry.get_block(&Identifier::from_str("willekeurig:grass_block").unwrap()) {
                            chunk.add_block(x, y, z, Some(block.clone()));
                        }
                    } else {// if block_y <= 46 {
                        if let Some(block) = registry.get_block(&Identifier::from_str("willekeurig:stone").unwrap()) {
                            chunk.add_block(x, y, z, Some(block.clone()));
                        }
                    }/* else {
                        if let Some(block) = registry.get_block(Identifier::from_str("willekeurig:dirt").unwrap()) {
                            chunk.add_block(x, y, z, block);
                        }
                    }*/
                }
            }
        }
    }
}

/*fn inverse_lerp(a: f64, b: f64, value: f64) -> f64 {
    if a != b {
        ((value - a) / (b - a)).clamp(0.0, 1.0)
    }
    else {
        0.0
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}*/