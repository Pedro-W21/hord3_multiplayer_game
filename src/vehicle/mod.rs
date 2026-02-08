use entity_derive::Entity;
use hord3::{defaults::default_rendering::vectorinator_binned::{Vectorinator, VectorinatorWrite, meshes::MeshInstance}, horde::{game_engine::{entity::{Component, ComponentEvent, EVecStopsIn, EVecStopsOut, Entity, EntityID, EntityVec, MultiplayerEntity, NewEntity, StaticEntity}, multiplayer::{Identify, MustSync}, position::EntityPosition, static_type_id::HasStaticTypeID}, geometry::{rotation::Rotation, vec3d::Vec3Df}}};

use crate::{cutscene::game_shader::GameShader, vehicle::{hull::Hull, locomotion::Locomotion, mesh_info::VehicleMeshInfo, position::VehiclePosition, vehicle_stats::VehicleStats}};

pub mod locomotion;
pub mod position;
pub mod hull;
pub mod vehicle_stats;
pub mod mesh_info;
pub mod default_vehicles;

pub fn test() {
    
}

impl<ID:Identify> NewEntity<VehicleEntity,ID> for NewVehicleEntity<ID> {
    fn get_ent(self, static_type:&<VehicleEntity as Entity<ID>>::SE) -> VehicleEntity {
        let mut hull = <Hull as Component<ID>>::from_static(&static_type.hull);
        hull.complex_collider = hull.complex_collider.get_moved(self.position.pos, self.position.orientation);
        VehicleEntity { position: self.position, stats: self.stats, mesh_info: <VehicleMeshInfo as Component<ID>>::from_static(&static_type.mesh_info), hull, locomotion:<Locomotion as Component<ID>>::from_static(&static_type.locomotion)  }
    }
}

impl<'a, ID:Identify> RenderVehicleEntity<VectorinatorWrite<'a>, ID> for VehicleEntity {
    fn do_render_changes(rendering_data: &mut VectorinatorWrite<'a>,position: &mut VehiclePosition,stats: &mut VehicleStats,mesh_info: &mut VehicleMeshInfo, locomotion:&mut Locomotion,static_type: &StaticVehicleEntity<ID>) {
        
        match mesh_info.instance_id {
            Some(id) => {
                let mut instance = rendering_data.meshes.instances[2].get_instance_mut(id);
                
                instance.change_pos(position.pos);
                instance.change_orient(position.orientation);
            },
            None => {
                if !rendering_data.meshes.does_mesh_exist(&static_type.mesh_info.mesh_id) {
                    rendering_data.meshes.add_mesh(static_type.mesh_info.mesh_data.clone());
                }
                mesh_info.instance_id = Some(rendering_data.meshes.add_instance(MeshInstance::new(position.pos, position.orientation, static_type.mesh_info.mesh_id.clone(), true, false, false), 2))
            }
        }
        match &mesh_info.loco_instances_ids {
            Some(ids) => {
                for (i, eq) in locomotion.equipment.iter().enumerate() {
                    let mut instance = rendering_data.meshes.instances[2].get_instance_mut(ids[i]);
                    let rotation = Rotation::from_orientation(position.orientation);
                    instance.change_pos(position.pos + rotation.rotate(eq.current_local_position));
                    instance.change_orient(position.orientation + eq.current_local_orient);
                }
            },
            None => {
                let mut ids = Vec::with_capacity(static_type.locomotion.equipment.len());
                for (i, eq) in locomotion.equipment.iter().enumerate() {
                    if !rendering_data.meshes.does_mesh_exist(&static_type.mesh_info.eq_mesh_ids[i]) {
                        rendering_data.meshes.add_mesh(static_type.mesh_info.eq_mesh_data[i].clone());
                    }
                    let rotation = Rotation::from_orientation(position.orientation);
                    ids.push(rendering_data.meshes.add_instance(MeshInstance::new(position.pos + rotation.rotate(eq.current_local_position), position.orientation + eq.current_local_orient, static_type.mesh_info.eq_mesh_ids[i].clone(), true, false, false), 2))
                }
                mesh_info.loco_instances_ids = Some(ids)
            }
        }
    }
}

#[derive(Entity, Clone)]
pub struct VehicleEntity {
    #[position]
    #[used_in_render]
    #[used_in_new]
    #[must_sync]
    position:VehiclePosition,
    #[static_id]
    #[used_in_new]
    #[must_sync]
    stats:VehicleStats,
    #[used_in_render]
    mesh_info:VehicleMeshInfo,
    hull:Hull,
    #[used_in_render]
    #[must_sync]
    locomotion:Locomotion
}