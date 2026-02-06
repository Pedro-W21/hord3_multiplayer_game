use hord3::horde::geometry::{plane::{EquationPlane, VectorPlane}, vec3d::{Vec3D, Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{game_engine::CoolVoxel, game_map::{Generator, VoxelLight, WorldChunkPos, WorldVoxelPos, get_float_pos}};

#[derive(Clone, ToBytes, FromBytes, Debug, PartialEq)]
pub struct Road {
    start:WorldChunkPos,
    current_head_c:Vec3Df,
    current_direction_c:Vec3Df,
    head_dir_plane:EquationPlane
}

impl Generator<CoolVoxel> for Road {
    fn generate(&self, pos:WorldVoxelPos) -> CoolVoxel {
        // Compute the signed distance of the pos to the plane
        // if negative or 0, solid
        // otherwise, empty
        let dist = self.head_dir_plane.signed_distance(&get_float_pos(pos));
        if dist > 0.0 {
            CoolVoxel::new(0, 0, VoxelLight::max_light(), None)
        }
        else {
             CoolVoxel::new(1, 0, VoxelLight::max_light(), None)
        }

    }
}

impl Road {
    pub fn new(start:WorldChunkPos, start_dir:Vec3Df) -> Self {
        let perp = start_dir.cross(&Vec3D::new(0.0, 0.0, 1.0));
        let current_head_c = get_float_pos(start) + Vec3Df::new(0.5, 0.5, 0.5);
        Self { start, current_head_c, current_direction_c: start_dir, head_dir_plane: VectorPlane::new(perp, start_dir, current_head_c).to_equation_plane() }
    }
}