use hord3::horde::game_engine::{entity::{Component, ComponentEvent, EntityID, StaticComponent}, multiplayer::Identify, static_type_id::HasStaticTypeID};
use to_from_bytes_derive::{FromBytes, ToBytes};


#[derive(Clone, PartialEq, ToBytes, FromBytes)]
pub struct Stats {
    pub static_type_id:usize,
    pub health:i32,
    pub damage:i32,
    pub stamina:i32,
    pub ground_speed:f32,
    pub jump_height:f32,
    pub personal_vehicle:Option<usize>,
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StatEvent<ID:Identify> {
    id:usize,
    source:Option<ID>,
    variant:StatEventVariant
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum StatEventVariant {
    UpdateHealth(i32),
    UpdateDamage(i32),
    UpdateStamina(i32)
}

#[derive(Clone)]
pub struct StaticStats {
    
}

impl StaticComponent for StaticStats {

}

impl<ID:Identify> ComponentEvent<Stats, ID> for StatEvent<ID> {
    type ComponentUpdate = StatEventVariant;
    fn get_id(&self) -> EntityID {
        self.id
    }
    fn get_source(&self) -> Option<ID> {
        self.source.clone()
    }
    fn apply_to_component(self, components:&mut Vec<Stats>) {
        match self.variant {
            StatEventVariant::UpdateDamage(new_dmg) => components[self.id].damage = new_dmg,
            StatEventVariant::UpdateHealth(new_health) => components[self.id].health = new_health,
            StatEventVariant::UpdateStamina(new_stam) => components[self.id].stamina = new_stam,
        }
    }
}

impl<ID:Identify> Component<ID> for Stats {
    type CE = StatEvent<ID>;
    type SC = StaticStats;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { static_type_id: 0, health: 0, damage: 0, stamina: 0, jump_height:1.0, ground_speed:0.2, personal_vehicle:None }
    }
}

impl HasStaticTypeID for Stats {
    fn get_id(&self) -> usize {
        self.static_type_id
    }
}