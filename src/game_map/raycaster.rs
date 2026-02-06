
use core::f32;

use hord3::horde::geometry::{rotation::Orientation, vec3d::Vec3Df};

use crate::{game_engine::CoolVoxel, game_map::road::Road};

use super::{get_voxel_pos, GameMap, VoxelType};


const PRECISION:f32 = 0.25;

pub struct Ray {
    start:Vec3Df,
    direction:Vec3Df,
    max_length:Option<f32>
}

pub struct RayEnd {
    pub end:Vec3Df,
    pub final_length:f32
}

impl Ray {
    pub fn new(start:Vec3Df, direction:Vec3Df, max_length:Option<f32>) -> Self {
        Self { start, direction, max_length }
    }
    pub fn get_end(&self, chunks:&GameMap<CoolVoxel, Road>) -> RayEnd {
        let mut test = self.start.clone();
        let mut dir = self.direction * PRECISION;
        let max_length = self.max_length.unwrap_or(f32::INFINITY);
        let mut length = 0.0;
        //dbg!(dir);
        while length < max_length && chunks.get_type_of_voxel_at(get_voxel_pos(test)).is_some_and(|vox_type| {vox_type.sides_empty() == 0b00111111}) {
            test += dir;
            length += PRECISION;
        }
        return RayEnd { end:test, final_length:length }
    }
    pub fn get_first_back_different(&self, chunks:&GameMap<CoolVoxel, Road>, end:Option<RayEnd>) -> RayEnd {
        match end {
            Some(end) => {
                RayEnd {end:end.end - self.direction * PRECISION, final_length:end.final_length - PRECISION}
            },
            None => {
                let end = self.get_end(chunks);
                let new_end = end.end - self.direction * PRECISION;
                RayEnd {end:new_end, final_length:end.final_length - PRECISION}
            }
        }
    }
}