use std::{collections::{HashSet, VecDeque}, hash::Hash};

use hord3::{defaults::default_rendering::vectorinator_binned::triangles::{collux_f32_a_u8, collux_u8_a_f32}, horde::geometry::vec3d::Vec3D};

use crate::{game_engine::CoolVoxel, game_map::{VoxelLight, road::Road}};

use super::{GameMap, Voxel, VoxelType, EXPLORATION};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LightPos {
    pos:Vec3D<i32>,
    value:VoxelLight,
}

impl LightPos {
    pub fn new(pos:Vec3D<i32>, value:VoxelLight) -> Self {
        Self { pos, value }
    }
    pub fn pos(&self) -> Vec3D<i32> {
        self.pos
    }
    pub fn value(&self) -> VoxelLight {
        self.value
    }
}

pub struct LightSpread {
    positions:HashSet<Vec3D<i32>>,
    all_spread:Vec<LightPos>,
    spread_limits:HashSet<Vec3D<i32>>,
    edge_positions:Vec<LightPos>,
}

impl LightSpread {
    pub fn calc_max_spread(chunks:&GameMap<CoolVoxel, Road>, start_light:LightPos) -> Self {
        let mut positions = HashSet::with_capacity(1024);
        let mut edge_positions = Vec::with_capacity(1);
        let mut spread_limits = HashSet::with_capacity(256);
        let mut all_spread = Vec::with_capacity(256);
        edge_positions.push(start_light);
        positions.insert(start_light.pos);
        all_spread.push(start_light);
        while edge_positions.len() > 0 {
            let mut new_edge_positions = Vec::with_capacity(edge_positions.len() * 2);
            for pos in edge_positions {
                for dir in EXPLORATION {
                    let new_pos = pos.pos + dir;
                    if !positions.contains(&new_pos) && !spread_limits.contains(&new_pos) {
                        match chunks.get_voxel_at(new_pos) {
                            Some(voxel) => {
                                let light_passthrough = chunks.get_voxel_types()[voxel.voxel_id()].light_passthrough();
                                if light_passthrough.level == 0 {
                                    spread_limits.insert(new_pos);
                                }
                                else {
                                    let light_passthrough_f = collux_u8_a_f32((light_passthrough.r, light_passthrough.g, light_passthrough.b));
                                    let current_light = collux_u8_a_f32((pos.value.r, pos.value.g, pos.value.b));
                                    let new_light = collux_f32_a_u8((current_light.0 * light_passthrough_f.0, current_light.1 * light_passthrough_f.1, current_light.2 * light_passthrough_f.2));
                                    let new_level = pos.value.level - (255 - light_passthrough.level).min(pos.value.level);
                                    if new_level > 0 {
                                        let new_light = LightPos { pos: new_pos, value: VoxelLight {level:new_level, r:new_light.0, g:new_light.1, b:new_light.2} };
                                        all_spread.push(new_light);
                                        new_edge_positions.push(new_light);
                                        positions.insert(new_light.pos);
                                    }
                                }
                            },
                            None => {spread_limits.insert(new_pos);}// dbg!(new_pos);}, 
                        }
                    }
                    
                }
            }
            edge_positions = new_edge_positions;
        }
        
        Self { positions, edge_positions, spread_limits, all_spread }
    } 
    pub fn get_all_spread(self) -> Vec<LightPos> {
        self.all_spread
    }
}