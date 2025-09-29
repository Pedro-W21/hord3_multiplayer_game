use std::{collections::HashMap, path::PathBuf, sync::{atomic::{AtomicUsize, Ordering}, mpmc::{Sender}, Arc, RwLock}};

use engine_derive::GameEngine;
use hord3::{defaults::default_rendering::vectorinator_binned::{rendering_spaces::ViewportData, shaders::NoOpShader, Vectorinator}, horde::{game_engine::{engine::{GameEngine, MovingObjectID}, entity::{Entity, EntityVec, MultiplayerEntity, Renderable}, multiplayer::{GlobalComponent, GlobalEvent, HordeEventReport, HordeMultiModeChoice, HordeMultiplayer, HordeMultiplayerMode, Identify, MultiplayerEngine}, world::{World, WorldComputeHandler, WorldEvent, WorldHandler, WorldOutHandler, WorldWriteHandler}}, geometry::vec3d::{Vec3D, Vec3Df}, rendering::camera::Camera, scheduler::IndividualTask, sound::{ARWWaves, WavesHandler}}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{cutscene::{game_shader::GameShader, reverse_camera_coords::reverse_from_raster_to_worldpos}, game_entity::{actions::{ActionsEvent, ActionsUpdate}, colliders::AABB, Collider, ColliderEvent, ColliderEventVariant, GameEntity, GameEntityEvent, GameEntityVecRead, GameEntityVecWrite, MovementEvent, MovementEventVariant}, game_map::{get_voxel_pos, GameMap, GameMapEvent, Voxel, VoxelLight, VoxelModel, VoxelType}, proxima_link::HordeProximaAIRequest};


#[derive(Clone, FromBytes, ToBytes, PartialEq, Debug)]
pub struct CoolVoxel {
    pub voxel_type:u16,
    pub orient:u8,
    pub light:VoxelLight,
    //pub extra_voxel_data:Option<Vec<ExtraVoxelData>>,
}

/// A passage can be
/// - flood-opened (all adjacent same passage)
/// - tied to a key
/// - a corridor opening (may be pre-opened by generation)
/// - an explicit point of entry into the room (only link to exits from others (corridor=false && entry=false) or corridors)

#[derive(Clone, FromBytes, ToBytes, PartialEq, Debug)]
pub struct PassageData {
    open_with_adjacent:bool,
    key_id:u16,
    possible_corridor:bool,
    is_entry:bool,
}

/// A trap is 
/// - activated one way (or periodically activated automatically)
/// - does a trap action at a position in a direction
/// - can have a cooldown, or be single use
/// - can activate with other adjacent traps or not
#[derive(Clone, FromBytes, ToBytes, PartialEq, Debug)]
pub struct TrapData {
    activation_type:ActivationType,
    action:TrapAction,
    cooldown:TrapCooldown,
    activate_with_all_adjacent:bool
}
#[derive(Clone, FromBytes, ToBytes, PartialEq, Debug)]
pub enum TrapCooldown {
    SingleUse{activated:bool},
    Ticks{max:usize, current:usize}
}
#[derive(Clone, FromBytes, ToBytes, PartialEq, Debug)]
pub enum TrapAction {
    Projectile,
    StraightDamage {hitbox:AABB},
    Effect
}

#[derive(Clone, FromBytes, ToBytes, PartialEq, Debug)]
pub enum ActivationType {
    AnyEntityContact,
    ProjectileContact,
    MonsterContact,
    PlayerContact,
    PlayerInteract,
    Periodic(usize)
}

#[derive(Clone, FromBytes, ToBytes, PartialEq, Debug)]
pub enum ExtraVoxelData {
    IsPassage(PassageData),
    IsLightSource(VoxelLight),
    IsTrap(TrapData)
    
}

impl CoolVoxel {
    pub fn new(voxel_type:u16, orient:u8, light:VoxelLight, extra_voxel_data:Option<Vec<ExtraVoxelData>>) -> Self {
        Self { voxel_type, orient, light, }//extra_voxel_data }
    }
}

#[derive(Clone, FromBytes, ToBytes, PartialEq)]
pub struct CoolVoxelType {
    pub empty_sides:u8,
    pub texture:usize,
    pub light_passthrough:VoxelLight,
    pub is_light_source:Option<VoxelLight>,
    pub name:String,
    pub texture_path:Option<String>,
    pub base_extra_voxel_data:Option<ExtraVoxelData>
}

impl CoolVoxelType {
    pub fn new(empty_sides:u8, texture:usize, light_passthrough:VoxelLight, is_light_source:Option<VoxelLight>, name:String, texture_path:Option<PathBuf>, base_extra_voxel_data:Option<ExtraVoxelData>) -> Self {
        Self { empty_sides, texture, light_passthrough, is_light_source, name, texture_path:texture_path.map(|path| {path.to_string_lossy().to_string()}), base_extra_voxel_data }
    }
}

impl VoxelType for CoolVoxelType {
    fn easy_texture(&self) -> usize {
        self.texture
    }
    fn sides_empty(&self) -> u8 {
        self.empty_sides
    }
    fn vertices_taken(&self) -> u8 {
        0
    }
    fn kind_of_model(&self) -> crate::game_map::VoxelModel {
        VoxelModel::WrappedTexture(self.texture)
    }
    fn light_passthrough(&self) -> VoxelLight {
        self.light_passthrough.clone()
    }
}

impl Voxel for CoolVoxel {
    type VT = CoolVoxelType;
    fn voxel_id(&self) -> usize {
        self.voxel_type as usize
    }
    fn orientation(&self) -> u8 {
        self.orient
    }
    fn light_level(&self) -> crate::game_map::VoxelLight {
        self.light
    }
}

fn get_push_to_next_integer_coords_in_dir(start:Vec3Df, dir:Vec3Df) -> Vec3Df {
    
    let push_to_x = if dir.x.is_sign_negative() {
        if start.x.is_sign_negative() {
            -start.x.fract()
        }
        else {
            1.0 - start.x.fract()
        }
    }
    else if dir.x != 0.0 {
        if start.x.is_sign_negative() {
            -(1.0 + start.x.fract())
        }
        else {
            -start.x.fract()
        }
    }
    else {
        0.0
    };
    let push_to_y = if dir.y.is_sign_negative() {
        if start.y.is_sign_negative() {
            -start.y.fract()
        }
        else {
            1.0 - start.y.fract()
        }
    }
    else if dir.y != 0.0 {
        if start.y.is_sign_negative() {
            -(1.0 + start.y.fract())
        }
        else {
            -start.y.fract()
        }
    }
    else {
        0.0
    };
    let push_to_z = if dir.z.is_sign_negative() {
        if start.z.is_sign_negative() {
            -start.z.fract()
        }
        else {
            1.0 - start.z.fract()
        }
        
    }
    else if dir.z != 0.0 {
        if start.z.is_sign_negative() {
            -(1.0 + start.z.fract())
        }
        else {
            -start.z.fract()
        }
    }
    else {
        0.0
    };
    Vec3Df::new(push_to_x, push_to_y, push_to_z)
}

fn get_push_to_next_integer_coords_in_dir_with_world(start:Vec3Df, dir:Vec3Df, world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>) -> Option<Vec3Df> {
    let test_pos = start + dir;
    match world.world.get_voxel_at(get_voxel_pos(test_pos)) {
        Some(voxel) => {    
            if !world.world.get_voxel_types()[voxel.voxel_type as usize].is_completely_empty() {
                Some(get_push_to_next_integer_coords_in_dir(test_pos, dir))
            }
            else {
                None
            }
        },
        None => {
            Some(get_push_to_next_integer_coords_in_dir(test_pos, dir))
        }
    }
}

const GRAVITY:f32 = 9.81/180.0;
const AIR_RESISTANCE:f32 = 0.99;
const DOWN_DIR:Vec3Df = Vec3Df::new(0.0,0.0, -0.5);
const OTHER_DIRS:[Vec3Df ; 5] = [
    Vec3Df::new(0.0,0.0, 0.5),
    Vec3Df::new(0.0,0.5, 0.0),
    Vec3Df::new(0.0,-0.5, 0.0),
    Vec3Df::new(0.5,0.0, 0.0),
    Vec3Df::new(-0.5,0.0, 0.0),
]; 

fn compute_tick<'a>(turn:EntityTurn, id:usize, first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>, second_ent:&GameEntityVecRead<'a, CoolGameEngineTID>, world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>, extra_data:&ExtraData) {
    match turn {
        EntityTurn::entity_1 => {
            let movement = &first_ent.movement[id];
            let mut total_push = Vec3D::zero();
            for i in world.world.set_grid.get_iter_from_to(movement.pos, movement.pos + movement.speed, 2, 1.0) {
                if i != id {
                    let other_move = &first_ent.movement[i];
                    let distance = other_move.pos.dist(&movement.pos);
                    if distance < 0.5 {
                        total_push += (movement.pos - other_move.pos) * ((0.5 - distance) * 2.0)
                    }
                }
            }
            first_ent.tunnels.movement_out.send(GameEntityEvent::new(true,MovementEvent::new(id, None, MovementEventVariant::AddToSpeed(total_push))));

            let actions = &first_ent.actions[id];
            let mut counter = actions.get_counter().clone();
            actions.perform(id, first_ent, second_ent, world, &mut counter, extra_data.tick.load(Ordering::Relaxed));
            first_ent.director[id].do_tick(id, first_ent, second_ent, world, extra_data.tick.load(Ordering::Relaxed), &mut counter);

            first_ent.tunnels.actions_out.send(GameEntityEvent::new(true,ActionsEvent::new(id, None, ActionsUpdate::UpdateCounter(counter))));
        },
        EntityTurn::entity_2 => {

        }
    }
}

fn get_nudge_to_nearest_next_whole(number:f32, delta_to_add:f32) -> f32 {
    if number.is_sign_positive() {
        let fract = number.fract();
        let nudge = if fract >= 0.5 {
            1.0 - fract + delta_to_add
        }
        else {
            -fract - delta_to_add
        };
        nudge
    }
    else {
        let fract = number.fract();
        let nudge = if fract <= -0.5 {
            -1.0 - fract - delta_to_add
        }
        else {
            -fract + delta_to_add
        };
        nudge
    }
}

fn compute_nudges_from(vertex:Vec3Df, spd:Vec3Df, collider:&Collider, world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>) -> (Vec3Df, bool) {
    let mut touching_ground = false;
    let mut nudges = Vec3Df::all_ones();
    let voxel = match world.world.get_voxel_at(get_voxel_pos(vertex)) {
        Some(voxel) => voxel.clone(),
        None => {CoolVoxel::new(0, 0, VoxelLight::zero_light(),None)}
    };
    if !world.world.get_voxel_types()[voxel.voxel_type as usize].is_completely_empty() {    
        let z_nudge = get_nudge_to_nearest_next_whole(vertex.z, 0.01);
        let z_nudged_collider = (collider.collider + spd + Vec3Df::new(0.0, 0.0, z_nudge));
        if z_nudged_collider.collision_world(&world.world) {
            let x_nudge = get_nudge_to_nearest_next_whole(vertex.x, 0.01);
            let y_nudge = get_nudge_to_nearest_next_whole(vertex.y, 0.01);
            let mut one_worked = true;
            if x_nudge.abs() < y_nudge.abs() {
                let x_nudged_collider = (collider.collider + spd + Vec3Df::new(x_nudge, 0.0, 0.0));
                if x_nudged_collider.collision_world(&world.world) {
                    one_worked = false;
                }
                else {
                    nudges.x = x_nudge;
                    nudges.z = 0.0;
                    nudges.y = 0.0;
                }
            }
            else {
                let y_nudged_collider = (collider.collider + spd + Vec3Df::new(0.0, y_nudge, 0.0));
                if y_nudged_collider.collision_world(&world.world) {
                    one_worked = false;
                }
                else {
                    nudges.y = y_nudge;
                    nudges.x = 0.0;
                    nudges.z = 0.0;
                }
            }
            if !one_worked {
                let xy_nudged_collider = (collider.collider + spd + Vec3Df::new(x_nudge, y_nudge, 0.0));
                if xy_nudged_collider.collision_world(&world.world) {
                    nudges.x = x_nudge;
                    nudges.y = y_nudge;
                    nudges.z = z_nudge;
                }
                else {
                    nudges.x = x_nudge;
                    nudges.y = y_nudge;
                    nudges.z = 0.0
                }
            }
        }
        else {
            touching_ground = true;
            nudges.z = z_nudge;
            nudges.x = 0.0;
            nudges.y = 0.0;
        }   
    }
    (nudges, touching_ground)
}


fn after_main_tick<'a>(turn:EntityTurn, id:usize, first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>, second_ent:&GameEntityVecRead<'a, CoolGameEngineTID>, world:&WorldComputeHandler<GameMap<CoolVoxel>, CoolGameEngineTID>, extra_data:&ExtraData) {
    match turn {
        EntityTurn::entity_1 => {
            let movement = &first_ent.movement[id];
            let collider = &first_ent.collider[id];
            let static_type = &first_ent.static_types[first_ent.stats[id].static_type_id];
            let mut movement_pos = movement.pos;
            //let mut movement_add = Vec3D::zero();
            let mut spd = movement.speed;
            spd *= AIR_RESISTANCE;
            spd.z -= GRAVITY;
            if spd.z.abs() > 0.45 {
                spd.z = 0.45 * spd.z.signum()
            }
            if spd.x.abs() > 0.45 {
                spd.x = 0.45 * spd.x.signum()
            }
            if spd.y.abs() > 0.45 {
                spd.y = 0.45 * spd.y.signum()
            }
            let mut touching_ground = false;
            let mut against_wall = false;
            let moved_aabb = (collider.collider + spd);
            let mut smallest_nudge = Vec3Df::all_ones();
            for vertex in moved_aabb.get_ground_vertices() {
                let (nudges, vertical) = compute_nudges_from(vertex, spd, collider, world);
                touching_ground = touching_ground | vertical;
                if touching_ground {
                    if nudges.z.abs() < smallest_nudge.z.abs() {
                        smallest_nudge = nudges;
                    }
                }
                else {
                    if nudges.norme_square() < smallest_nudge.norme_square() {
                        smallest_nudge = nudges;
                    }
                }
            }
            if !touching_ground {
                for vertex in moved_aabb.get_top_vertices() {
                    let (nudges, vertical) = compute_nudges_from(vertex, spd, collider, world);
                    if nudges.norme_square() < smallest_nudge.norme_square() {
                        smallest_nudge = nudges;
                    }
                }
            }
            if smallest_nudge != Vec3Df::all_ones() {
                if smallest_nudge.x != 0.0 || smallest_nudge.y != 0.0 && smallest_nudge.z > 0.0 {
                    against_wall = true;
                }
                spd += smallest_nudge;
            }
           
            
            /*match world.world.get_type_of_voxel_at(get_voxel_pos((movement_pos + spd + DOWN_DIR /*+ Vec3D::new(0.0, 0.0, -GRAVITY)*/))) {
                Some(voxel_type) => if !touching_ground && voxel_type.is_completely_empty() {
                    spd.z -= GRAVITY;
                }
                else {

                },
                None => if !touching_ground {
                    spd.z -= GRAVITY;
                }
            }*/
            match world.world.set_grid.get_point_move_update(&movement.pos, &(movement_pos + spd), id, 2) {
                Some(update) => {
                    //dbg!(update.clone());
                    world.tunnels.send_event(GameMapEvent::UpdateSetGrid(update))
                },
                None => ()
            }
            if touching_ground != movement.touching_ground {
                first_ent.tunnels.movement_out.send(GameEntityEvent::new(true,MovementEvent::new(id, None, MovementEventVariant::UpdateTouchingGround(touching_ground))));
            }
            if against_wall != movement.against_wall {
                first_ent.tunnels.movement_out.send(GameEntityEvent::new(true,MovementEvent::new(id, None, MovementEventVariant::UpdateAgainstWall(against_wall))));
            }
            first_ent.tunnels.movement_out.send(GameEntityEvent::new(false,MovementEvent::new(id, None, MovementEventVariant::UpdatePos(movement_pos + spd))));
            first_ent.tunnels.movement_out.send(GameEntityEvent::new(true,MovementEvent::new(id, None, MovementEventVariant::UpdateSpeed(spd))));
            first_ent.tunnels.collider_out.send(GameEntityEvent::new(true,ColliderEvent::new(id, None, ColliderEventVariant::UpdateCollider(static_type.collider.init_aabb + (movement_pos + spd )))));
            

            let planner = &first_ent.planner[id];
            planner.update(id, 100, first_ent, second_ent, world);
            
            first_ent.director[id].do_after_tick(id, first_ent, second_ent, world, &extra_data, extra_data.tick.load(Ordering::Relaxed));
            
        },
        _ => ()
    }
}

#[derive(Clone)]
pub struct ExtraData {
    pub tick:Arc<AtomicUsize>,
    pub waves:WavesHandler<CoolGameEngine>,
    pub current_render_data:Arc<RwLock<(Camera, ViewportData)>>,
    pub payload_sender:Sender<HordeProximaAIRequest>,
}


#[derive(GameEngine, Clone)]
#[rendering_engine = "Vectorinator"]
#[rendering_engine_generic = "GameShader"]
#[do_multiplayer]
pub struct CoolGameEngine {
    entity_1:GameEntity,
    entity_2:GameEntity,
    world:GameMap<CoolVoxel>,
    #[extra_data]
    extra_data:ExtraData
}