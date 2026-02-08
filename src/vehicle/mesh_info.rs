use hord3::{defaults::default_rendering::vectorinator_binned::meshes::{Mesh, MeshID}, horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify}};
use to_from_bytes_derive::{FromBytes, ToBytes};


#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct VehicleMeshInfo {
    pub instance_id:Option<usize>,
    pub loco_instances_ids:Option<Vec<usize>>
}

#[derive(Clone)]
pub struct StaticVMeshInfo {
    pub mesh_data:Mesh,
    pub mesh_id:MeshID,
    pub eq_mesh_data:Vec<Mesh>,
    pub eq_mesh_ids:Vec<MeshID>
}

impl StaticComponent for StaticVMeshInfo {
    
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum VMeshInfoUpdate {
    UpdateInstance(Option<usize>),
}

impl<ID:Identify> SimpleComponentUpdate<VehicleMeshInfo, ID> for VMeshInfoUpdate {
    fn apply_to_comp(self, component:&mut VehicleMeshInfo) {
        match self {
            VMeshInfoUpdate::UpdateInstance(instance) => component.instance_id = instance
        }
    }
}

impl<ID:Identify> Component<ID> for VehicleMeshInfo {
    type SC = StaticVMeshInfo;
    type CE = SimpleComponentEvent<ID, VMeshInfoUpdate>;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { instance_id: None, loco_instances_ids:None }
    }
}