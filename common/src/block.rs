use renderer::vertex::Vertex;

use super::identifier::Identifier;

pub const VERTICES_FRONT: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0625, 0.0625], },
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.125, 0.0625], },
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.125, 0.0], },
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0625, 0.0], },
];

pub const VERTICES_BACK: &[Vertex] = &[
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0625, 0.0], },
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.125, 0.0], },
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.125, 0.0625], },
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0625, 0.0625], },
];

pub const VERTICES_TOP: &[Vertex] = &[
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 0.0], },
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.0625, 0.0], },
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.0625, 0.0625], },
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0625], },
];

pub const VERTICES_BOTTOM: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.125, 0.0], },
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.1875, 0.0], },
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.1875, 0.0625], },
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.125, 0.0625], },
];

pub const VERTICES_LEFT: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0625, 0.0625], },
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.125, 0.0625], },
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.125, 0.0], },
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0625, 0.0], },
];

pub const VERTICES_RIGHT: &[Vertex] = &[
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0625, 0.0625], },
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.125, 0.0625], },
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.125, 0.0], },
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.0625, 0.0], },
];

#[derive(Debug, Clone)]
pub struct TextureCoords {
    bottom_left_x: f32,
    bottom_left_y: f32,

    top_right_x: f32,
    top_right_y: f32
}

impl TextureCoords {
    pub const TEX_ATLAS_SIZE: f32 = 512.0;
    pub const TEX_WIDTH_HEIGHT: f32 = 32.0;

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            bottom_left_x: x / Self::TEX_ATLAS_SIZE,
            bottom_left_y: y / Self::TEX_ATLAS_SIZE,
            top_right_x: (x + Self::TEX_WIDTH_HEIGHT) / Self::TEX_ATLAS_SIZE,
            top_right_y: (y + Self::TEX_WIDTH_HEIGHT) / Self::TEX_ATLAS_SIZE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    identifier: Identifier,

    texture_front: TextureCoords,
    texture_back: TextureCoords,
    texture_top: TextureCoords,
    texture_btm: TextureCoords,
    texture_left: TextureCoords,
    texture_right: TextureCoords,

    //pub position: cgmath::Vector3<f32>,
    //pub rotation: cgmath::Quaternion<f32>,
}

impl Block {
    pub fn new(identifier: Identifier, tex_x: f32, tex_y: f32) -> Self {
        Self {
            identifier,

            texture_front: TextureCoords::new(tex_x, tex_y),
            texture_back: TextureCoords::new(tex_x, tex_y),
            texture_top: TextureCoords::new(tex_x, tex_y),
            texture_btm: TextureCoords::new(tex_x, tex_y),
            texture_left: TextureCoords::new(tex_x, tex_y),
            texture_right: TextureCoords::new(tex_x, tex_y),
        }
    }

    pub fn get_identifier(&self) -> &Identifier { &self.identifier }
}

// texture getters
impl Block {
    pub fn get_vert_front(&self) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = VERTICES_FRONT.into();

        verts[0].tex_coords = [ self.texture_back.bottom_left_x, self.texture_back.top_right_y ];
        verts[1].tex_coords = [ self.texture_back.top_right_x, self.texture_back.top_right_y ];
        verts[2].tex_coords = [ self.texture_back.top_right_x, self.texture_back.bottom_left_y ];
        verts[3].tex_coords = [ self.texture_back.bottom_left_x, self.texture_back.bottom_left_y ];

        verts
    }

    pub fn get_vert_back(&self) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = VERTICES_BACK.into();

        verts[0].tex_coords = [ self.texture_back.top_right_x, self.texture_back.bottom_left_y ];
        verts[1].tex_coords = [ self.texture_back.bottom_left_x, self.texture_back.bottom_left_y ];
        verts[2].tex_coords = [ self.texture_back.bottom_left_x, self.texture_back.top_right_y ];
        verts[3].tex_coords = [ self.texture_back.top_right_x, self.texture_back.top_right_y ];

        verts
    }

    pub fn get_vert_top(&self) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = VERTICES_TOP.into();

        verts[0].tex_coords = [ self.texture_top.bottom_left_x, self.texture_top.bottom_left_y ];
        verts[1].tex_coords = [ self.texture_top.top_right_x, self.texture_top.bottom_left_y ];
        verts[2].tex_coords = [ self.texture_top.top_right_x, self.texture_top.top_right_y ];
        verts[3].tex_coords = [ self.texture_top.bottom_left_x, self.texture_top.top_right_y ];

        verts
    }

    pub fn get_vert_bottom(&self) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = VERTICES_BOTTOM.into();

        verts[0].tex_coords = [ self.texture_btm.top_right_x, self.texture_btm.bottom_left_y ];
        verts[1].tex_coords = [ self.texture_btm.bottom_left_x, self.texture_btm.bottom_left_y ];
        verts[2].tex_coords = [ self.texture_btm.bottom_left_x, self.texture_btm.top_right_y ];
        verts[3].tex_coords = [ self.texture_btm.top_right_x, self.texture_btm.top_right_y ];

        verts
    }

    pub fn get_vert_left(&self) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = VERTICES_LEFT.into();

        verts[0].tex_coords = [ self.texture_front.top_right_x, self.texture_front.top_right_y ];
        verts[1].tex_coords = [ self.texture_front.bottom_left_x, self.texture_front.top_right_y ];
        verts[2].tex_coords = [ self.texture_front.bottom_left_x, self.texture_front.bottom_left_y ];
        verts[3].tex_coords = [ self.texture_front.top_right_x, self.texture_front.bottom_left_y ];

        verts
    }

    pub fn get_vert_right(&self) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = VERTICES_RIGHT.into();

        verts[0].tex_coords = [ self.texture_front.top_right_x, self.texture_front.top_right_y ];
        verts[1].tex_coords = [ self.texture_front.bottom_left_x, self.texture_front.top_right_y ];
        verts[2].tex_coords = [ self.texture_front.bottom_left_x, self.texture_front.bottom_left_y ];
        verts[3].tex_coords = [ self.texture_front.top_right_x, self.texture_front.bottom_left_y ];

        verts
    }
}

// texture setters
impl Block {
    pub fn set_texture_front(&mut self, tex_x: f32, tex_y: f32) {
        self.texture_front = TextureCoords::new(tex_x, tex_y);
    }

    pub fn set_texture_back(&mut self, tex_x: f32, tex_y: f32) {
        self.texture_back = TextureCoords::new(tex_x, tex_y);
    }

    pub fn set_texture_top(&mut self, tex_x: f32, tex_y: f32) {
        self.texture_top = TextureCoords::new(tex_x, tex_y);
    }

    pub fn set_texture_bottom(&mut self, tex_x: f32, tex_y: f32) {
        self.texture_btm = TextureCoords::new(tex_x, tex_y);
    }

    pub fn set_texture_left(&mut self, tex_x: f32, tex_y: f32) {
        self.texture_left = TextureCoords::new(tex_x, tex_y);
    }

    pub fn set_texture_right(&mut self, tex_x: f32, tex_y: f32) {
        self.texture_right = TextureCoords::new(tex_x, tex_y);
    }

    pub fn set_side_textures(&mut self, tex_x: f32, tex_y: f32) {
        self.set_texture_front(tex_x, tex_y);
        self.set_texture_back(tex_x, tex_y);
        self.set_texture_left(tex_x, tex_y);
        self.set_texture_right(tex_x, tex_y);
    }
}