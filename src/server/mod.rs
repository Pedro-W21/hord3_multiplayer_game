
use std::{collections::HashMap, f32::consts::PI, net::Ipv4Addr, path::PathBuf, simd::Simd, sync::{atomic::{AtomicUsize, Ordering}, mpmc::{self, channel}, Arc, RwLock}, thread, time::{Duration, Instant}};

use crate::{driver::{colliders::AABB, stats::{StaticStats, Stats}}, game_map::{GameMapEvent, WorldChunkPos, road::Road}, server::server_tasks::GameUserEvent, vehicle::{NewVehicleEntity, VehicleEntityVec, default_vehicles::default_car::get_default_car_type, position::VehiclePosition, vehicle_stats::VehicleStats}};
use cosmic_text::{Color, Font, Metrics};
use crate::cutscene::{camera_movement::{CameraMovement, CameraMovementDuration, CameraMovementElement, CameraSequence}, demo_cutscene::{get_demo_cutscene, get_empty_cutscene}, game_shader::GameShader, real_demo_cutscene::get_real_demo_cutscene, write_in_the_air::get_positions_of_air_written_text, written_texture::get_written_texture_buffer};
use crate::day_night::DayNight;
use crate::game_3d_models::{clustered_ent_mesh, grey_sphere_mesh, lit_selection_cube, second_spread_out_ent_mesh, simple_line, sphere_mesh, spread_out_ent_mesh, textured_sphere_mesh, wireframe_sphere_mesh, xyz_mesh};
use crate::game_engine::{CoolGameEngineBase, CoolVoxel, CoolVoxelType, ExtraData};
use crate::driver::{Collider, GameEntityVec, Movement, NewGameEntity, StaticCollider, StaticGameEntity, StaticMeshInfo, StaticMovement};
use crate::game_input_handler::GameInputHandler;
use crate::game_map::{get_f64_pos, get_float_pos, light_spreader::{LightPos, LightSpread}, ChunkDims, GameMap, VoxelLight};
use crate::gui_elements::{list_choice::get_list_choice, number_config::get_number_config};
use hord3::{defaults::{default_frontends::minifb_frontend::MiniFBWindow, default_rendering::vectorinator_binned::{Vectorinator, meshes::{Mesh, MeshID, MeshLODS, MeshLODType}, rendering_spaces::ViewportData, shaders::NoOpShader, textures::{TextureSetID, argb_to_rgb, rgb_to_argb}, triangles::{color_u32_to_u8_simd, simd_rgb_to_argb}}, default_ui::simple_ui::{SimpleUI, UIDimensions, UIElement, UIElementBackground, UIElementContent, UIElementID, UIEvent, UIUnit, UIUserAction, UIVector}}, horde::{frontend::{HordeWindowDimensions, WindowingHandler}, game_engine::{entity::Renderable, multiplayer::{HordeMultiModeChoice, MustSync}, world::{WorldComputeHandler, WorldHandler}}, geometry::{plane::EquationPlane, rotation::{Orientation, Rotation}, vec3d::{Vec3D, Vec3Df}}, rendering::{camera::Camera, framebuffer::HordeColorFormat}, scheduler::{HordeScheduler, HordeTaskQueue, HordeTaskSequence, SequencedTask}, sound::{SoundRequest, WaveIdentification, WavePosition, WaveRequest, WaveSink, Waves}}};
use noise::{NoiseFn, Perlin, Seedable};
use crate::tile_editor::{get_tile_voxels, TileEditorData};
use server_tasks::{ServerTask, ServerTaskTaskHandler};

use crate::{driver::{actions::{Action, ActionKind, ActionSource, ActionTimer, ActionsEvent, ActionsUpdate, StaticGameActions}, director::{llm_director::LLMDirector, Director, DirectorKind, StaticDirector}, planner::StaticPlanner, GameEntityEvent}, game_map::get_voxel_pos, proxima_link::ProximaLink};

pub mod server_tasks;

pub fn server_func() {
    const TICKRATE:usize = 45;
    let mut world = GameMap::new(100, ChunkDims::new(8, 8, 8), get_tile_voxels(), (255,255,255), 1, Road::new(WorldChunkPos::new(0,0,1), Vec3Df::new(1.0, 0.0, 0.0)));
    let mut perlin = Perlin::new().set_seed(13095);
    let mut world_height = 15.0;
    let mut water_level = 10.0;
    let start = Vec3D::new(-15, -15, -2);
    let end = Vec3D::new(15, 15, 5);

    let mut ground_at = vec![0; ((end.x - start.x) * 8 * (end.y - start.y) * 8) as usize];
    let length_f64 = ((end.x - start.x) * 8 ) as f64;
    world.generate_chunks(start, end, &mut |pos| {
        let pos_3D = (get_f64_pos(pos) * 0.01);
        let value_2D = (perlin.get([pos_3D.x, pos_3D.y]) + 1.0) * 0.5;
        let local_world_height = world_height - (((pos.x - start.x) * 8) as f64/length_f64) * world_height;
        let actual_height = local_world_height + world_height * value_2D * 2.0;
        if /*(pos.z as f64) < actual_height ||*/ (pos.z as f64) < water_level {
            let ground_pos = (pos.x - (start.x * 8) + (pos.y - (start.y * 8)) * ((end.y - start.y) * 8)) as usize;
            if (pos.z as f64) < water_level {
                if let Some(val) =  ground_at.get(ground_pos) && *val < pos.z {
                    ground_at[ground_pos] = water_level as i32;
                }
                CoolVoxel {voxel_type:6, orient:0, light:VoxelLight::random_light()}
            }
            else {
                if let Some(val) =  ground_at.get(ground_pos) && *val < pos.z {
                    ground_at[ground_pos] = pos.z;
                }
                CoolVoxel {voxel_type:1 + ((actual_height - water_level)/(6.0*world_height * (1.0/6.0))).clamp(0.0, 5.99) as u16, orient:0, light:VoxelLight::random_light()}
            }
        } else {
            CoolVoxel {voxel_type:0, orient:0, light:VoxelLight::zero_light()}
        }
    }
    );
    let mut world_clone = world.clone();
    if false {
        world_clone.change_mesh_vec(10);
        world_clone.set_min_light_levels((50,50,50));
        for i in 0..1 {
            let (x,y) = (fastrand::i32((start.x * 8)..(end.x * 8)), fastrand::i32((start.y * 8)..(end.y * 8)));
            let light_source = LightPos::new(Vec3D::new(x, y, ground_at[(x - (start.x * 8) + (y - (start.y * 8)) * ((end.y - start.y) * 8)) as usize] + 1), VoxelLight::slightly_less_random_light());
            let total_light_spread = LightSpread::calc_max_spread(&world_clone, light_source).get_all_spread();
            for light_pos in total_light_spread {
                world_clone.get_voxel_at_mut(light_pos.pos()).unwrap().light = light_pos.value().merge_with_other(&world_clone.get_voxel_at(light_pos.pos()).unwrap().light);
            }
            println!("light {i} done !");
        }
    }
    
    let entity_vec = GameEntityVec::new(1000);
    {
        let mut writer = entity_vec.get_write();
        writer.new_sct(StaticGameEntity{planner:StaticPlanner{},director:StaticDirector {kind:DirectorKind::Nothing},actions:StaticGameActions {base_actions:Vec::with_capacity(8)},movement:StaticMovement{}, mesh_info:StaticMeshInfo{mesh_id:MeshID::Named("EntityMesh".to_string()),mesh_data:Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(simple_line(-Vec3D::all_ones()*0.5, Vec3D::all_ones()*0.5, 2, (255,255,255))))]), "EntityMesh".to_string(), 2.0)}, stats:StaticStats{}, collider:StaticCollider{init_aabb:AABB::new(-Vec3D::all_ones()*0.5, Vec3D::all_ones()*0.5)}});

        writer.new_sct(StaticGameEntity{planner:StaticPlanner{},director:StaticDirector {kind:DirectorKind::Nothing},actions:StaticGameActions {base_actions:Vec::with_capacity(8)},movement:StaticMovement{}, mesh_info:StaticMeshInfo{mesh_id:MeshID::Named("GREY_MESH".to_string()),mesh_data:grey_sphere_mesh()}, stats:StaticStats{}, collider:StaticCollider{init_aabb:AABB::new(-Vec3D::all_ones()*0.5, Vec3D::all_ones()*0.5)}});

        for i in 0..1 {
            let pos = Vec3D::new((fastrand::f32() - 0.5) * 2.0 * 50.0, (fastrand::f32() - 0.5) * 2.0 * 50.0, 30.0);
            writer.new_ent(NewGameEntity::new(Movement{against_wall:false, touching_ground:false,pos:pos, speed:Vec3D::zero(), orient:Orientation::zero(), rotat:Rotation::from_orientation(Orientation::zero())}, Stats {static_type_id:1, health:0, damage:0, stamina:0, ground_speed:0.2, jump_height:1.0, personal_vehicle:Some(0)}, Collider{team:0, collider:AABB::new(pos - Vec3D::all_ones() * 0.5, pos + Vec3D::all_ones() * 0.5)}, Director::new_with_random_name(DirectorKind::Nothing), MustSync::Server, None));
            //writer.new_ent(NewGameEntity::new(Movement{against_wall:false, touching_ground:false,pos:pos, speed:Vec3D::zero(), orient:Orientation::zero(), rotat:Rotation::from_orientation(Orientation::zero())}, Stats {static_type_id:1, health:0, damage:0, stamina:0, ground_speed:0.2, jump_height:1.0}, Collider{team:0, collider:AABB::new(pos - Vec3D::all_ones() * 0.5, pos + Vec3D::all_ones() * 0.5)}, Director::new_with_random_name(DirectorKind::LLM(LLMDirector::new_with_goals(test_goals[i].clone())))));
        }
    }

    let (payload_sender, response_receiver) = match ProximaLink::initialize(String::from("HORDE"), String::from("HORDE"), String::from("http://localhost:8085")) {
        Ok((s, r)) => (s, r),
        Err(_) => (mpmc::channel().0, mpmc::channel().1)
    };
    
    let mut entity_vec_2 = VehicleEntityVec::new(1000);
    {
        let mut writer = entity_vec_2.get_write();
        writer.new_sct(get_default_car_type());

        writer.new_ent(NewVehicleEntity::new(VehiclePosition::new().with_pos(Vec3Df::new(0.0, 0.0, 10.5)), VehicleStats {static_id:0, nitro_left:100.0, mass:10.0},  MustSync::Server, None));
    }
    let windowing = WindowingHandler::new::<MiniFBWindow>(HordeWindowDimensions::new(1280, 720), HordeColorFormat::ARGB8888);
    let framebuf = windowing.get_outside_framebuf();
    let mut shader = Arc::new(GameShader::new_default());
    let viewport_data = {
        let framebuf = framebuf.read().unwrap();
        ViewportData {
            near_clipping_plane: 1.0,
            half_image_width: (framebuf.get_dims().get_width()/2) as f32,
            half_image_height: (framebuf.get_dims().get_height()/2) as f32,
            aspect_ratio: (framebuf.get_dims().get_width() as f32)/(framebuf.get_dims().get_height() as f32),
            camera_plane: EquationPlane::new(Vec3D::new(0.0, 0.0, 1.0), -1.0),
            image_height: (framebuf.get_dims().get_height() as f32),
            image_width: (framebuf.get_dims().get_width() as f32),
            poscam: Vec3D::zero(),
            rotat_cam: Rotation::new_from_inverted_orient(Orientation::zero())
        }
    };
    let vectorinator = Vectorinator::new(framebuf.clone(), shader);
    let (waves, waves_handler, stream) = Waves::new(Vec::new(), 10);
    let world_handler = WorldHandler::new(world);
    
    let engine = CoolGameEngineBase::new(
        entity_vec, entity_vec_2, world_handler.clone(), Arc::new(vectorinator.clone()), 
        HordeMultiModeChoice::Server { adress: (Ipv4Addr::new(127, 0, 0, 1), 5678), max_players: 30, tick_tolerance: 3, tickrate: TICKRATE },
        ExtraData {payload_sender, tick: Arc::new(AtomicUsize::new(0)), waves:waves_handler.clone(), current_render_data:Arc::new(RwLock::new((Camera::empty(), viewport_data.clone()))),}
        
    );
    waves_handler.send_gec(engine.clone());

    let handler = ServerTaskTaskHandler::new(engine.clone(), windowing);
    
    let queue = HordeTaskQueue::new(vec![HordeTaskSequence::new(vec![
        SequencedTask::StartTask(ServerTask::ApplyEvents),
        SequencedTask::WaitFor(ServerTask::ApplyEvents),
        SequencedTask::StartTask(ServerTask::Main),
        SequencedTask::WaitFor(ServerTask::Main),
        SequencedTask::StartTask(ServerTask::ApplyEvents),
        SequencedTask::WaitFor(ServerTask::ApplyEvents),
        SequencedTask::StartTask(ServerTask::AfterMain),
        SequencedTask::WaitFor(ServerTask::AfterMain),
        SequencedTask::StartTask(ServerTask::ApplyEvents),
        SequencedTask::WaitFor(ServerTask::ApplyEvents),
        SequencedTask::StartTask(ServerTask::SendMustSync),
        SequencedTask::WaitFor(ServerTask::SendMustSync),
        SequencedTask::StartTask(ServerTask::MultiFirstPart),
        SequencedTask::WaitFor(ServerTask::MultiFirstPart),
        SequencedTask::StartTask(ServerTask::MultiSecondPart),
        SequencedTask::WaitFor(ServerTask::MultiSecondPart),
        SequencedTask::StartTask(ServerTask::MultiThirdPart),
        SequencedTask::WaitFor(ServerTask::MultiThirdPart),
        ]
    )], Vec::new());
    println!("Hello, world!");
    let mut scheduler = HordeScheduler::new(queue.clone(), handler, 16);
    
    for i in 0..75000 {
        println!("----------------- NEW TICK {i} ----------------");

        let mut start = Instant::now();
        let new_camera = {
            if i < 100 {
                let mut world_write = engine.world.world.write().unwrap();
                let dims = world_write.get_chunk_dims_vector_f();
                world_write.generator.step_forwards(3.0, &dims);
                let chunks = world_write.generator.get_chunks_to_generate(3.0, &world_write);
                let data = world_write.generate_chunks_with_generator_and_get_them(chunks);
                for (c_pos, chunk) in data {
                    engine.world.tunnels_out.send_event(GameMapEvent::NewChunk(c_pos, chunk));
                }
            }
            engine.extra_data.tick.fetch_add(1, Ordering::Relaxed);
            0
        };
        {
            let first_ent = engine.entity_1.get_read();
            let second_ent = engine.vehicles.get_read();
            let world = WorldComputeHandler::from_world_handler(&engine.world);
            loop {
                match response_receiver.try_recv() {
                    Ok(response) => response.apply(&first_ent, &second_ent, &world),
                    Err(_) => break
                }
            }
        }
        
        scheduler.initialise(queue.clone());
        scheduler.tick();
        let frametime = Instant::now().checked_duration_since(start).unwrap().as_secs_f64();
        let mut fps = 1.0/frametime;
        println!("FPS : {}", fps);
        if fps > TICKRATE as f64 {
            thread::sleep(Duration::from_secs_f64(1.0/(TICKRATE as f64) - frametime));
        }
    }
    scheduler.end_threads();
}