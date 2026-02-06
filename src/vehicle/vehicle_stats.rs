use hord3::horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify, static_type_id::HasStaticTypeID};
use to_from_bytes_derive::{FromBytes, ToBytes};

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct VehicleStats {
    pub static_id:usize,
    pub nitro_left:f32,
    pub mass:f32,
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StaticVehicleStats {
    pub start_mass:f32,
    pub max_nitro:f32,
}

impl StaticComponent for StaticVehicleStats {
    
}

impl HasStaticTypeID for VehicleStats {
    fn get_id(&self) -> usize {
        self.static_id
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum VehicleStatsUpdate {
    UpdateNitro(f32),
    UpdateMass(f32),
}

impl<ID:Identify> SimpleComponentUpdate<VehicleStats, ID> for VehicleStatsUpdate {
    fn apply_to_comp(self, component:&mut VehicleStats) {
        match self {
            VehicleStatsUpdate::UpdateNitro(new_nitro) => component.nitro_left = new_nitro,
            VehicleStatsUpdate::UpdateMass(new_mass) => component.mass = new_mass,
        }
    }
}

impl<ID:Identify> Component<ID> for VehicleStats {
    type CE = SimpleComponentEvent<ID, VehicleStatsUpdate>;
    type SC = StaticVehicleStats;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { static_id: 0, nitro_left: 0.0, mass:static_comp.start_mass }
    }
}