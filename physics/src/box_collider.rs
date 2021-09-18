use cgmath::{Vector3, Zero};

#[derive(Debug, Clone, Copy)]
pub struct BoxCollider {
    pub position: Vector3<f32>,
    pub size: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

impl BoxCollider {
    pub fn new(position: Vector3<f32>, size: Vector3<f32>, velocity: Vector3<f32>) -> Self {
        Self {
            position,
            size,
            velocity
        }
    }

    pub fn swept_aabb(&self, other: &Self) -> SweptResult {
        swept_aabb(&self, other)
    }

    pub fn collision_with(&self, other: &Self) -> bool {
        let self_position = self.position + self.velocity;
        let other_position = other.position + other.velocity;

        !(self_position.x + self.size.x < other_position.x || self_position.x > other_position.x + other.size.x ||
          self_position.y + self.size.y < other_position.y || self_position.y > other_position.y + other.size.y ||
          self_position.z + self.size.z < other_position.z || self_position.z > other_position.z + other.size.z
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SweptResult {
    pub time: f32,
    pub normal: Vector3<f32>,
}

pub fn swept_aabb(b1: &BoxCollider, b2: &BoxCollider) -> SweptResult {
    let mut inverse_entry: Vector3<f32> = Vector3::zero();
    let mut inverse_exit: Vector3<f32> = Vector3::zero();

    if b1.velocity.x > 0. {
        inverse_entry.x = b2.position.x - (b1.position.x + b2.size.x);
        inverse_exit.x = (b2.position.x + b2.size.x) - b1.position.x;
    } else {
        inverse_entry.x = (b2.position.x + b2.size.x) - b1.position.x;
        inverse_exit.x = b2.position.x - (b1.position.x + b1.size.x);
    }

    if b1.velocity.y > 0. {
        inverse_entry.y = b2.position.y - (b1.position.y + b2.size.y);
        inverse_exit.y = (b2.position.y + b2.size.y) - b1.position.y;
    } else {
        inverse_entry.y = (b2.position.y + b2.size.y) - b1.position.y;
        inverse_exit.y = b2.position.y - (b1.position.y + b1.size.y);
    }

    if b1.velocity.z > 0. {
        inverse_entry.z = b2.position.z - (b1.position.z + b2.size.z);
        inverse_exit.z = (b2.position.z + b2.size.z) - b1.position.z;
    } else {
        inverse_entry.z = (b2.position.z + b2.size.z) - b1.position.z;
        inverse_exit.z = b2.position.z - (b1.position.z + b1.size.z);
    }

    // find time of collision and time of leaving for each axis (if statement is to prevent divide by zero)
    let mut entry: Vector3<f32> = Vector3::zero();
    let mut exit: Vector3<f32> = Vector3::zero();

    if b1.velocity.x == 0. {
        entry.x = f32::NEG_INFINITY;
        exit.x = f32::NEG_INFINITY;
    } else {
        entry.x = inverse_entry.x / b1.velocity.x;
        exit.x = inverse_exit.x / b1.velocity.x;
    }

    if b1.velocity.y == 0. {
        entry.y = f32::NEG_INFINITY;
        exit.y = f32::NEG_INFINITY;
    } else {
        entry.y = inverse_entry.y / b1.velocity.y;
        exit.y = inverse_exit.y / b1.velocity.y;
    }

    if b1.velocity.z == 0. {
        entry.z = f32::NEG_INFINITY;
        exit.z = f32::NEG_INFINITY;
    } else {
        entry.z = inverse_entry.z / b1.velocity.z;
        exit.z = inverse_exit.z / b1.velocity.z;
    }

    if entry.x > 1.0 {
        entry.x = -f32::MAX;
    }

    if entry.y > 1.0 {
        entry.y = -f32::MAX;
    }

    if entry.z > 1.0 {
        entry.z = -f32::MAX;
    }

    // find the earliest/latest times of collision
    let entry_time = entry.x.max(entry.y.max(entry.z));
    let exit_time = exit.x.min(exit.y.min(exit.z));

    //println!(
    //    "entry: {:?}\t| exit: {:?}\t| entry_time: {}\t| exit_time: {}",
    //    entry, exit, entry_time, exit_time
    //);

    // if there was no collision
    if entry_time > exit_time ||
       entry.x < 0. && entry.y < 0. && entry.z < 0. ||
       entry.x > 1. || entry.y > 1. || entry.z > 1. {
        SweptResult {
            normal: Vector3::zero(),
            time: 1.
        }
    } else { // if there was a collision
        let mut normal = Vector3::zero();

        if entry.x > entry.y {
            if entry.x > entry.z {
                normal.x = -sign(b1.velocity.x);
                normal.y = 0.;
                normal.z = 0.;
            } else {
                normal.x = 0.;
                normal.y = 0.;
                normal.z = -sign(b1.velocity.z);
            }
        } else {
            if entry.y > entry.z {
                normal.x = 0.;
                normal.y = -sign(b1.velocity.y);
                normal.z = 0.;
            } else {
                normal.x = 0.;
                normal.y = 0.;
                normal.z = -sign(b1.velocity.z);
            }
        }

        SweptResult {
            normal,
            time: entry_time
        }
    }
}

fn sign(a: f32) -> f32 { if a >= 0. { 1. } else { -1. } }