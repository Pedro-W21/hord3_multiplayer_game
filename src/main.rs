#![feature(portable_simd)]
#![feature(int_roundings)]
#![feature(mpmc_channel)]
use std::{collections::HashMap, f32::consts::PI, path::PathBuf, simd::Simd, sync::{atomic::{AtomicUsize, Ordering}, mpmc::{self, channel}, Arc, RwLock}, thread, time::{Duration, Instant}};

use game_entity::colliders::AABB;
use cosmic_text::{Color, Font, Metrics};
use cutscene::{camera_movement::{CameraMovement, CameraMovementDuration, CameraMovementElement, CameraSequence}, demo_cutscene::{get_demo_cutscene, get_empty_cutscene}, game_shader::GameShader, real_demo_cutscene::get_real_demo_cutscene, write_in_the_air::get_positions_of_air_written_text, written_texture::get_written_texture_buffer};
use day_night::DayNight;
use game_3d_models::{clustered_ent_mesh, grey_sphere_mesh, lit_selection_cube, second_spread_out_ent_mesh, simple_line, sphere_mesh, spread_out_ent_mesh, textured_sphere_mesh, wireframe_sphere_mesh, xyz_mesh};
use game_engine::{CoolGameEngineBase, CoolVoxel, CoolVoxelType, ExtraData};
use game_entity::{Collider, GameEntityVec, Movement, NewGameEntity, StaticCollider, StaticGameEntity, StaticMeshInfo, StaticMovement, StaticStats, Stats};
use game_input_handler::GameInputHandler;
use game_map::{get_f64_pos, get_float_pos, light_spreader::{LightPos, LightSpread}, ChunkDims, GameMap, VoxelLight};
use gui_elements::{list_choice::get_list_choice, number_config::get_number_config};
use hord3::{defaults::{default_frontends::minifb_frontend::MiniFBWindow, default_rendering::vectorinator_binned::{meshes::{Mesh, MeshID, MeshLODS, MeshLODType}, rendering_spaces::ViewportData, shaders::NoOpShader, textures::{argb_to_rgb, rgb_to_argb, TextureSetID}, triangles::{color_u32_to_u8_simd, simd_rgb_to_argb}, Vectorinator}, default_ui::simple_ui::{SimpleUI, UIDimensions, UIElement, UIElementBackground, UIElementContent, UIElementID, UIEvent, UIUnit, UIUserAction, UIVector}}, horde::{frontend::{HordeWindowDimensions, WindowingHandler}, game_engine::{entity::Renderable, world::{WorldComputeHandler, WorldHandler}}, geometry::{plane::EquationPlane, rotation::{Orientation, Rotation}, vec3d::{Vec3D, Vec3Df}}, rendering::{camera::Camera, framebuffer::HordeColorFormat}, scheduler::{HordeScheduler, HordeTaskQueue, HordeTaskSequence, SequencedTask}, sound::{SoundRequest, WaveIdentification, WavePosition, WaveRequest, WaveSink, Waves}}};
use noise::{NoiseFn, Perlin, Seedable};
use tile_editor::{get_tile_voxels, TileEditorData};

use crate::{client::client_func, game_entity::{actions::{Action, ActionKind, ActionSource, ActionTimer, ActionsEvent, ActionsUpdate, StaticGameActions}, director::{llm_director::LLMDirector, Director, DirectorKind, StaticDirector}, planner::StaticPlanner, GameEntityEvent}, game_map::get_voxel_pos, proxima_link::ProximaLink, server::server_func};

pub mod game_map;
pub mod flat_game_map;
pub mod game_entity;
pub mod game_engine;
pub mod game_input_handler;
pub mod tile_editor;
pub mod gui_elements;
pub mod game_3d_models;
pub mod game_tiles;
pub mod cutscene;
pub mod day_night;
pub mod proxima_link;
pub mod client;
pub mod server;

fn main() {
    
    let args:Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        match args[1].trim() {
            "server" => server_func(),
            "client" => client_func(),
            _ => client_func(),
        }
    }
    else {
        client_func();
    }
}
