use hord3::horde::{game_engine::{entity::{Component, ComponentEvent, StaticComponent}, multiplayer::{Identify, MustSync}, world::{WorldComputeHandler, WorldEvent}}, geometry::vec3d::Vec3Df};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{driver::{GameEntityEvent, GameEntityVecRead, MovementEvent, MovementEventVariant, director::{Director, DirectorEvent, DirectorUpdate}, planner::{Plan, PlannerEvent, PlannerUpdate}}, game_engine::{CoolGameEngineTID, CoolVoxel}, game_map::{GameMap, GameMapEvent, VoxelLight, VoxelType, WorldVoxelPos, get_voxel_pos, road::Road}, vehicle::{VehicleEntityEvent, VehicleEntityVecRead, locomotion::{DriverAction, LocomotionEvent, LocomotionUpdate}}};

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub struct Action {
    id:usize,
    started_at:usize, // tick this was started at
    timer:ActionTimer,
    kind:ActionKind,
    source:ActionSource
}
#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum ActionSource {
    Director,
    Planner
}

impl Action {
    pub fn new(id:usize, started_at:usize, timer:ActionTimer, kind:ActionKind, source:ActionSource) -> Self {
        Self { id, started_at, timer, kind, source }
    }
    pub fn get_kind(&self) -> &ActionKind {
        &self.kind
    }
    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn needs_planning(&self) -> bool {
        match &self.kind {
            ActionKind::PathToPosition(pos, tolerance) => true,
            _ => false
        }
    }
    pub fn is_possible<'a>(
        &self,
        agent_id:usize,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&VehicleEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel, Road>, CoolGameEngineTID>,
        tick:usize
    ) -> bool {
        match self.kind {
            ActionKind::Jump => true,
            ActionKind::MoveInDirection(_) => true,
            ActionKind::MoveTowards(position, tolerance) => world.world.get_voxel_at(get_voxel_pos(position)).is_some(),
            ActionKind::PathToPosition(position, tolerance) => world.world.get_voxel_at(get_voxel_pos(position)).and_then(|voxel| {if world.world.get_voxel_types()[voxel.voxel_type as usize].is_completely_empty() {Some(true)} else {None}}).is_some(),
            ActionKind::ChangeVoxel(position, _) => world.world.get_chunk_at(world.world.get_chunk_pos_i(position)).is_some(),
            ActionKind::StopAt(pos, _, _) => true,
            _ => true
        }
    }
    pub fn is_done<'a>(
        &self,
        agent_id:usize,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&VehicleEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel, Road>, CoolGameEngineTID>,
        tick:usize
    ) -> bool {
        match self.kind {
            ActionKind::Jump => false,
            ActionKind::MoveInDirection(_) => false,
            ActionKind::MoveTowards(position, tolerance) => first_ent.movement[agent_id].pos.dist(&position) < tolerance,
            ActionKind::PathToPosition(position, tolerance) => first_ent.movement[agent_id].pos.dist(&position) < tolerance,
            ActionKind::ChangeVoxel(_, _) => false,
            ActionKind::StopAt(pos, speed_tolerance, pos_tolerance) => {
                let movement = &first_ent.movement[agent_id];
                movement.pos.dist(&pos) < pos_tolerance && movement.speed.norme() < speed_tolerance
            },
            _ => false
        }
    }
    pub fn perform<'a>(
        &self,
        agent_id:usize,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&VehicleEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel, Road>, CoolGameEngineTID>,
        counter:&mut ActionCounter,
        tick:usize
    ) -> ActionResult {
        if self.timer.timed_out(self.started_at, tick) {
            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent { id: agent_id, source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
            ActionResult::FailedTimer
        }
        else if self.is_possible(agent_id, first_ent, second_ent, world, tick) {
            if self.is_done(agent_id, first_ent, second_ent, world, tick) {
                first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent { id: agent_id, source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
                ActionResult::Done
            }
            else if self.needs_planning() {
                let planner = &first_ent.planner[agent_id];
                if planner.plan_exists_for(self.id) {
                    let plan = planner.get_plan_for_id(self.id).unwrap();
                    match plan.get_actions_to_add(counter, tick) {
                        Some(actions) => for action in actions.iter().rev() {
                            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent::new(agent_id, None, ActionsUpdate::InsertActionAtStart(action.clone()))));
                        },
                        None => ()
                    }
                    ActionResult::InProgress
                }
                else {
                    match self.kind {
                        ActionKind::PathToPosition(position, tolerance) => {
                            let movement = &first_ent.movement[agent_id];
                            let plan = Plan::create_pathfinding(self.id, tolerance, movement.pos, position, agent_id, 1000, first_ent, second_ent, world);
                            let actions = plan.get_actions_to_add(counter, tick);
                            match actions {
                                Some(actions) => for action in actions.iter().rev() {
                                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent::new(agent_id, None, ActionsUpdate::InsertActionAtStart(action.clone()))));
                                },
                                None => ()
                            }
                            first_ent.tunnels.planner_out.send(GameEntityEvent::new(MustSync::Server,PlannerEvent::new(agent_id, None, PlannerUpdate::AddPlan(plan))));

                            ActionResult::InProgress
                        },
                        _ => panic!("impossible to get there with an action that requires no planning"),
                    }
                }
                
            }
            else {
                match &self.kind {
                    ActionKind::Jump => {
                        let movement = &first_ent.movement[agent_id];
                        let stats = &first_ent.stats[agent_id];
                        if movement.touching_ground {
                            first_ent.tunnels.movement_out.send(GameEntityEvent::new(MustSync::Server,MovementEvent::new(agent_id, None, MovementEventVariant::AddToSpeed(Vec3Df::new(0.0, 0.0, stats.jump_height)))));
                            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent { id: agent_id, source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
                            ActionResult::Done
                        }
                        else {
                            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent { id: agent_id, source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
                            ActionResult::Error(ActionError::ImpossibleAction)
                        }
                    },
                    ActionKind::MoveInDirection(direction) => {
                        let movement = &first_ent.movement[agent_id];
                        let stats = &first_ent.stats[agent_id];
                        if direction.z != 0.0 && movement.against_wall && movement.touching_ground {
                            first_ent.tunnels.movement_out.send(GameEntityEvent::new(MustSync::Server,MovementEvent::new(agent_id, None, MovementEventVariant::AddToSpeed(Vec3Df::new(direction.x * stats.ground_speed, direction.y * stats.ground_speed, stats.jump_height)))));
                        }
                        else {
                            first_ent.tunnels.movement_out.send(GameEntityEvent::new(MustSync::Server,MovementEvent::new(agent_id, None, MovementEventVariant::AddToSpeed(Vec3Df::new(direction.x * stats.ground_speed, direction.y * stats.ground_speed, 0.0)))));
                        }
                        first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent::new(agent_id, None, ActionsUpdate::RemoveAction(self.id))));
                        ActionResult::InProgress
                    },
                    ActionKind::MoveTowards(position, tolerance) => {
                        let movement = &first_ent.movement[agent_id];
                        let stats = &first_ent.stats[agent_id];
                        let mut direction = position - movement.pos;
                        direction.z = 0.0;
                        direction = direction.normalise();
                        if movement.against_wall {
                            first_ent.tunnels.movement_out.send(GameEntityEvent::new(MustSync::Server,MovementEvent::new(agent_id, None, MovementEventVariant::AddToSpeed(Vec3Df::new(direction.x * stats.ground_speed, direction.y * stats.ground_speed, stats.jump_height)))));
                        }
                        else {
                            first_ent.tunnels.movement_out.send(GameEntityEvent::new(MustSync::Server,MovementEvent::new(agent_id, None, MovementEventVariant::AddToSpeed(Vec3Df::new(direction.x * stats.ground_speed, direction.y * stats.ground_speed, 0.0)))));
                        }
                        first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent::new(agent_id, None, ActionsUpdate::RemoveAction(self.id))));
                        ActionResult::InProgress
                    },
                    ActionKind::StopAt(pos, spd_tolerance, pos_tolerance) => {
                        
                        let movement = &first_ent.movement[agent_id];
                        let dist = movement.pos.dist(&pos);
                        if dist >= *pos_tolerance {
                            let mut direction = pos - movement.pos;
                            let stats = &first_ent.stats[agent_id];
                            direction.z = 0.0;
                            direction = direction.normalise();
                            if movement.against_wall {
                                first_ent.tunnels.movement_out.send(GameEntityEvent::new(MustSync::Server,MovementEvent::new(agent_id, None, MovementEventVariant::AddToSpeed(Vec3Df::new(direction.x * stats.ground_speed * 0.2, direction.y * stats.ground_speed * 0.2, stats.jump_height)))));
                            }
                            else {
                                first_ent.tunnels.movement_out.send(GameEntityEvent::new(MustSync::Server,MovementEvent::new(agent_id, None, MovementEventVariant::AddToSpeed(Vec3Df::new(direction.x * stats.ground_speed * 0.2, direction.y * stats.ground_speed * 0.2, 0.0)))));
                            }
                        }
                        else {
                            let mut direction = -movement.speed;
                            let stats = &first_ent.stats[agent_id];
                            direction.z = 0.0;
                            direction = direction.normalise();
                            first_ent.tunnels.movement_out.send(GameEntityEvent::new(MustSync::Server,MovementEvent::new(agent_id, None, MovementEventVariant::AddToSpeed(Vec3Df::new(direction.x * stats.ground_speed * 0.5, direction.y * stats.ground_speed * 0.5, 0.0)))));
                        }

                        ActionResult::InProgress
                    }
                    ActionKind::ChangeVoxel(voxel_pos, new_voxel) => {
                        world.tunnels.send_event(GameMapEvent::UpdateVoxelAt(voxel_pos.clone(), new_voxel.clone()));
                        first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent::new(agent_id, None, ActionsUpdate::RemoveAction(self.id))));
                        ActionResult::Done
                    },
                    ActionKind::PathToPosition(position, tolerance) => ActionResult::Error(ActionError::ImpossibleAction),
                    other => {
                        let stats = &first_ent.stats[agent_id];
                        first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent { id: agent_id, source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
                        match stats.personal_vehicle {
                            Some(vehicle) => {
                                dbg!(other.clone());
                                second_ent.tunnels.locomotion_out.send(VehicleEntityEvent::new(MustSync::Server, LocomotionEvent::new(vehicle, Some(CoolGameEngineTID::entity_1(agent_id)), LocomotionUpdate::AddAction(self.clone(), ActionResult::Done)))).unwrap();
                                ActionResult::Done
                            },
                            None => ActionResult::Done
                        }
                    }
                }
            }
        }
        else {
            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Server,ActionsEvent { id: agent_id, source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
            ActionResult::Error(ActionError::ImpossibleAction)
        }
    }

}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum ActionTimer {
    Infinite,
    Delay(usize), // ticks
    Deadline(usize) // must be done by specified tick
}

impl ActionTimer {
    pub fn timed_out(&self, started:usize, current_tick:usize) -> bool {
        match self {
            ActionTimer::Infinite => false,
            ActionTimer::Delay(delay) => current_tick - started >= *delay,
            ActionTimer::Deadline(deadline) => current_tick >= *deadline,

        }
    }
}

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub enum ActionResult {
    Done,
    InProgress,
    FailedTimer,
    Error(ActionError)
}

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub enum ActionError {
    ImpossibleAction,
    StartedAfterDeadline
}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum ActionKind {
    MoveInDirection(Vec3Df),
    Jump,
    PathToPosition(Vec3Df, f32),
    MoveTowards(Vec3Df, f32),
    StopAt(Vec3Df, f32, f32),
    ChangeVoxel(WorldVoxelPos, CoolVoxel),
    DriverAction(DriverAction),
    Turn(f32),
    Throttle(f32),
    ActivateNitro
}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub struct ActionCounter {
    latest_id:usize,
}

impl ActionCounter {
    pub fn new() -> Self {
        Self { latest_id: 0 }
    }
    pub fn get_next_id(&mut self) -> usize {
        let id = self.latest_id;
        self.latest_id += 1;
        id
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct Actions {
    action_counter:ActionCounter,
    all_actions:Vec<Action>
}

impl Actions {
    pub fn get_counter(&self) -> &ActionCounter {
        &self.action_counter
    }
    pub fn new() -> Self {
        Self { action_counter: ActionCounter::new(), all_actions: Vec::with_capacity(8) }
    }
    pub fn perform<'a>(
        &self,
        agent_id:usize,
        first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>,
        second_ent:&VehicleEntityVecRead<'a, CoolGameEngineTID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel, Road>, CoolGameEngineTID>,
        counter:&mut ActionCounter,
        tick:usize
    ) {
        if self.all_actions.len() > 0 {
            let action = &self.all_actions[0];
            let result = action.perform(agent_id, first_ent, second_ent, world, counter, tick);
            match &result {
                ActionResult::InProgress => (),
                _ => match action.source {
                    ActionSource::Planner => first_ent.tunnels.planner_out.send(GameEntityEvent::new(MustSync::Server,PlannerEvent::new(agent_id, None, PlannerUpdate::AddFinished((action.clone(), result))))).unwrap(),
                    ActionSource::Director => first_ent.tunnels.director_out.send(GameEntityEvent::new(MustSync::Server,DirectorEvent::new(agent_id, None, DirectorUpdate::NotifyFinished((action.clone(), result))))).unwrap(),
                }
            }
        }
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StaticGameActions {
    pub base_actions:Vec<Action>
}

impl StaticComponent for StaticGameActions {
    
}
#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct ActionsEvent<ID:Identify> {
    id:usize,
    source:Option<ID>,
    variant:ActionsUpdate
}

impl<ID:Identify> ActionsEvent<ID> {
    pub fn new(id:usize, source:Option<ID>, variant:ActionsUpdate) -> Self {
        Self { id, source, variant }
    }
}

impl<ID:Identify> ComponentEvent<Actions, ID> for ActionsEvent<ID> {
    type ComponentUpdate = ActionsUpdate;
    fn get_id(&self) -> hord3::horde::game_engine::entity::EntityID {
        self.id
    }
    fn get_source(&self) -> Option<ID> {
        self.source.clone()
    }
    fn apply_to_component(self, components:&mut Vec<Actions>) {
        match self.variant {
            ActionsUpdate::UpdateCounter(counter) => components[self.id].action_counter = counter,
            ActionsUpdate::AddAction(action) => components[self.id].all_actions.push(action),
            ActionsUpdate::RemoveAction(action_id) => {
                match components[self.id].all_actions.iter().enumerate().find(|(i, action)| {action.id == action_id}) {
                    Some((i, action)) => {
                        components[self.id].all_actions.remove(i);
                    },
                    None => ()
                }
            },
            ActionsUpdate::InsertActionAtStart(action) => components[self.id].all_actions.insert(0,action),
            ActionsUpdate::UpdateAllActions(new_actions) => components[self.id].all_actions = new_actions,
        }
    }  
}
#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum ActionsUpdate {
    UpdateCounter(ActionCounter),
    AddAction(Action),
    InsertActionAtStart(Action),
    UpdateAllActions(Vec<Action>),
    RemoveAction(usize) // action id
}

impl<ID:Identify> Component<ID> for Actions {
    type SC = StaticGameActions;
    type CE = ActionsEvent<ID>;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { action_counter: ActionCounter::new(), all_actions:static_comp.base_actions.clone() }
    }
}