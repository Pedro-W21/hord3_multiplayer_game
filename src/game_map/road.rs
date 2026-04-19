use std::collections::HashSet;

use hord3::horde::geometry::{plane::{EquationPlane, VectorPlane}, vec3d::{Vec3D, Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{game_engine::CoolVoxel, game_map::{Collision, GameMap, Generator, VoxelLight, WorldChunkPos, WorldVoxelPos, get_float_pos, get_voxel_pos}};

#[derive(Clone, ToBytes, FromBytes, Debug, PartialEq)]
pub struct Road {
    start:WorldChunkPos,
    current_head_c:Vec3Df,
    current_direction_c:Vec3Df,
    // plane must be computed with world pos
    road_plane:EquationPlane,
    center_road_plane:EquationPlane,
    segments:Vec<RoadSegment>
}

impl Generator<CoolVoxel> for Road {
    const PROVIDES_COLLISION:bool = true;
    fn generate(&self, pos:WorldVoxelPos) -> CoolVoxel {
        // Compute the signed distance of the pos to the plane
        // if negative or 0, solid
        // otherwise, empty
        let float_pos = get_float_pos(pos);
        let dist = self.road_plane.signed_distance(&float_pos);
        if dist > -2.0 {
            CoolVoxel::new(0, 0, VoxelLight::max_light(), None)
        }
        else {
            let center_dist = self.center_road_plane.signed_distance(&float_pos);
            if center_dist.abs() <= 1.5 {
                CoolVoxel::new(2, 0, VoxelLight::max_light(), None)
            }
            else {
                CoolVoxel::new(1, 0, VoxelLight::max_light(), None)
            }
        }

    }
    fn full_collision(&self, pos:Vec3Df, speed_nudge:Vec3Df) -> Option<Collision<CoolVoxel>> {
        let mut closest_segment = None;
        let mut closest_distance = None;
        for i in 0..self.segments.len() {
            match self.segments[i].distance_to_road_if_in_segment(pos) {
                Some(distance) => if distance < closest_distance.unwrap_or(f32::INFINITY) {
                    closest_distance = Some(distance);
                    closest_segment = Some(i);
                },
                None => ()
            }
        }
        if let Some(dist) = closest_distance && dist < 0.0 && let Some(seg) = closest_segment {
            let segment = &self.segments[seg];
            let normal = segment.road_plane.get_normal();
            Some(Collision { surface_normal: normal, minimum_nudge:normal * dist.abs(), voxel: CoolVoxel::new(4, 0, VoxelLight::max_light(), None), position:pos })
        }
        else {
            None
        }
    }
}

impl Road {
    pub fn new(start:WorldChunkPos, start_dir:Vec3Df) -> Self {
        let perp = start_dir.cross(&Vec3D::new(0.0, 0.0, 1.0));
        let current_head_c = get_float_pos(start) + Vec3Df::new(-0.5, -0.5, 0.25);
        Self { start, current_head_c, current_direction_c: start_dir, road_plane: VectorPlane::new(perp, start_dir, current_head_c).to_equation_plane(), center_road_plane:EquationPlane::new(Vec3Df::all_ones(), 0.0), segments:Vec::with_capacity(32) }
    }
    pub fn get_chunks_to_generate(&self, steps:f32, world:&GameMap<CoolVoxel, Self>) -> Vec<WorldChunkPos> {
        let mut chunks = HashSet::with_capacity(16);
        let mut i = 0.3;
        while i <= steps {
            let at = self.current_head_c + self.current_direction_c * i;
            let chunkpos = get_voxel_pos(at);
            for xc in (chunkpos.x-3)..=(chunkpos.x+3) {
                for yc in (chunkpos.y-3)..=(chunkpos.y+3) {
                    for zc in (chunkpos.z-1)..=(chunkpos.z+1) {
                        chunks.insert(Vec3D::new(xc, yc, zc));
                    }
                }
            }
            i += 0.1;
        }
        chunks.drain().collect()
    }
    pub fn step_forwards(&mut self, steps:f32, chunk_dims:&Vec3Df) {
        let perp = self.current_direction_c.cross(&Vec3D::new(0.0, 0.0, 1.0));
        let normal = self.current_direction_c.cross(&perp);
        self.segments.push(
            RoadSegment {
                head: self.current_head_c.component_product(chunk_dims),
                length: steps * chunk_dims.x,
                direction: self.current_direction_c,
                road_plane: self.road_plane.clone(),
                center_road_plane: self.center_road_plane.clone(),
                road_slice_plane: VectorPlane::new(perp, normal, self.current_head_c.component_product(chunk_dims)).to_equation_plane()
            }
        );
        self.current_head_c += self.current_direction_c * steps;
        self.current_direction_c.x += (fastrand::f32() - 0.5) * 0.1;
        self.current_direction_c.y += (fastrand::f32() - 0.5) * 0.1;
        self.current_direction_c.z += (fastrand::f32() - 0.5) * 0.1;
        self.current_direction_c.z = self.current_direction_c.z.clamp(-0.1, 0.1);
        self.current_direction_c = self.current_direction_c.normalise();
        let perp = self.current_direction_c.cross(&Vec3D::new(0.0, 0.0, 1.0));
        let normal = self.current_direction_c.cross(&perp);
        self.road_plane = VectorPlane::new(perp, self.current_direction_c, self.current_head_c.component_product(chunk_dims)).to_equation_plane();
        self.center_road_plane = VectorPlane::new(Vec3D::new(0.0, 0.0, 1.0), self.current_direction_c, self.current_head_c.component_product(chunk_dims)).to_equation_plane();
    }
    pub fn position_within_last(&self, pos:Vec3Df) -> bool {
        let mut closest_segment = None;
        let mut closest_distance = None;
        for i in 0..self.segments.len() {
            match self.segments[i].distance_to_road_if_in_segment(pos) {
                Some(distance) => if distance < closest_distance.unwrap_or(f32::INFINITY) {
                    closest_distance = Some(distance);
                    closest_segment = Some(i);
                },
                None => ()
            }
        }
        if let Some(closest) = closest_segment && self.segments.len() - closest < 10 {
            true
        }
        else {
            false
        }
    }
}

#[derive(Clone, ToBytes, FromBytes, Debug, PartialEq)]
pub struct RoadSegment {
    head:Vec3Df,
    length:f32,
    direction:Vec3Df,
    road_plane:EquationPlane,
    center_road_plane:EquationPlane,
    road_slice_plane:EquationPlane

}

impl RoadSegment {
    pub fn in_segment(&self, pos:Vec3Df) -> bool {
        let back_dist = self.road_slice_plane.signed_distance(&pos);
        self.center_road_plane.signed_distance(&pos).abs() < 30.0 && self.road_plane.signed_distance(&pos).abs() <= 10.0 && back_dist >= 0.0 && back_dist <= self.length
    }
    pub fn distance_to_road(&self, pos:Vec3Df) -> f32 {
        self.road_plane.signed_distance(&pos)
    }
    pub fn distance_to_road_if_in_segment(&self, pos:Vec3Df) -> Option<f32> {
        if self.in_segment(pos) {
            Some(self.distance_to_road(pos))
        }
        else {
            None
        }
    }
}