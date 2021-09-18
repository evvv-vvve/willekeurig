use crate::chunk;

use common::block::Block;

// Credit: https://www.reddit.com/r/Unity3D/comments/5ys3vc/voxel_face_culling/desvzlu/
// Archived at: https://web.archive.org/web/20210528184220/https://www.reddit.com/r/Unity3D/comments/5ys3vc/voxel_face_culling/desvzlu/

pub enum CullCode
{
    Default = 0,  //0000
    F = 1,        //0001
    B = 2,        //0010
    L = 4,        //0100
    R = 8,        //1000
    LR = 12,      //1100
    FB = 3,       //0011
    FL = 5,       //0101
    FR = 9,       //1001
    BL = 6,       //0110
    BR = 10,      //1010
    FBL = 7,      //0111
    FBR = 11,     //1011
    FLR = 13,     //1101
    BLR = 14,     //1110
    FBLR = 15,    //1111

    U= 16,        //0001 0000
    UF = 17,      //0001 0001
    UB = 18,      //0001 0010
    UL = 20,      //0001 0100
    UR = 24,      //0001 1000
    ULR = 28,     //0001 1100
    UFB = 19,     //0001 0011
    UFL = 21,     //0001 0101
    UFR = 25,     //0001 1001
    UBL = 22,     //0001 0110
    UBR = 26,     //0001 1010
    UFBL = 23,    //0001 0111
    UFBR = 27,    //0001 1011
    UFLR = 29,    //0001 1101
    UBLR = 30,    //0001 1110
    UFBLR = 31,   //0001 1111

    D= 32,        //0010 0000
    DF = 33,      //0010 0001
    DB = 34,      //0010 0010
    DL = 36,      //0010 0100
    DR = 40,      //0010 1000
    DLR = 44,     //0010 1100
    DFB = 35,     //0010 0011
    DFL = 37,     //0010 0101
    DFR = 41,     //0010 1001
    DBL = 38,     //0010 0110
    DBR = 42,     //0010 1010
    DFBL = 39,    //0010 0111
    DFBR = 43,    //0010 1011
    DFLR = 45,    //0010 1101
    DBLR = 46,    //0010 1110
    DFBLR = 47,   //0010 1111

    UD= 48,       //0011 0000
    UDF = 49,     //0011 0001
    UDB = 50,     //0011 0010
    UDL = 52,     //0011 0100
    UDR = 56,     //0011 1000
    UDLR = 60,    //0011 1100
    UDFB = 51,    //0011 0011
    UDFL = 53,    //0011 0101
    UDFR = 57,    //0011 1001
    UDBL = 54,    //0011 0110
    UDBR = 58,    //0011 1010
    UDFBL = 55,   //0011 0111
    UDFBR = 59,   //0011 1011
    UDFLR = 61,   //0011 1101
    UDBLR = 62,   //0011 1110
    UDFBLR = 63   //0011 1111
}

pub fn cull_neighbors(chunk: &chunk::ChunkData, x: usize, y: usize, z: usize) -> u8 {
    let mut code = 0;

    if x > 0 {
        code = if let Some(_) = get_block_dirty(chunk, x - 1, y, z) { code } else { code | CullCode::L as u8 }
    }
    else {
        code |= CullCode::L as u8;
    }

    if z > 0 {
        code = if let Some(_) = get_block_dirty(chunk, x, y, z - 1) { code } else { code | CullCode::B as u8 }
    }
    else {
        code |= CullCode::B as u8;
    }

    if x < chunk::CHUNK_SIZE - 1 {
        code = if let Some(_) = get_block_dirty(chunk, x + 1, y, z) { code } else { code | CullCode::R as u8 }
    }
    else {
        code |= CullCode::R as u8;
    }

    if z < chunk::CHUNK_SIZE - 1 {
        code = if let Some(_) = get_block_dirty(chunk, x , y, z + 1) { code } else { code | CullCode::F as u8 }
    }
    else {
        code |= CullCode::F as u8;
    }

    if y < chunk::CHUNK_SIZE - 1 {
        code = if let Some(_) = get_block_dirty(chunk, x, y + 1, z) { code } else { code | CullCode::U as u8 }
    }
    else {
        code |= CullCode::U as u8;
    }

    if y > 0 {
        code = if let Some(_) = get_block_dirty(chunk, x, y - 1, z) { code } else { code | CullCode::D as u8 }
    }
    else {
        code |= CullCode::D as u8;
    }

    return code;
}

fn get_block_dirty(chunk: &chunk::ChunkData, x: usize, y: usize, z: usize) -> Option<Block> {
    chunk.get_block(x, y, z)
}