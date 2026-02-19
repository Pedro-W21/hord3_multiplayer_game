use std::{collections::HashMap, f32::consts::PI, io::{self, Read}, sync::Arc};

use hord3::{defaults::default_rendering::vectorinator_binned::meshes::{Mesh, MeshID, MeshLODS, MeshLODType}, horde::geometry::{rotation::Orientation, vec3d::{Coord, Vec3Df}}};

use crate::{driver::colliders::{AABB, BoundingCollider, BoundingSphere, ComplexCollider, InternalCollider, SubCollider}, game_3d_models::simple_prism, game_engine::CoolGameEngineTID, vehicle::{StaticVehicleEntity, hull::{Hull, StaticHull}, locomotion::{ActivationOutput, ActivationRequirement, ActivationRequirements, ApplicationPoint, DriverAction, EqMotion, EqMotionKind, EqRecoil, EqRecoilKind, MotionApplication, StaticLocomotion, StaticLocomotionEquipment, SurfaceType}, mesh_info::StaticVMeshInfo, position::StaticVehiclePos, vehicle_stats::StaticVehicleStats}};

pub fn get_default_car_type() -> StaticVehicleEntity<CoolGameEngineTID> {
    let aabb = AABB::new(
                    Vec3Df::new(2.0, 1.0, 1.0),
                    
                    -Vec3Df::new(2.0, 1.0, 0.0),
                );
    let wheel_aabb = AABB::new(
                    Vec3Df::new(1.0, 0.75, 0.75),
                    
                    -Vec3Df::new(0.75, 0.75, 0.75),
                );
    let mut loco_equipments = get_default_loco_equips(aabb);
    StaticVehicleEntity {
        position: StaticVehiclePos {},
        stats: StaticVehicleStats {
            start_mass: 1.0,
            max_nitro:100.0
        },
        mesh_info: StaticVMeshInfo {
            mesh_id:MeshID::Named(String::from("default_car")),
            mesh_data:Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(simple_prism(aabb.get_first_point(), aabb.get_second_point(), 3, (255,255,255))))]), "default_car".to_string(), aabb.get_first_point().dist(&aabb.get_second_point())),
            eq_mesh_ids:vec![
                MeshID::Named(String::from("default_car_front_wheel")),
                MeshID::Named(String::from("default_car_back_wheel")),
                MeshID::Named(String::from("default_car_back_wheel")),
                MeshID::Named(String::from("default_car_front_wheel")),
            ],
            eq_mesh_data:vec![
                Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(simple_prism(wheel_aabb.get_first_point(), wheel_aabb.get_second_point(), 4, (255,255,255))))]), "default_car_front_wheel".to_string(), wheel_aabb.get_first_point().dist(&wheel_aabb.get_second_point())),
                Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(simple_prism(wheel_aabb.get_first_point(), wheel_aabb.get_second_point(), 5, (255,255,255))))]), "default_car_back_wheel".to_string(), wheel_aabb.get_first_point().dist(&wheel_aabb.get_second_point())),
                Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(simple_prism(wheel_aabb.get_first_point(), wheel_aabb.get_second_point(), 5, (255,255,255))))]), "default_car_back_wheel".to_string(), wheel_aabb.get_first_point().dist(&wheel_aabb.get_second_point())),
                Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(simple_prism(wheel_aabb.get_first_point(), wheel_aabb.get_second_point(), 4, (255,255,255))))]), "default_car_front_wheel".to_string(), wheel_aabb.get_first_point().dist(&wheel_aabb.get_second_point())),
            ]
        },
        hull: StaticHull {
            base_collider:ComplexCollider::new(
                aabb.clone(),
                vec![
                    SubCollider::new(
                        BoundingCollider::AABB(aabb.clone()),
                        vec![
                            InternalCollider::AABB(aabb.clone())
                        ]
                    )
                ]
            )
        },
        locomotion: StaticLocomotion {
            equipment:loco_equipments
        }
    }
}

fn get_default_loco_equips(aabb:AABB) -> Vec<StaticLocomotionEquipment> {
    let c:char = '1';
    let first_equipment = StaticLocomotionEquipment {
        activation_requirements:vec![
            ActivationRequirements::new(
                vec![
                    ActivationRequirement::SurfaceContact(SurfaceType::Ground),
                    ActivationRequirement::DriverAction(DriverAction::Throttle { from: -1.0, to: 1.0 })
                ],
                ActivationOutput::ActivateMotion,
            ),
            ActivationRequirements::new(
                vec![
                    ActivationRequirement::SurfaceContact(SurfaceType::Ground),
                    ActivationRequirement::DriverAction(DriverAction::HorizontalReorientation { from: -1.0, to: 1.0 })
                ],
                ActivationOutput::Turn(Coord::Z),
            ),
        ],
        resting_local_position:aabb.get_ground_vertices()[0],
        max_self_rotation:Orientation::new(PI/3.0, 0.0, 0.0),
        motion:EqMotion {
            kind:EqMotionKind::Switch,
            forward_vector:Vec3Df::new(1.0, 0.0, 0.0),
            motion_application:MotionApplication::FlatAlong2AxisFromEquipment { removed: Coord::Z },
            application_point:ApplicationPoint::CenterOfEquipment,   
        },
        recoil:EqRecoil {

            kind:EqRecoilKind::InstantOnActivation,
            equipment_local_vector_towards_recoil:Vec3Df::new(0.0, 0.0, 0.0),
            max_recoil:1.0
        },
        collider:BoundingCollider::BS(BoundingSphere::new(aabb.get_ground_vertices()[0], 0.5)),
        down_dir:None,
        is_ground_equipment:Some(SurfaceType::Ground),
        drag_coefficients:HashMap::new(),
    };
    let mut equipments = Vec::with_capacity(4);
    let turning_wheels = [1, 2];
    for i in 0..4 {
        let mut eq_clone = first_equipment.clone();
        eq_clone.resting_local_position = aabb.get_ground_vertices()[i];
        if !turning_wheels.contains(&i) {
            eq_clone.activation_requirements.remove(1);
            eq_clone.activation_requirements.remove(0);
        }
        eq_clone.collider = BoundingCollider::BS(BoundingSphere::new(aabb.get_ground_vertices()[i], 0.35));
        equipments.push(eq_clone);
    }
    equipments
}