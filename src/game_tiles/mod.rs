use hord3::horde::geometry::vec3d::Vec3D;

use crate::{game_engine::CoolVoxel, game_map::{GameMap, road::Road}};

pub struct Tile {
    chunks:GameMap<CoolVoxel, Road>,
    enemies:Vec<TileEnemy>
}

pub struct OutsidePassage {
    chunk:Vec3D<i32>,
    direction:Vec3D<i32>
}

impl Tile {
    pub fn new(chunks:GameMap<CoolVoxel, Road>, enemies:Vec<TileEnemy>) -> Self {
        Self { chunks, enemies }
    }
    pub fn get_entries(&self) -> Vec<OutsidePassage> {
        let mut entries = Vec::with_capacity(8);
        for chunk_pos in self.chunks.get_all_chunk_pos() {
            
        }
        entries
    }
}

pub struct TileEnemy {

}