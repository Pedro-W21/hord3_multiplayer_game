use hord3::horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::driver::colliders::{ComplexCollider, AABB};

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct Hull {
    pub complex_collider:ComplexCollider
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StaticHull {
    pub base_collider:ComplexCollider,
}

impl StaticComponent for StaticHull {
    
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum HullUpdate {
    UpdateCollider(ComplexCollider),
    UpdateColliderAABB(AABB),
}

impl<ID:Identify> SimpleComponentUpdate<Hull, ID> for HullUpdate {
    fn apply_to_comp(self, component:&mut Hull) {
        match self {
            HullUpdate::UpdateCollider(collider) => component.complex_collider = collider,
            HullUpdate::UpdateColliderAABB(aabb) => component.complex_collider.set_aabb(aabb),
        }
    }
}

impl<ID:Identify> Component<ID> for Hull {
    type CE = SimpleComponentEvent<ID, HullUpdate>;
    type SC = StaticHull;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { complex_collider: static_comp.base_collider.clone() }
    }
}