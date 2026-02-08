use std::collections::HashSet;

use hord3::horde::geometry::{plane::{EquationPlane, VectorPlane}, vec3d::{Vec3D, Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{game_engine::CoolVoxel, game_map::{GameMap, Generator, VoxelLight, WorldChunkPos, WorldVoxelPos, get_float_pos, get_voxel_pos}};

#[derive(Clone, ToBytes, FromBytes, Debug, PartialEq)]
pub struct Road {
    start:WorldChunkPos,
    current_head_c:Vec3Df,
    current_direction_c:Vec3Df,
    // plane must be computed with world pos
    head_dir_plane:EquationPlane,
    center_road_plane:EquationPlane
}

impl Generator<CoolVoxel> for Road {
    fn generate(&self, pos:WorldVoxelPos) -> CoolVoxel {
        // Compute the signed distance of the pos to the plane
        // if negative or 0, solid
        // otherwise, empty
        let float_pos = get_float_pos(pos);
        let dist = self.head_dir_plane.signed_distance(&float_pos);
        if dist > 0.0 {
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
}

impl Road {
    pub fn new(start:WorldChunkPos, start_dir:Vec3Df) -> Self {
        let perp = start_dir.cross(&Vec3D::new(0.0, 0.0, 1.0));
        let current_head_c = get_float_pos(start) + Vec3Df::new(0.5, 0.5, 0.5);
        Self { start, current_head_c, current_direction_c: start_dir, head_dir_plane: VectorPlane::new(perp, start_dir, current_head_c).to_equation_plane(), center_road_plane:EquationPlane::new(Vec3Df::all_ones(), 0.0) }
    }
    pub fn get_chunks_to_generate(&self, steps:f32, world:&GameMap<CoolVoxel, Self>) -> Vec<WorldChunkPos> {
        let mut chunks = HashSet::with_capacity(16);
        let mut i = 0.3;
        while i <= steps {
            let at = self.current_head_c + self.current_direction_c * i;
            let chunkpos = get_voxel_pos(at);
            for xc in (chunkpos.x-2)..=(chunkpos.x+2) {
                for yc in (chunkpos.y-2)..=(chunkpos.y+2) {
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
        self.current_head_c += self.current_direction_c * steps;
        self.current_direction_c.x += (fastrand::f32() - 0.5) * 0.1;
        self.current_direction_c.y += (fastrand::f32() - 0.5) * 0.1;
        self.current_direction_c.z += (fastrand::f32() - 0.5) * 0.1;
        self.current_direction_c.z = self.current_direction_c.z.clamp(-0.5, 0.3);
        self.current_direction_c = self.current_direction_c.normalise();
        let perp = self.current_direction_c.cross(&Vec3D::new(0.0, 0.0, 1.0));
        self.head_dir_plane = VectorPlane::new(perp, self.current_direction_c, self.current_head_c.component_product(chunk_dims)).to_equation_plane();
        self.center_road_plane = VectorPlane::new(Vec3D::new(0.0, 0.0, 1.0), self.current_direction_c, self.current_head_c.component_product(chunk_dims)).to_equation_plane();
    } 
}