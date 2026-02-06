use entity_derive::Entity;
use hord3::{defaults::default_rendering::vectorinator_binned::{VectorinatorWrite, meshes::{Mesh, MeshID, MeshInstance}}, horde::{game_engine::{entity::{Component, ComponentEvent, EVecStopsIn, EVecStopsOut, Entity, EntityID, EntityVec, MultiplayerEntity, NewEntity, StaticComponent, StaticEntity}, multiplayer::{Identify, MustSync}, position::EntityPosition, static_type_id::HasStaticTypeID}, geometry::{rotation::{Orientation, Rotation}, vec3d::Vec3Df}}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::driver::{actions::Actions, colliders::AABB, director::Director, planner::Planner, stats::Stats};
pub mod cutscene_support;
pub mod colliders;
pub mod actions;
pub mod director;
pub mod planner;
pub mod stats;


#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub struct Movement {
    pub pos:Vec3Df,
    pub speed:Vec3Df,
    pub orient:Orientation,
    pub rotat:Rotation,
    pub touching_ground:bool,
    pub against_wall:bool,
}

#[derive(Clone)]
pub struct StaticMovement {

}

impl StaticComponent for StaticMovement {
    
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum MovementEventVariant {
    UpdatePos(Vec3Df),
    AddToSpeed(Vec3Df),
    UpdateSpeed(Vec3Df),
    UpdateOrient(Orientation),
    UpdateRotat(Rotation),
    UpdateTouchingGround(bool),
    UpdateAgainstWall(bool)
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct MovementEvent<ID:Identify> {
    id:usize,
    source:Option<ID>,
    variant:MovementEventVariant
}

impl<ID:Identify> MovementEvent<ID> {
    pub fn new(id:usize, source:Option<ID>, variant:MovementEventVariant) -> Self {
        Self { id, source, variant }
    }
}

impl<ID:Identify> ComponentEvent<Movement, ID> for MovementEvent<ID> {
    type ComponentUpdate = MovementEventVariant;
    fn get_id(&self) -> EntityID {
        self.id
    }
    fn apply_to_component(self, components:&mut Vec<Movement>) {
        match self.variant {
            MovementEventVariant::UpdatePos(new_pos) => components[self.id].pos = new_pos,
            MovementEventVariant::AddToSpeed(speed_add) => components[self.id].speed += speed_add,
            MovementEventVariant::UpdateOrient(new_orient) => components[self.id].orient = new_orient,
            MovementEventVariant::UpdateRotat(new_rotat) => components[self.id].rotat = new_rotat,
            MovementEventVariant::UpdateSpeed(spd) => components[self.id].speed = spd,
            MovementEventVariant::UpdateTouchingGround(touching) => components[self.id].touching_ground = touching,
            MovementEventVariant::UpdateAgainstWall(against) => components[self.id].against_wall = against
        }
    }
    fn get_source(&self) -> Option<ID> {
        self.source.clone()
    }
}

impl<ID:Identify> Component<ID> for Movement {
    type CE = MovementEvent<ID>;
    type SC = StaticMovement;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { pos:Vec3Df::zero(), speed:Vec3Df::zero(), orient:Orientation::zero(), rotat: Rotation::from_orientation(Orientation::zero()), touching_ground:false, against_wall:false }
    }
}

impl<ID:Identify> EntityPosition<ID> for Movement {
    fn get_pos(&self) -> Vec3Df {
        self.pos
    }
    fn get_orientation(&self) -> Orientation {
        self.orient
    }
    fn get_rotation(&self) -> Option<&Rotation> {
        Some(&self.rotat)
    }
}
#[derive(Clone, PartialEq, ToBytes, FromBytes)]
pub struct Collider {
    pub team:u8,
    pub collider:AABB,
}

#[derive(Clone, ToBytes, FromBytes)]
pub struct StaticCollider {
    pub init_aabb:AABB,
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct ColliderEvent<ID:Identify> {
    id:usize,
    source:Option<ID>,
    variant:ColliderEventVariant
}

impl<ID:Identify> ColliderEvent<ID> {
    pub fn new(id:usize, source:Option<ID>, variant:ColliderEventVariant) -> Self {
        Self { id, source, variant }
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum ColliderEventVariant {
    UpdateCollider(AABB),
    ChangeTeam(u8),
}

impl<ID:Identify> ComponentEvent<Collider, ID> for ColliderEvent<ID> {
    type ComponentUpdate = ColliderEventVariant;
    fn get_id(&self) -> EntityID {
        self.id
    }
    fn get_source(&self) -> Option<ID> {
        self.source.clone()
    }
    fn apply_to_component(self, components:&mut Vec<Collider>) {
        match self.variant {
            ColliderEventVariant::ChangeTeam(new_team) => components[self.id].team = new_team,
            ColliderEventVariant::UpdateCollider(new_collider) => components[self.id].collider = new_collider,
        }
    }
}

impl StaticComponent for StaticCollider {

}

impl<ID:Identify> Component<ID> for Collider {
    type CE = ColliderEvent<ID>;
    type SC = StaticCollider;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { team: 0, collider:static_comp.init_aabb.clone() }
    }
}
#[derive(Clone, PartialEq, ToBytes, FromBytes)]
pub struct MeshInfo {
    instance_id:Option<usize>,

}

#[derive(Clone)]
pub struct StaticMeshInfo {
    pub mesh_data:Mesh,
    pub mesh_id:MeshID,
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct MeshEvent {
    id:usize,
    variant:MeshEventVariant
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum MeshEventVariant {
    UpdateInstanceID(Option<usize>),
}

impl<ID:Identify> ComponentEvent<MeshInfo, ID> for MeshEvent {
    type ComponentUpdate = MeshEventVariant;
    fn get_id(&self) -> EntityID {
        self.id
    }
    fn get_source(&self) -> Option<ID> {
        None
    }
    fn apply_to_component(self, components:&mut Vec<MeshInfo>) {
        match self.variant {
            MeshEventVariant::UpdateInstanceID(new_inst_id) => components[self.id].instance_id = new_inst_id,
        }
    }
}

impl StaticComponent for StaticMeshInfo {

}

impl<ID:Identify> Component<ID> for MeshInfo {
    type CE = MeshEvent;
    type SC = StaticMeshInfo;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { instance_id: None }
    }
}



impl<ID:Identify> NewEntity<GameEntity,ID> for NewGameEntity<ID> {
    fn get_ent(self, static_type:&StaticGameEntity<ID>) -> GameEntity {
        GameEntity {
            movement:self.movement,
            stats:self.stats,
            mesh_info:MeshInfo { instance_id: None },
            collider:self.collider,
            actions:Actions::new(),
            director:self.director,
            planner:Planner::new()
        }
    }
}

impl<'a, ID:Identify> RenderGameEntity<VectorinatorWrite<'a>, ID> for GameEntity {
    fn do_render_changes(rendering_data: &mut VectorinatorWrite<'a>,movement: &mut Movement,stats: &mut Stats,mesh_info: &mut MeshInfo,static_type: &StaticGameEntity<ID>) {
        match mesh_info.instance_id {
            Some(id) => {
                let mut instance = rendering_data.meshes.instances[2].get_instance_mut(id);
                instance.change_pos(movement.pos);
                instance.change_orient(movement.orient);
            },
            None => {
                if !rendering_data.meshes.does_mesh_exist(&static_type.mesh_info.mesh_id) {
                    rendering_data.meshes.add_mesh(static_type.mesh_info.mesh_data.clone());
                }
                mesh_info.instance_id = Some(rendering_data.meshes.add_instance(MeshInstance::new(movement.pos, movement.orient, static_type.mesh_info.mesh_id.clone(), true, false, false), 2))
            }
        }
    }
}


#[derive(Entity, Clone)]
pub struct GameEntity {
    #[position]
    #[used_in_render]
    #[used_in_new]
    #[must_sync]
    movement:Movement,
    #[static_id]
    #[used_in_new]
    #[must_sync]
    stats:Stats,
    #[used_in_render]
    mesh_info:MeshInfo,
    #[used_in_new]
    collider:Collider,
    actions:Actions,
    #[used_in_new]
    director:Director,
    planner:Planner
}