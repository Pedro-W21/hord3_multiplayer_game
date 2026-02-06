use std::sync::mpmc::Sender;

use hord3::horde::{game_engine::{entity::{Component, ComponentEvent, StaticComponent}, multiplayer::{Identify, MustSync}}, geometry::{rotation::{Orientation, Rotation}, vec3d::{Coord, Vec3Df}}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{driver::{actions::{Action, ActionKind, ActionResult}, colliders::BoundingCollider}, game_engine::{CoolGameEngineTID, CoolVoxel}, game_map::{GameMap, get_voxel_pos, raycaster::Ray, road::Road}, vehicle::{VehicleEntityEvent, position::{VehiclePosEvent, VehiclePosUpdate, VehiclePosition}, vehicle_stats::VehicleStats}};

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub struct Locomotion {
    equipment:Vec<LocomotionEquipment>,
    driver_actions:Vec<(Action, ActionResult)>,
}

impl Locomotion {
    pub fn new(equipment:Vec<LocomotionEquipment>) -> Self {
        Self { equipment, driver_actions:vec![] }
    }
    pub fn compute_vehicle_and_locomotion_changes(
        &self,
        self_id:usize,
        static_locomotion:&StaticLocomotion,
        world:&GameMap<CoolVoxel, Road>,
        vehicle_stats:&VehicleStats,
        vehicle_position:&VehiclePosition,
        loco_events:&Sender<VehicleEntityEvent<LocomotionEvent<CoolGameEngineTID>>>,
        pos_events:&Sender<VehicleEntityEvent<VehiclePosEvent<CoolGameEngineTID>>>
    ) {
        let mut vehicle_spd_change = Vec3Df::zero();
        let mut vehicle_turn_spd_change = Orientation::zero();
        let mut new_eqs = Vec::with_capacity(self.equipment.len());
        for eq in &self.equipment {
            let (new_eq, v_spd_chng, v_turn_spd_chng) = eq.compute_activations_and_update_equipment(&static_locomotion.equipment[eq.static_equipment], world, vehicle_stats, vehicle_position, &self.driver_actions);
            if v_turn_spd_chng != Orientation::zero() {
                dbg!(v_turn_spd_chng);
                dbg!(vehicle_turn_spd_change);
            }
            vehicle_spd_change += v_spd_chng;
            vehicle_turn_spd_change += v_turn_spd_chng;
            new_eqs.push(new_eq);
        }
        dbg!(vehicle_spd_change, vehicle_turn_spd_change);
        loco_events.send(VehicleEntityEvent::new(MustSync::Server, LocomotionEvent::new(self_id, None, LocomotionUpdate::UpdateEverything(new_eqs)))).unwrap();
        if self.driver_actions.len() > 0 {
            loco_events.send(VehicleEntityEvent::new(MustSync::Server, LocomotionEvent::new(self_id, None, LocomotionUpdate::FlushActions))).unwrap();
        }
        pos_events.send(VehicleEntityEvent::new(MustSync::Server,VehiclePosEvent::new(self_id, None, VehiclePosUpdate::AddToEverySpeed(vehicle_spd_change, vehicle_turn_spd_change)))).unwrap();
    }
}

impl StaticComponent for StaticLocomotion {
    
}

impl<ID:Identify> Component<ID> for Locomotion {
    type SC = StaticLocomotion;
    type CE = LocomotionEvent<ID>;
    fn from_static(static_comp:&Self::SC) -> Self {
        let mut equipment = Vec::with_capacity(static_comp.equipment.len());
        for (i, eq) in static_comp.equipment.iter().enumerate() {
            equipment.push(LocomotionEquipment {
                static_equipment:i,
                current_local_position:eq.resting_local_position,
                current_local_speed:Vec3Df::zero(),
                current_local_orient:Orientation::zero(),
                current_local_turn_speed:Orientation::zero(),
                current_collider:eq.collider.clone()
            });
        }
        Self { equipment, driver_actions:vec![] }
    }
}

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub struct LocomotionEvent<ID:Identify> {
    id:usize,
    source:Option<ID>,
    update:LocomotionUpdate
}

impl<ID:Identify> LocomotionEvent<ID> {
    pub fn new(id:usize, source:Option<ID>, update:LocomotionUpdate) -> Self {
        Self { id, source, update }
    }
}

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub enum LocomotionUpdate {
    UpdateEverything(Vec<LocomotionEquipment>),
    UpdateEverySpeed(Vec<(Vec3Df, Orientation)>),
    AddToEverySpeed(Vec<(Vec3Df, Orientation)>),
    FlushActions,
    AddAction(Action, ActionResult)
}

impl<ID:Identify> ComponentEvent<Locomotion, ID> for LocomotionEvent<ID> {
    type ComponentUpdate = LocomotionUpdate;
    fn get_id(&self) -> hord3::horde::game_engine::entity::EntityID {
        self.id
    }
    fn get_source(&self) -> Option<ID> {
        self.source.clone()
    }
    fn apply_to_component(self, components:&mut Vec<Locomotion>) {
        match self.update {
            LocomotionUpdate::UpdateEverything(updates) => for (eq, up) in components[self.id].equipment.iter_mut().zip(updates) {
                *eq = up;
            },
            LocomotionUpdate::UpdateEverySpeed(updates) => for (eq, up) in components[self.id].equipment.iter_mut().zip(updates) {
                eq.current_local_speed = up.0;
                eq.current_local_turn_speed = up.1;
            },
            LocomotionUpdate::AddToEverySpeed(updates) => for (eq, up) in components[self.id].equipment.iter_mut().zip(updates) {
                eq.current_local_speed += up.0;
                eq.current_local_turn_speed += up.1;
            },
            LocomotionUpdate::FlushActions => components[self.id].driver_actions.clear(),
            LocomotionUpdate::AddAction(act, res) => components[self.id].driver_actions.push((act, res)),
        }
    }
}

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub struct LocomotionEquipment {
    static_equipment:usize,
    current_local_position:Vec3Df,
    current_local_speed:Vec3Df,
    current_local_orient:Orientation,
    current_local_turn_speed:Orientation,
    current_collider:BoundingCollider,
}

pub struct Activated {
    activation_id:usize,
    strength:f32
}

impl LocomotionEquipment {
    pub fn can_activate(
        &self,
        static_type:&StaticLocomotionEquipment,
        world:&GameMap<CoolVoxel, Road>,
        vehicle_stats:&VehicleStats,
        vehicle_position:&VehiclePosition,
        driver_actions:&Vec<(Action, ActionResult)>
    ) -> Vec<Activated> {
        let mut activations = Vec::with_capacity(2);
        'reqs: for (i, reqs) in static_type.activation_requirements.iter().enumerate() {
            let mut possible = true;
            let mut strength = 0.0;
            for req in &reqs.requirements {
                match req {
                    ActivationRequirement::NitroAmount(amount) => possible = possible && vehicle_stats.nitro_left > *amount,
                    ActivationRequirement::DriverAction(action) => {
                        let mut any_action_compatible = false;
                        for (act, result) in driver_actions {
                            match act.get_kind() {
                                ActionKind::Throttle(value) => match action {
                                    DriverAction::Throttle { from, to } => {
                                        any_action_compatible = any_action_compatible || (*value >= *from && *value <= *to);
                                        if any_action_compatible {
                                            strength = *value;
                                        }
                                    },
                                    _ => ()
                                },
                                ActionKind::Turn(value) => match action {
                                    DriverAction::HorizontalReorientation { from, to } => {
                                        any_action_compatible = any_action_compatible || (*value >= *from && *value <= *to);
                                        if any_action_compatible {
                                            strength = *value;
                                        }
                                    },
                                    _ => ()
                                },
                                ActionKind::ActivateNitro => match action {
                                    DriverAction::Nitro => any_action_compatible = true,
                                    _ => ()
                                },
                                _ => ()
                            }
                        }
                        possible = possible && any_action_compatible
                    },
                    ActivationRequirement::DistanceToSurface { from, to, surface_type } => {
                        let self_rotation = Rotation::from_orientation(self.current_local_orient);
                        let vehicle_rotation = Rotation::from_orientation(vehicle_position.orientation);
                        let ray = Ray::new(self.current_local_position + vehicle_position.pos, vehicle_rotation.rotate(self_rotation.rotate(static_type.down_dir)), Some(*to));
                        let end = ray.get_end(&world);
                        if end.final_length >= *from && end.final_length <= *to {
                            match world.get_type_of_voxel_at(get_voxel_pos(end.end)) {
                                Some(voxel_type) => possible = possible && voxel_type.surface_type == *surface_type,
                                None => possible = false
                            }
                        }
                    },
                    ActivationRequirement::HullPosition => todo!("Figure out where that is useful and how to implement it"),
                    ActivationRequirement::SurfaceContact(surface_type) => {

                        let self_rotation = Rotation::from_orientation(self.current_local_orient);
                        let vehicle_rotation = Rotation::from_orientation(vehicle_position.orientation);
                        let ray = Ray::new(self.current_local_position + vehicle_position.pos, vehicle_rotation.rotate(self_rotation.rotate(static_type.down_dir)), Some(100.0));
                        let end = ray.get_end(&world);
                        match world.get_type_of_voxel_at(get_voxel_pos(end.end)) {
                            Some(voxel_type) => {
                                if self.current_collider.point_inside(end.end) {
                                    possible = possible && voxel_type.surface_type == *surface_type;
                                }
                            },
                            None => possible = false
                        }
                    },
                }
                if driver_actions.len() > 0 {

                    //dbg!(req);
                    //dbg!(possible);
                    //dbg!(driver_actions.len());
                }
                if !possible {
                    continue 'reqs;
                }
            }
            if possible {
                activations.push(Activated { activation_id: i, strength });
            }
        }
        activations
    }
    pub fn compute_vehicle_speed_vector_and_turn_spd_change(&self,
        static_type:&StaticLocomotionEquipment,
        world:&GameMap<CoolVoxel, Road>,
        vehicle_stats:&VehicleStats,
        vehicle_position:&VehiclePosition,
        strength:f32,
    ) -> (Vec3Df, Orientation) {
        let vehicle_rotat = Rotation::from_orientation(vehicle_position.orientation);
        let equip_rotat = Rotation::from_orientation(self.current_local_orient);
        let rotated_motion_vector = vehicle_rotat.rotate(equip_rotat.rotate(static_type.motion.equipment_local_motion_vector * strength));
        
        match static_type.motion.application_point {
            ApplicationPoint::CenterOfEquipment => {
                let ap = vehicle_rotat.rotate(self.current_local_position);
                let center_of_gravity = Vec3Df::zero();
                let lever = ap - center_of_gravity;
                let moment = lever.cross(&rotated_motion_vector) / vehicle_stats.mass;
                let orient_change = Orientation::new(moment.z, moment.y, moment.x);
                let normalised_dot = rotated_motion_vector.normalise().dot(&lever.normalise());
                let resulting_force = (rotated_motion_vector * normalised_dot) / vehicle_stats.mass;
                
                (resulting_force, orient_change)
            },
            ApplicationPoint::CenterOfGravity => {
                (rotated_motion_vector / vehicle_stats.mass, Orientation::zero())
            },
            ApplicationPoint::ContactPoint => todo!("Compute contact point motion vector (e.g. wheel against ground)")
        }
    }
    pub fn compute_new_self_orient(&self,
        static_type:&StaticLocomotionEquipment,
    ) -> Orientation {
        let max_rotation = static_type.max_self_rotation;
        let vec3D_max_rotat = Vec3Df::new(max_rotation.roll, max_rotation.pitch, max_rotation.yaw);
        let mut vec3D_self_rotat = Vec3Df::new(self.current_local_orient.roll, self.current_local_orient.pitch, self.current_local_orient.yaw);
        let mut vec3D_rotat_spd = Vec3Df::new(self.current_local_turn_speed.roll, self.current_local_turn_speed.pitch, self.current_local_turn_speed.yaw);
        vec3D_self_rotat = (vec3D_self_rotat + vec3D_rotat_spd).clamp(-vec3D_max_rotat.x, -vec3D_max_rotat.y, -vec3D_max_rotat.z, vec3D_max_rotat.x, vec3D_max_rotat.y, vec3D_max_rotat.z);
        Orientation::new(vec3D_self_rotat.z, vec3D_self_rotat.y, vec3D_self_rotat.x)
    }
    pub fn compute_turn_speed_change(&self, rotation_axis:Coord, strength:f32) -> Orientation {
        let mut change = Orientation::zero();
        match rotation_axis {
            Coord::X => change.roll += strength,
            Coord::Y => change.pitch += strength,
            Coord::Z => change.yaw += strength,
        }
        change
    }
    pub fn compute_activations_and_update_equipment(
        &self,
        static_type:&StaticLocomotionEquipment,
        world:&GameMap<CoolVoxel, Road>,
        vehicle_stats:&VehicleStats,
        vehicle_position:&VehiclePosition,
        driver_actions:&Vec<(Action, ActionResult)>
    ) -> (Self, Vec3Df, Orientation) {
        let mut vehicle_spd_add = Vec3Df::zero();
        let mut vehicle_turn_spd_add = Orientation::zero();
        let mut self_clone = self.clone();
        for activated in self.can_activate(static_type, world, vehicle_stats, vehicle_position, driver_actions) {
            match static_type.activation_requirements[activated.activation_id].output {
                ActivationOutput::ActivateMotion => {
                    let (spd_add, turn_spd_add) = self_clone.compute_vehicle_speed_vector_and_turn_spd_change(static_type, world, vehicle_stats, vehicle_position, activated.strength);
                    vehicle_spd_add += spd_add;
                    vehicle_turn_spd_add += turn_spd_add;
                    dbg!(spd_add, turn_spd_add);
                },
                ActivationOutput::Turn(axis) => {
                    let orient_change = self_clone.compute_turn_speed_change(axis, activated.strength);
                    self_clone.current_local_turn_speed += orient_change;
                    dbg!(orient_change);
                }
            }
        }
        self_clone.current_local_orient = self_clone.compute_new_self_orient(static_type);
        if vehicle_spd_add != Vec3Df::zero() {
            //dbg!(vehicle_spd_add);
            //dbg!(self_clone.current_local_turn_speed);
            //dbg!(self_clone.current_local_orient);
        }
        (self_clone, vehicle_spd_add, vehicle_turn_spd_add)
    }

}

#[derive(Clone, ToBytes, FromBytes)]
pub struct StaticLocomotionEquipment {
    pub activation_requirements:Vec<ActivationRequirements>,
    pub resting_local_position:Vec3Df,
    pub max_self_rotation:Orientation,
    pub motion:EqMotion,
    pub recoil:EqRecoil,
    pub collider:BoundingCollider,
    pub down_dir:Vec3Df
}

#[derive(Clone, Debug, ToBytes, FromBytes)]
pub enum EqMotionKind {
    Switch,
    AnalogLinear
}

#[derive(Clone, Debug, ToBytes, FromBytes)]
pub struct EqMotion {
    pub equipment_local_motion_vector:Vec3Df,
    pub kind:EqMotionKind,
    pub application_point:ApplicationPoint
}

#[derive(Clone, Debug, ToBytes, FromBytes)]
pub enum ApplicationPoint {
    CenterOfEquipment,
    CenterOfGravity,
    ContactPoint
}

#[derive(Clone, Debug, ToBytes, FromBytes)]
pub enum EqRecoilKind {
    InstantOnActivation,
    ProgressiveOnActivation(f32),
    SuspensionOnContact(f32)
}

#[derive(Clone, Debug, ToBytes, FromBytes)]
pub struct EqRecoil {
    pub kind:EqRecoilKind,
    pub equipment_local_vector_towards_recoil:Vec3Df,
    pub max_recoil:f32 // number of times the vector can be applied in recoil
}


#[derive(Clone, ToBytes, FromBytes)]
pub struct StaticLocomotion {
    pub equipment:Vec<StaticLocomotionEquipment>,

}

#[derive(Clone, ToBytes, FromBytes)]
pub enum ActivationOutput {
    ActivateMotion,
    Turn(Coord), // turning around the given local axis
}

#[derive(Clone, ToBytes, FromBytes)]
pub struct ActivationRequirements {
    requirements:Vec<ActivationRequirement>,
    output:ActivationOutput
}

impl ActivationRequirements {
    pub fn new(requirements:Vec<ActivationRequirement>, output:ActivationOutput) -> Self {
        Self { requirements, output }
    }
}

#[derive(Clone, Debug, ToBytes, FromBytes)]
pub enum ActivationRequirement {
    NitroAmount(f32),
    SurfaceContact(Option<SurfaceType>),
    DistanceToSurface{from:f32, to:f32, surface_type:Option<SurfaceType>},
    DriverAction(DriverAction),
    HullPosition
}

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]

pub enum DriverAction {
    Throttle{from:f32, to:f32},
    HorizontalReorientation {from:f32, to:f32},
    Nitro
}

impl Eq for DriverAction {
    
}

impl DriverAction {
    pub fn throttle_in_bounds(&self, throttle:f32) -> bool {
        match self {
            DriverAction::Throttle { from, to } => throttle >= *from && throttle <= *to,
            _ => false
        }
    }
    pub fn turning_in_bounds(&self, turn:f32) -> bool {
        match self {
            DriverAction::HorizontalReorientation { from, to } => turn >= *from && turn <= *to,
            _ => false
        }
    }
    pub fn is_nitro(&self) -> bool {
        match self {
            DriverAction::Nitro => true,
            _ => false
        }
    }
}

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub enum SurfaceType {
    Ground,
    Water,
    Any
}