
use core::f32;

use hord3::horde::geometry::{rotation::{Orientation, Rotation}, vec3d::Vec3Df};

use crate::{game_engine::CoolVoxel, game_map::road::Road};

use super::{get_voxel_pos, GameMap, VoxelType};


const PRECISION:f32 = 0.1;

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
    pub fn get_start(&self) -> Vec3Df {
        self.start
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

        if length < max_length && length != 0.0 {
            let mut final_precision = PRECISION * 0.5;
            for i in 0..8 {
                let dir = self.direction * final_precision;
                let test_back = test - dir;
                if chunks.is_voxel_solid(get_voxel_pos(test_back)) {
                    test = test_back;
                    length -= final_precision;
                }
                else {
                    final_precision *= 0.5
                }
            }
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

const CURVE_PRECISION:f32 = 0.01;

pub struct Curve {
    start:Vec3Df,
    center_of_rotation:Vec3Df,
    speed_to_end:Vec3Df,
    orient_diff:Orientation,
    start_ccr_diff: Vec3Df,
}

impl Curve {
    pub fn get_start(&self) -> Vec3Df {
        self.start
    }
    pub fn new(start:Vec3Df, center_of_rotation:Vec3Df, speed_to_end:Vec3Df, orient_diff:Orientation) -> Self {
        Self { start, center_of_rotation, speed_to_end, orient_diff, start_ccr_diff: (start - center_of_rotation) }
    }
    pub fn get_tangent_at(&self, coef:f32) -> Vec3Df {
        (self.get_at(coef + 0.01) - self.get_at(coef - 0.01)).normalise()
    }
    pub fn get_end(&self, chunks:&GameMap<CoolVoxel, Road>) -> ArcEnd {
        let mut coef = 0.0;

        //dbg!(self.speed_to_end);
        //dbg!(self.center_of_rotation);
        //dbg!(self.orient_diff);
        let mut test = self.get_at(coef);
        //dbg!(test);
        while coef < 1.0 && !chunks.is_voxel_solid(get_voxel_pos(test)) {
            coef += CURVE_PRECISION;
            test = self.get_at(coef);
        }
        //dbg!(test, coef);

        
        if coef < 1.0 && coef != 0.0 {
            let mut final_precision = CURVE_PRECISION * 0.5;
            for i in 0..8 {
                let test_back = self.get_at(coef - final_precision);
                if chunks.is_voxel_solid(get_voxel_pos(test_back)) {
                    test = test_back;
                    coef -= final_precision;
                }
                else {
                    final_precision *= 0.5
                }
            }
        }

        ArcEnd { end: test, tangent: self.get_tangent_at(coef), final_coef: coef }
    }
    pub fn get_at(&self, coef:f32) -> Vec3Df {
        let rotat = Rotation::from_orientation(self.orient_diff * coef);
        rotat.rotate(self.start_ccr_diff) + self.speed_to_end * coef + self.center_of_rotation
    }
}

pub struct ArcEnd {
    pub end:Vec3Df,
    pub tangent:Vec3Df,
    pub final_coef:f32
}