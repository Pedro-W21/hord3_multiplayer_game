use std::{fmt::Debug, sync::mpmc::Sender};

use hord3::horde::{game_engine::{entity::{Component, ComponentEvent, SimpleComponentEvent, StaticComponent}, multiplayer::{Identify, MustSync}, world::WorldComputeHandler}, geometry::{rotation::{Orientation, Rotation}, vec3d::{Coord, Vec3Df}}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{driver::{actions::{Action, ActionKind, ActionResult}, colliders::BoundingCollider}, game_engine::{AIR_RESISTANCE, CoolGameEngineTID, CoolVoxel, GRAVITY, TURN_RESISTANCE, get_nudge_to_nearest_next_whole}, game_map::{GameMap, VoxelLight, VoxelType, get_voxel_pos, raycaster::{Curve, Ray}, road::Road}, vehicle::{StaticVehicleEntity, VehicleEntityEvent, hull::HullUpdate, position::{VehiclePosEvent, VehiclePosUpdate, VehiclePosition}, vehicle_stats::VehicleStats}};

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub struct Locomotion {
    pub equipment:Vec<LocomotionEquipment>,
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
        let vehicle_rotat = Rotation::from_orientation(vehicle_position.orientation);
        let mut new_eqs = Vec::with_capacity(self.equipment.len());
        let mut on_ground_eqs = Vec::with_capacity(self.equipment.len());
        let mut not_on_ground_eqs= Vec::with_capacity(self.equipment.len());
        for (i, eq) in self.equipment.iter().enumerate() {
            let (new_eq, v_spd_chng, v_turn_spd_chng) = eq.compute_activations_and_update_equipment(&static_locomotion.equipment[eq.static_equipment], world, vehicle_stats, vehicle_position, &self.driver_actions);
            if let Some(on_ground) = new_eq.compute_on_ground_if_relevant(&static_locomotion.equipment[eq.static_equipment], world, vehicle_stats, vehicle_position) {
                if on_ground {
                    on_ground_eqs.push(i);
                }
                else {
                    not_on_ground_eqs.push(i);
                }
            }
            if v_turn_spd_chng != Orientation::zero() {
                dbg!(v_turn_spd_chng);
                dbg!(vehicle_turn_spd_change);
            }
            vehicle_spd_change += vehicle_rotat.rotate(v_spd_chng);
            vehicle_turn_spd_change += v_turn_spd_chng;
            new_eqs.push(new_eq);
        }

        // apply speed from ground equipment that is on the ground
        //dbg!(on_ground_eqs.len(), not_on_ground_eqs.len());
        let total_spd = (vehicle_position.spd + vehicle_spd_change);
        if on_ground_eqs.len() > 0 && total_spd.norme_square() > 0.0001 {
            let divided_spd = total_spd/on_ground_eqs.len() as f32;
            let dot = vehicle_rotat.rotate(Vec3Df::new(1.0, 0.0, 0.0)).dot(&divided_spd.normalise());
            let final_spd = Vec3Df::new(1.0, 0.0, 0.0) * divided_spd.norme() * dot;
            for ground in on_ground_eqs {
                let (spd_add, turn_spd_add, _) = new_eqs[ground].compute_vehicle_speed_vector_and_turn_spd_change(&static_locomotion.equipment[new_eqs[ground].static_equipment], world, vehicle_stats, vehicle_position, final_spd.norme(),final_spd.normalise(), MotionApplication::FlatAlong2AxisFromEquipment { removed: Coord::Z });
                vehicle_turn_spd_change += turn_spd_add;
                //vehicle_spd_change += spd_add  * 0.01;
                //let (spd_add, turn_spd_add) = new_eqs[ground].compute_vehicle_speed_vector_and_turn_spd_change(&static_locomotion.equipment[new_eqs[ground].static_equipment], world, vehicle_stats, vehicle_position, gravity, Vec3Df::new(0.0, 0.0, 1.0), MotionApplication::WorldCoords);
                //vehicle_turn_spd_change += turn_spd_add;
            }
        }
        

        //dbg!(vehicle_spd_change, vehicle_turn_spd_change);
        loco_events.send(VehicleEntityEvent::new(MustSync::Server, LocomotionEvent::new(self_id, None, LocomotionUpdate::UpdateEverything(new_eqs)))).unwrap();
        if self.driver_actions.len() > 0 {
            loco_events.send(VehicleEntityEvent::new(MustSync::Server, LocomotionEvent::new(self_id, None, LocomotionUpdate::FlushActions))).unwrap();
        }
        pos_events.send(VehicleEntityEvent::new(MustSync::Server,VehiclePosEvent::new(self_id, None, VehiclePosUpdate::AddToEverySpeed(vehicle_spd_change, vehicle_turn_spd_change)))).unwrap();
    }
    pub fn compute_vehicle_physics(&self,
        self_id:usize,
        static_locomotion:&StaticLocomotion,
        static_type:&StaticVehicleEntity<CoolGameEngineTID>,
        world:&GameMap<CoolVoxel, Road>,
        vehicle_stats:&VehicleStats,
        vehicle_position:&VehiclePosition,
        collider_events:&Sender<VehicleEntityEvent<SimpleComponentEvent<CoolGameEngineTID, HullUpdate>>>,
        pos_events:&Sender<VehicleEntityEvent<VehiclePosEvent<CoolGameEngineTID>>>
    ) {
        let mut new_vehicle_spd = vehicle_position.spd;
        new_vehicle_spd *= AIR_RESISTANCE;
        let mut new_vehicle_turn_spd = vehicle_position.turn_spd;
        let mut total_ground_spd_add = Vec3Df::zero();
        let mut total_nonground_spd_add = Vec3Df::zero();
        let mut total_nonzero = 0;
        dbg!(new_vehicle_turn_spd);
        for eq in &self.equipment {
            let (spd_add, turn_spd_add, got_ground) = eq.collide_with_world(&static_locomotion.equipment[eq.static_equipment], world, vehicle_stats, vehicle_position);
            
            if got_ground {
                total_nonzero += 1;
                total_ground_spd_add += spd_add;
            }
            else {
                total_nonground_spd_add += spd_add;
            }
            new_vehicle_turn_spd += turn_spd_add;
        }
        if total_nonzero > 0 {
            total_ground_spd_add *= 1.0/total_nonzero as f32;
        }
        dbg!(new_vehicle_turn_spd);
        new_vehicle_spd += total_ground_spd_add + total_nonground_spd_add;
        new_vehicle_turn_spd.yaw *= AIR_RESISTANCE * TURN_RESISTANCE;
        new_vehicle_turn_spd.pitch *= AIR_RESISTANCE * TURN_RESISTANCE;
        new_vehicle_turn_spd.roll *= AIR_RESISTANCE * TURN_RESISTANCE;
        pos_events.send(VehicleEntityEvent::new(MustSync::Server, VehiclePosEvent::new(self_id, Some(CoolGameEngineTID::vehicles(self_id)), VehiclePosUpdate::UpdateEveryPos(vehicle_position.pos + new_vehicle_spd, vehicle_position.orientation + new_vehicle_turn_spd))));
        pos_events.send(VehicleEntityEvent::new(MustSync::Server, VehiclePosEvent::new(self_id, Some(CoolGameEngineTID::vehicles(self_id)), VehiclePosUpdate::UpdateEverySpeed(new_vehicle_spd, new_vehicle_turn_spd))));

        collider_events.send(VehicleEntityEvent::new(MustSync::Server,SimpleComponentEvent::new(self_id, None, HullUpdate::UpdateCollider(static_type.hull.base_collider.get_moved(vehicle_position.pos + new_vehicle_spd, vehicle_position.orientation + new_vehicle_turn_spd)))));
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
    pub static_equipment:usize,
    pub current_local_position:Vec3Df,
    current_local_speed:Vec3Df,
    pub current_local_orient:Orientation,
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
                        let ray = Ray::new(vehicle_rotation.rotate(self.current_local_position) + vehicle_position.pos, static_type.down_dir.map_or(Vec3Df::new(0.0, 0.0, -1.0), |vec| {vehicle_rotation.rotate(self_rotation.rotate(vec))}), Some(*to));
                        let end = ray.get_end(&world);
                        if end.final_length >= *from && end.final_length <= *to {
                            match world.get_type_of_voxel_at(get_voxel_pos(end.end - vehicle_position.pos)) {
                                Some(voxel_type) => possible = possible && voxel_type.surface_type == *surface_type,
                                None => possible = false
                            }
                        }
                    },
                    ActivationRequirement::HullPosition => todo!("Figure out where that is useful and how to implement it"),
                    ActivationRequirement::SurfaceContact(surface_type) => {

                        let self_rotation = Rotation::from_orientation(self.current_local_orient);
                        let vehicle_rotation = Rotation::from_orientation(vehicle_position.orientation);
                        let ray = Ray::new(vehicle_rotation.rotate(self.current_local_position) + vehicle_position.pos, static_type.down_dir.map_or(Vec3Df::new(0.0, 0.0, -1.0), |vec| {vehicle_rotation.rotate(self_rotation.rotate(vec))}), Some(100.0));
                        let end = ray.get_end(&world);
                        match world.get_type_of_voxel_at(get_voxel_pos(end.end)) {
                            Some(voxel_type) => {
                                if self.current_collider.rotate_around_origin(&vehicle_rotation).point_inside(end.end - vehicle_position.pos) {
                                    possible = possible && voxel_type.surface_type == *surface_type;
                                }
                                else {
                                    possible = false;
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
        motion_vector:Vec3Df,
        motion_application:MotionApplication
    ) -> (Vec3Df, Orientation, f32) {
        let vehicle_rotat = Rotation::from_orientation(vehicle_position.orientation);
        let against_vehicle_rotat = Rotation::from_orientation(Orientation::new(-vehicle_position.orientation.yaw, -vehicle_position.orientation.pitch, -vehicle_position.orientation.roll));
        let equip_rotat = Rotation::from_orientation(self.current_local_orient);
        let mut rotated_motion_vector = match motion_application {
            MotionApplication::EquipmentLocal => vehicle_rotat.rotate(equip_rotat.rotate(motion_vector * strength)),
            MotionApplication::FlatAlong2AxisFromEquipment { removed } => {
                let mut rotated = equip_rotat.rotate(motion_vector * strength);
                match removed {
                    Coord::X => rotated.x = 0.0,
                    Coord::Y => rotated.y = 0.0,
                    Coord::Z => rotated.z = 0.0
                }
                if rotated.norme() > 0.01 {
                    rotated = rotated.normalise();
                }
                rotated
            },
            MotionApplication::WorldCoords => {
                motion_vector * strength
            },
            MotionApplication::RotateAgainstVehicle => {
                against_vehicle_rotat.rotate(motion_vector * strength)
            }
        };
        rotated_motion_vector.zero_out_nans();
        match static_type.motion.application_point {
            ApplicationPoint::CenterOfEquipment => {
                let ap = self.current_local_position;
                let center_of_gravity = Vec3Df::zero();
                let lever = ap - center_of_gravity;
                let moment = lever.cross(&rotated_motion_vector) / vehicle_stats.mass;
                let orient_change = Orientation::new(moment.z, moment.y, moment.x);
                let normalised_dot = rotated_motion_vector.normalise().dot(&lever.normalise()).abs();
                let factor = if normalised_dot.is_nan() {0.00001} else {normalised_dot} / vehicle_stats.mass;
                let resulting_force = rotated_motion_vector * factor;
                
                (resulting_force, orient_change, factor)
            },
            ApplicationPoint::CenterOfGravity => {
                (rotated_motion_vector / vehicle_stats.mass, Orientation::zero(), 1.0)
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
                    let (spd_add, turn_spd_add, factor) = self_clone.compute_vehicle_speed_vector_and_turn_spd_change(static_type, world, vehicle_stats, vehicle_position, activated.strength, static_type.motion.forward_vector, static_type.motion.motion_application.clone());
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
        self_clone.current_local_turn_speed.yaw *= 0.6;
        self_clone.current_local_turn_speed.pitch *= 0.6;
        self_clone.current_local_turn_speed.roll *= 0.6;
        if vehicle_spd_add != Vec3Df::zero() {
            //dbg!(vehicle_spd_add);
            //dbg!(self_clone.current_local_turn_speed);
            //dbg!(self_clone.current_local_orient);
        }
        (self_clone, vehicle_spd_add, vehicle_turn_spd_add)
    }
    pub fn compute_on_ground_if_relevant(&self,
        static_type:&StaticLocomotionEquipment,
        world:&GameMap<CoolVoxel, Road>,
        vehicle_stats:&VehicleStats,
        vehicle_position:&VehiclePosition,
    ) -> Option<bool> {
        if let Some(surface_type) = &static_type.is_ground_equipment {
            let self_rotation = Rotation::from_orientation(self.current_local_orient);
            let vehicle_rotation = Rotation::from_orientation(vehicle_position.orientation);
            let ray = Ray::new(vehicle_rotation.rotate(self.current_local_position) + vehicle_position.pos, static_type.down_dir.map_or(Vec3Df::new(0.0, 0.0, -1.0), |vec| {vehicle_rotation.rotate(self_rotation.rotate(vec))}) , Some(100.0));
            let end = ray.get_end(&world);
            let mut on_ground = false;
            //dbg!(end.final_length);
            //dbg!(ray.get_start() - vehicle_position.pos);
            //dbg!(end.end - vehicle_position.pos);
            //dbg!(self.current_collider.rotate_around_origin(&vehicle_rotation));
            match world.get_type_of_voxel_at(get_voxel_pos(end.end)) {
                Some(voxel_type) => {
                    if self.current_collider.rotate_around_origin(&vehicle_rotation).point_inside(end.end - vehicle_position.pos) {
                        on_ground = voxel_type.surface_type == Some(surface_type.clone());
                    }
                },
                None => ()
            }
            Some(on_ground)
        }
        else {
            None
        }
    }

    pub fn collide_with_world(&self,
        static_type:&StaticLocomotionEquipment,
        world:&GameMap<CoolVoxel, Road>,
        vehicle_stats:&VehicleStats,
        vehicle_position:&VehiclePosition
    ) -> (Vec3Df, Orientation, bool) {

        if let Some(surface_type) = &static_type.is_ground_equipment {
            let vehicle_rotation = Rotation::from_orientation(vehicle_position.orientation);
            let current_world_pos = vehicle_position.pos + vehicle_rotation.rotate(self.current_local_position);
            let vehicle_next_rotation = Rotation::from_orientation(vehicle_position.orientation + vehicle_position.turn_spd);
            let next_world_pos = vehicle_position.pos + vehicle_next_rotation.rotate(self.current_local_position) + vehicle_position.spd;
            let diff_vector = next_world_pos - current_world_pos;
            let diff_len = diff_vector.norme();
            let ray = Curve::new(current_world_pos, vehicle_position.pos, diff_vector, vehicle_position.turn_spd);
            let end = ray.get_end(&world);
            if end.final_coef >= 1.0 {
                // Apply gravity because the ray didn't reach anything
                println!("APPLYING GRAVITY ON {}", self.static_equipment);
                let (spd_add, turn_spd_add, factor) = self.compute_vehicle_speed_vector_and_turn_spd_change(static_type, world, vehicle_stats, vehicle_position, GRAVITY, Vec3Df::new(0.0, 0.0, -1.0), MotionApplication::RotateAgainstVehicle);
                dbg!(turn_spd_add);
                (spd_add, turn_spd_add, false)
            }
            else {
                println!("NOT GRAVITY ON {} with ray len {} and diff len {}", self.static_equipment, end.final_coef, diff_len);
                match world.get_type_of_voxel_at(get_voxel_pos(end.end)) {
                    Some(voxel_type) => {
                        if self.current_collider.rotate_around_origin(&vehicle_rotation).point_inside(end.end - vehicle_position.pos) && voxel_type.surface_type == Some(surface_type.clone()) {
                            //let (speed_nudge, vertical, pos_nudge) = compute_nudges_from(end.end, Vec3Df::zero(), world, true);
                            let mut speed_nudge = diff_vector * -(1.0 - end.final_coef);
                            speed_nudge = get_minimum_nudge(end.end, speed_nudge, world);
                            let (spd_add, turn_spd_add, factor) = self.compute_vehicle_speed_vector_and_turn_spd_change(static_type, world, vehicle_stats, vehicle_position, speed_nudge.norme(), speed_nudge.normalise(), MotionApplication::RotateAgainstVehicle);
                            //let new_rotation = Rotation::from_orientation(vehicle_position.orientation + vehicle_position.turn_spd + turn_spd_add);
                            //let next_pos_old_turn = vehicle_position.pos + vehicle_next_rotation.rotate(self.current_local_position) + vehicle_position.spd + speed_nudge;
                            //let next_pos_new_turn = vehicle_position.pos + new_rotation.rotate(self.current_local_position) + vehicle_position.spd + speed_nudge;
                            //speed_nudge += next_pos_new_turn - next_pos_old_turn;
                            //speed_nudge = get_minimum_nudge(end.end, speed_nudge, world);
                            dbg!(speed_nudge, turn_spd_add, factor);
                            (speed_nudge, turn_spd_add, true)
                        }
                        else {
                            (Vec3Df::zero(), Orientation::zero(), false)
                        }
                    },
                    None => (Vec3Df::zero(), Orientation::zero(), false)
                }
            }
        }
        else {
            (Vec3Df::zero(), Orientation::zero(), false)
        }
        

    }

}

pub fn get_minimum_nudge(end:Vec3Df, nudge:Vec3Df, world:&GameMap<CoolVoxel, Road>) -> Vec3Df {
    let xy_zeroed_out = end + Vec3Df::new(0.0, 0.0, nudge.z);
    if world.is_voxel_solid(get_voxel_pos(xy_zeroed_out)) {
        let stepped_z_nudge = end + Vec3Df::new(0.0, 0.0, nudge.z + 1.0);
        if world.is_voxel_solid(get_voxel_pos(stepped_z_nudge)) {
            let x_zeroed = end + Vec3Df::new(0.0, nudge.y, nudge.z);
            let y_zeroed = end + Vec3Df::new(nudge.x, 0.0, nudge.z);
            let z_zeroed = end + Vec3Df::new(nudge.x, nudge.y, 0.0);
            if !world.is_voxel_solid(get_voxel_pos(x_zeroed)) {
                Vec3Df::new(0.0, nudge.y, nudge.z)
            }
            else if !world.is_voxel_solid(get_voxel_pos(y_zeroed)) {
                Vec3Df::new(nudge.x, 0.0, nudge.z)
            }
            else if !world.is_voxel_solid(get_voxel_pos(z_zeroed)) {
                Vec3Df::new(nudge.x, nudge.y, 0.0)
            }
            else {
                nudge
            }
        }
        else {
            Vec3Df::new(0.0, 0.0, nudge.z + 1.0)
        }
        
        
    }
    else {
        Vec3Df::new(0.0, 0.0, nudge.z)
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
    pub down_dir:Option<Vec3Df>,
    pub is_ground_equipment:Option<SurfaceType>
}

#[derive(Clone, Debug, ToBytes, FromBytes)]
pub enum EqMotionKind {
    Switch,
    AnalogLinear
}

#[derive(Clone, Debug, ToBytes, FromBytes)]
pub struct EqMotion {
    pub motion_application:MotionApplication,
    pub forward_vector:Vec3Df,
    pub kind:EqMotionKind,
    pub application_point:ApplicationPoint
}

#[derive(Clone, ToBytes, FromBytes)]
pub enum MotionApplication {
    EquipmentLocal,
    FlatAlong2AxisFromEquipment {removed:Coord},
    WorldCoords,
    RotateAgainstVehicle
}

impl Debug for MotionApplication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("type : {}", match self {MotionApplication::EquipmentLocal => "EquipmentLocal", _ => "FlatAlong2Axis"}))
    }
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