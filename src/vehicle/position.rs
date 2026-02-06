use hord3::horde::{game_engine::{entity::{Component, ComponentEvent, StaticComponent}, multiplayer::Identify, position::EntityPosition}, geometry::{rotation::Orientation, vec3d::Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct VehiclePosition {
    pub pos:Vec3Df,
    pub spd:Vec3Df,
    pub orientation:Orientation,
    pub turn_spd:Orientation,
}
impl VehiclePosition {
    pub fn new() -> Self {
        Self { pos: Vec3Df::zero(), spd: Vec3Df::zero(), orientation: Orientation::zero(), turn_spd: Orientation::zero() }
    }
    pub fn with_pos(mut self, pos:Vec3Df) -> Self {
        self.pos = pos;
        self
    }
}
#[derive(Clone)]
pub struct StaticVehiclePos {

}

impl StaticComponent for StaticVehiclePos {

}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct VehiclePosEvent<ID:Identify> {
    id:usize,
    source:Option<ID>,
    update:VehiclePosUpdate
}

impl<ID:Identify> VehiclePosEvent<ID> {
    pub fn new(id:usize, source:Option<ID>, update:VehiclePosUpdate) -> Self {
        Self { id, source, update }
    }
}

impl<ID:Identify> Component<ID> for VehiclePosition {
    type SC = StaticVehiclePos;
    type CE = VehiclePosEvent<ID>;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { pos: Vec3Df::zero(), spd: Vec3Df::zero(), orientation: Orientation::zero(), turn_spd: Orientation::zero() }
    }
}
#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum VehiclePosUpdate {
    UpdateEverySpeed(Vec3Df, Orientation),
    AddToEverySpeed(Vec3Df, Orientation),
    UpdateEveryPos(Vec3Df, Orientation)
}   

impl<ID:Identify> ComponentEvent<VehiclePosition, ID> for VehiclePosEvent<ID> {
    type ComponentUpdate = VehiclePosUpdate;
    fn get_source(&self) -> Option<ID> {
        self.source.clone()
    }
    fn get_id(&self) -> hord3::horde::game_engine::entity::EntityID {
        self.id
    }
    fn apply_to_component(self, components:&mut Vec<VehiclePosition>) {
        match self.update {
            VehiclePosUpdate::UpdateEverySpeed(spd, turn_spd) => {
                components[self.id].spd = spd;
                components[self.id].turn_spd = turn_spd;
            },
            VehiclePosUpdate::AddToEverySpeed(spd, turn_spd) => {
                components[self.id].spd += spd;
                components[self.id].turn_spd += turn_spd;
            },
            VehiclePosUpdate::UpdateEveryPos(pos, orientation) => {
                components[self.id].pos = pos;
                components[self.id].orientation = orientation;
            }
        }
    }
}

impl<ID:Identify> EntityPosition<ID> for VehiclePosition {
    fn get_orientation(&self) -> Orientation {
        self.orientation
    }
    fn get_pos(&self) -> Vec3Df {
        self.pos
    }
    fn get_rotation(&self) -> Option<&hord3::horde::geometry::rotation::Rotation> {
        None
    }
}
