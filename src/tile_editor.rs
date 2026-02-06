use std::{collections::{HashMap, HashSet, VecDeque}, f32::consts::{PI, SQRT_2}, path::{Path, PathBuf}, sync::Arc};

use cosmic_text::{Color, Metrics};
use hord3::{defaults::{default_rendering::vectorinator_binned::{meshes::{Mesh, MeshID, MeshInstance, MeshLODS, MeshLODType}, shaders::NoOpShader, textures::Textures, Vectorinator}, default_ui::simple_ui::{SimpleUI, SimpleUISave, TextCentering, UIElement, UIElementBackground, UIElementContent, UIElementID}}, horde::{frontend::{interact::Button, MouseState}, game_engine::{multiplayer::Identify, world::WorldTunnelsOut}, geometry::{rotation::Orientation, vec3d::{Vec3D, Vec3Df}}, rendering::camera::Camera}};

use crate::{client::client_tasks::GameUserEvent, cutscene::game_shader::GameShader, game_3d_models::{lit_selection_cube, selection_cube}, game_engine::{CoolGameEngineTID, CoolVoxel, CoolVoxelType}, game_input_handler::GameInputHandler, game_map::{GameMap, GameMapEvent, Voxel, VoxelLight, WorldChunkPos, WorldVoxelPos, get_chunk_pos_i, get_float_pos, get_voxel_pos, light_spreader::{LightPos, LightSpread}, raycaster::Ray, road::Road}, gui_elements::{editor_gui_elements::{light_spreader_elts, voxel_type_choice}, list_choice}, vehicle::locomotion::SurfaceType};


pub const CHUNK_SIZE:usize = 8;
pub const CHUNK_SIZE_F:f32 = CHUNK_SIZE as f32;

pub fn get_tile_voxels() -> Vec<CoolVoxelType> {
    vec![
        CoolVoxelType::new(0b00111111, 0, VoxelLight::new(247, 255, 255, 255), None, "Air".to_string(), Some(PathBuf::from("textures/arbre.png")), None, None),
        CoolVoxelType::new(0, 1, VoxelLight::zero_light(), None, "Sand".to_string(), Some(PathBuf::from("textures/sable.png")), None, Some(SurfaceType::Ground)),
        CoolVoxelType::new(0, 2, VoxelLight::zero_light(), None, "Flowers".to_string(), Some(PathBuf::from("textures/terre_herbe.png")), None, Some(SurfaceType::Ground)),
        CoolVoxelType::new(0, 3, VoxelLight::zero_light(), None, "Grassy Ground".to_string(), Some(PathBuf::from("textures/terre_cail.png")), None, Some(SurfaceType::Ground)),
        CoolVoxelType::new(0, 4, VoxelLight::zero_light(), None, "Ground".to_string(), Some(PathBuf::from("textures/terre.png")), None, Some(SurfaceType::Ground)),
        CoolVoxelType::new(0, 5, VoxelLight::zero_light(), None, "Rock".to_string(), Some(PathBuf::from("textures/roche.png")), None, Some(SurfaceType::Ground)),
        CoolVoxelType::new(0, 0, VoxelLight::zero_light(), None, "Snow".to_string(), Some(PathBuf::from("textures/neige.png")), None, Some(SurfaceType::Ground)),
        CoolVoxelType::new(0, 6, VoxelLight::zero_light(), None, "Water".to_string(), Some(PathBuf::from("textures/eau.png")), None, Some(SurfaceType::Water)),
        CoolVoxelType::new(0, 7, VoxelLight::zero_light(), None, "Deep Water".to_string(), Some(PathBuf::from("textures/eau_prof.png")), None, Some(SurfaceType::Water)),
        CoolVoxelType::new(0, 8, VoxelLight::zero_light(), None, "Metal".to_string(), Some(PathBuf::from("textures/metal_0.png")), None, Some(SurfaceType::Ground)),
        CoolVoxelType::new(0, 3, VoxelLight::zero_light(), None, "Text Test".to_string(), None, None, Some(SurfaceType::Ground)),
    ]
}

pub fn get_selector_cube_mesh() -> Mesh {
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(selection_cube(Vec3D::new(-0.5, -0.5, -0.5), Vec3D::new(0.5, 0.5, 0.5), 2)))]), "Selection_Cube".to_string(), 2.0)
}
pub fn get_chunk_cube_mesh() -> Mesh {
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(selection_cube(Vec3D::new(0.0, 0.0, 0.0), Vec3D::new(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F), 2)))]), "Chunk_Cube".to_string(), CHUNK_SIZE_F * SQRT_2)
}

pub fn get_selected_chunk_cube_mesh() -> Mesh {
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(selection_cube(Vec3D::new(0.0, 0.0, 0.0), Vec3D::new(CHUNK_SIZE_F, CHUNK_SIZE_F, CHUNK_SIZE_F), 1)))]), "Selected_Chunk_Cube".to_string(), CHUNK_SIZE_F * SQRT_2)
}

pub fn get_zone_selection_cube(start:Vec3Df, end:Vec3Df) -> Mesh {
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(selection_cube(Vec3D::zero(), end - start, 2)))]), "Zone_Selection_Cube".to_string(), start.dist(&end) * SQRT_2)
}

pub fn get_lit_selection_cube(light:(u8,u8,u8)) -> Mesh {
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(lit_selection_cube(Vec3D::new(-0.5, -0.5, -0.5), Vec3D::new(0.5, 0.5, 0.5), 2, light)))]), "Light_Selection_cube".to_string(),  2.0)
}

#[derive(Clone)]
pub enum TileEditingTool {
    PlaceAndDestroy { chosen:usize, empty_voxel:usize },
    ZonedPlaceAndDestroy {chosen:usize, empty_voxel:usize, start:WorldVoxelPos, end:WorldVoxelPos, action:Option<ZoneAction>},
    MakeLight {strength:u8, color:(u8,u8,u8)},
    ChooseTileChunks,
}

#[derive(Clone, Debug, Copy)]
pub enum ZoneAction {
    Place,
    Destroy
}

impl TileEditingTool {
    pub fn ui_elems(&self) -> Vec<Vec<UIElement<GameUserEvent>>> {
        match self {
            TileEditingTool::PlaceAndDestroy { chosen, empty_voxel } => {
                voxel_type_choice(get_tile_voxels(), "PlaceAndDestroy".to_string())
            },
            TileEditingTool::ChooseTileChunks => vec![],
            TileEditingTool::ZonedPlaceAndDestroy { chosen, empty_voxel, start, end, action } => voxel_type_choice(get_tile_voxels(), "ZonedPlaceAndDestroy".to_string()),
            TileEditingTool::MakeLight { strength, color } => light_spreader_elts()
        }
    }
    pub fn update_ui(&mut self, ui:&mut SimpleUI<GameUserEvent>) {
        
    }
    pub fn handle_mouse_state(&mut self, editor_data:&mut TileEditorData, chunks:&mut GameMap<CoolVoxel, Road>, tunnels:WorldTunnelsOut<GameMap<CoolVoxel, Road>, CoolGameEngineTID>) -> Self {
        editor_data.mouse_state.update_local();
        let ray = Ray::new(editor_data.cam.pos, Orientation::new(editor_data.cam.orient.yaw - PI/2.0, editor_data.cam.orient.roll - PI/8.0, 0.0).into_vec(), Some(100.0));
        match self {
            TileEditingTool::PlaceAndDestroy {chosen, empty_voxel } => {
                if editor_data.mouse_state.get_deltas_and_scroll().left >= 2 { // Destroy
                    let end = ray.get_end(&chunks);
                    //dbg!(end.final_length);
                    let voxel_types = chunks.get_voxel_types().clone();
                    let mut modified_at = None;
                    match chunks.get_voxel_at(get_voxel_pos(end.end)) {
                        Some(voxel) => {
                            //println!("FOUND A VOXEL");
                            //dbg!(voxel.voxel_id());
                            if voxel.voxel_id() != *empty_voxel {
                                editor_data.action_queue.push_back(EditorAction::ModifyVoxel { position: get_voxel_pos(end.end), previous_state:voxel.clone() });
                                modified_at = Some(get_voxel_pos(end.end));
                                // chunks.modified_this_pos_signal_remesh(get_voxel_pos(end.end));

                                let mut new_voxel = voxel.clone();
                                new_voxel.voxel_type = *empty_voxel as u16;
                                tunnels.send_event(GameMapEvent::UpdateVoxelAt(get_voxel_pos(end.end), new_voxel));
                                //println!("TEST");
                            }
                        },
                        None => ()
                    }
                    match modified_at {
                        Some(pos) => {
                            chunks.modified_this_pos_signal_remesh(pos);
                        },
                        None => ()
                    }
                    self.clone()
                }
                else if editor_data.mouse_state.get_deltas_and_scroll().right >= 2 { // Place
                    let end = ray.get_first_back_different(&chunks, None);
                    //dbg!(end.final_length);
                    let voxel_types = chunks.get_voxel_types().clone();
                    let mut modified_at = None;
                    match chunks.get_voxel_at(get_voxel_pos(end.end)) {
                        Some(voxel) => {
                            //println!("FOUND A VOXEL");
                            //dbg!(voxel.voxel_id());
                            if voxel.voxel_id() != *chosen {
                                editor_data.action_queue.push_back(EditorAction::ModifyVoxel { position: get_voxel_pos(end.end), previous_state:voxel.clone() });
                                modified_at = Some(get_voxel_pos(end.end));
                                // chunks.modified_this_pos_signal_remesh(get_voxel_pos(end.end));
                                let mut new_voxel = voxel.clone();
                                new_voxel.voxel_type = *chosen as u16;
                                tunnels.send_event(GameMapEvent::UpdateVoxelAt(get_voxel_pos(end.end), new_voxel));
                            }
                            //println!("TEST");
                        },
                        None => ()
                    }
                    match modified_at {
                        Some(pos) => {
                            chunks.modified_this_pos_signal_remesh(pos);
                        },
                        None => ()
                    }
                    self.clone()
                }
                else {
                    self.clone()
                }
            },
            TileEditingTool::ChooseTileChunks => {
                if editor_data.mouse_state.get_deltas_and_scroll().left >= 2 { // Add to chunks
                    if editor_data.tile_chunks.insert(chunks.get_chunk_pos(editor_data.cam.pos)) {
                        editor_data.action_queue.push_back(EditorAction::ChooseTiles { position: chunks.get_chunk_pos(editor_data.cam.pos), added:true });
                    }
                    self.clone()
                }
                else if editor_data.mouse_state.get_deltas_and_scroll().right >= 2 { // Remove from chunks
                    if editor_data.tile_chunks.remove(&chunks.get_chunk_pos(editor_data.cam.pos)) {
                        editor_data.action_queue.push_back(EditorAction::ChooseTiles { position: chunks.get_chunk_pos(editor_data.cam.pos), added:false });
                    }
                    self.clone()
                }
                else {
                    self.clone()
                }
            },
            TileEditingTool::ZonedPlaceAndDestroy { chosen, empty_voxel, start, end, action } => {
                let ray_end = ray.get_end(&chunks);
                
                if editor_data.mouse_state.get_current_state().left == 0 && editor_data.mouse_state.get_current_state().right == 0 && editor_data.mouse_state.get_deltas_and_scroll().left == 0 && editor_data.mouse_state.get_deltas_and_scroll().right == 0 {
                    *start = get_voxel_pos(ray_end.end);
                    *end = get_voxel_pos(ray_end.end);
                    self.clone()
                }
                else if editor_data.mouse_state.get_deltas_and_scroll().left >= 2 && action.is_none() { // Destroy
                    let end = ray.get_end(&chunks);
                    *start = get_voxel_pos(end.end);
                    *action = Some(ZoneAction::Destroy);
                    self.clone()
                }
                else if editor_data.mouse_state.get_deltas_and_scroll().right >= 2 && action.is_none() { // Place
                    let end = ray.get_end(&chunks);
                    *start = get_voxel_pos(end.end);
                    *action = Some(ZoneAction::Place);
                    self.clone()
                }
                else {
                    *end = get_voxel_pos(ray_end.end);

                    dbg!(action.clone(), editor_data.mouse_state.get_deltas_and_scroll());
                    if (editor_data.mouse_state.get_deltas_and_scroll().left <= -2 || editor_data.mouse_state.get_deltas_and_scroll().right <= -2) && action.is_some() {
                        let mut changes = Vec::with_capacity(64);
                        match action {
                            Some(act) => {
                                for x in start.x.min(end.x)..=start.x.max(end.x) {
                                    for y in start.y.min(end.y)..=start.y.max(end.y) {
                                        for z in start.z.min(end.z)..=start.z.max(end.z) {
                                            let mut modified = false;
                                            match chunks.get_voxel_at_mut(WorldVoxelPos::new(x, y, z)) {
                                                Some(vox) => {
                                                    match act.clone() {
                                                        ZoneAction::Place => {
                                                            if vox.voxel_id() != *chosen {
                                                                modified = true;
                                                                changes.push((WorldVoxelPos::new(x, y, z), vox.clone()));
                                                            }
                                                            vox.voxel_type = *chosen as u16;
                                                        },
                                                        ZoneAction::Destroy => {
                                                            if vox.voxel_id() != *empty_voxel {
                                                                modified = true;
                                                                changes.push((WorldVoxelPos::new(x, y, z), vox.clone()));
                                                            }
                                                            vox.voxel_type = *empty_voxel as u16;
                                                        },
                                                    }
                                                },
                                                None => ()
                                            }
                                            if modified {
                                                chunks.modified_this_pos_signal_remesh(Vec3D::new(x, y, z));
                                            }
                                            
                                        }
                                    }
                                }
                                *action = None;
                                if changes.len() > 0 {
                                    editor_data.action_queue.push_back(EditorAction::ModifyVoxels { positions_previous: changes });
                                }
                                self.clone()
                            },  
                            None =>{ *action = None; self.clone()}
                        }
                    }
                    else {
                        self.clone()
                    }
                }

            },
            TileEditingTool::MakeLight { strength, color } => {
                *strength = *editor_data.ui_variables.get("Light Strength").unwrap_or(&255) as u8;
                color.0 = *editor_data.ui_variables.get("Light Red Color").unwrap_or(&255) as u8;
                color.1 = *editor_data.ui_variables.get("Light Green Color").unwrap_or(&255) as u8;
                color.2 = *editor_data.ui_variables.get("Light Blue Color").unwrap_or(&255) as u8;
                
                if editor_data.mouse_state.get_deltas_and_scroll().left >= 2 { // Destroy
                    
                    self.clone()
                }
                else if editor_data.mouse_state.get_deltas_and_scroll().right >= 2 { // Place
                    let end = ray.get_first_back_different(&chunks, None);
                    //dbg!(end.final_length);
                    let voxel_types = chunks.get_voxel_types().clone();
                    let mut modified_at = None;
                    match chunks.get_voxel_at_mut(get_voxel_pos(end.end)) {
                        Some(voxel) => {
                            //println!("FOUND A VOXEL");
                            //dbg!(voxel.voxel_id());
                            modified_at = Some(get_voxel_pos(end.end));
                            //println!("TEST");
                        },
                        None => ()
                    }
                    match modified_at {
                        Some(pos) => {
                            let mut changes = Vec::with_capacity(64);
                            let mut light = LightSpread::calc_max_spread(chunks, LightPos::new(pos, VoxelLight::new(*strength, color.0, color.1, color.2)));
                            let max_spread = light.get_all_spread();
                            for light_pos in max_spread {
                                changes.push((light_pos.pos(), chunks.get_voxel_at_mut(light_pos.pos()).unwrap().clone()));
                                chunks.get_voxel_at_mut(light_pos.pos()).unwrap().light = light_pos.value().merge_with_other(&chunks.get_voxel_at(light_pos.pos()).unwrap().light);
                                chunks.modified_this_pos_signal_remesh(light_pos.pos());
                            }
                            editor_data.action_queue.push_back(EditorAction::ModifyVoxels { positions_previous: changes });
                            
                        },
                        None => ()
                    }
                    self.clone()
                }
                else {
                    self.clone()
                }
            }
        }
    }
    pub fn add_viewmodel(&self, vectorinator:&Vectorinator<GameShader>, chunks:&GameMap<CoolVoxel, Road>, editor_data:&TileEditorData) {

        let mut write = vectorinator.get_write();
        let ray = Ray::new(editor_data.cam.pos, Orientation::new(editor_data.cam.orient.yaw - PI/2.0, editor_data.cam.orient.roll - PI/8.0, 0.0).into_vec(), Some(100.0));
        let ray_end = ray.get_end(chunks);


        write.meshes.change_visibility_of_all_instances_of_vec(8, false);
        write.meshes.change_visibility_of_all_instances_of_vec(7, false);
        write.meshes.change_visibility_of_all_instances_of_vec(6, false);
        write.meshes.change_visibility_of_all_instances_of_vec(5, false);
        match self {
            TileEditingTool::PlaceAndDestroy {chosen, empty_voxel } => {
                write.meshes.set_or_add_mesh(&MeshID::Named("Selection_Cube".to_string()), get_selector_cube_mesh());
                write.meshes.set_or_add_instance(
                    MeshInstance::new(
                        get_float_pos(get_voxel_pos(ray_end.end)),
                        Orientation::zero(),
                        MeshID::Named("Selection_Cube".to_string()),
                        true,
                        false,
                        false
                    ),
                    5,
                    0
                );
            },
            TileEditingTool::ChooseTileChunks => {
                write.meshes.set_or_add_mesh(&MeshID::Named("Selected_Chunk_Cube".to_string()), get_selected_chunk_cube_mesh());
                write.meshes.set_or_add_mesh(&MeshID::Named("Chunk_Cube".to_string()), get_chunk_cube_mesh());
                if !editor_data.tile_chunks.contains(&chunks.get_chunk_pos(editor_data.cam.pos)) {
                    write.meshes.set_or_add_instance(
                        MeshInstance::new(
                            get_float_pos(chunks.get_chunk_pos(editor_data.cam.pos) * chunks.get_chunk_dims_vector().x),
                            Orientation::zero(),
                            MeshID::Named("Selected_Chunk_Cube".to_string()),
                            true,
                            false,
                            false
                        ),
                        6,
                        0
                    );
                }
                for (i, chunk) in editor_data.tile_chunks.iter().enumerate() {
                    write.meshes.set_or_add_instance(
                        MeshInstance::new(
                            get_float_pos(chunk * chunks.get_chunk_dims_vector().x),
                            Orientation::zero(),
                            MeshID::Named("Chunk_Cube".to_string()),
                            true,
                            false,
                            false
                        ),
                        6,
                        i + 1
                    );
                }
            },
            TileEditingTool::ZonedPlaceAndDestroy { chosen, empty_voxel, start, end, action } => {
                if *start != *end {
                    write.meshes.set_or_add_mesh(&MeshID::Named("Zone_Selection_Cube".to_string()), get_zone_selection_cube(get_float_pos(*start), get_float_pos(*end)));
                    write.meshes.set_or_add_instance(
                        MeshInstance::new(
                            get_float_pos(*start),
                            Orientation::zero(),
                            MeshID::Named("Zone_Selection_Cube".to_string()),
                            true,
                            false,
                            false
                        ),
                        7,
                        0
                    );
                }
                else {
                    write.meshes.set_or_add_mesh(&MeshID::Named("Selection_Cube".to_string()), get_selector_cube_mesh());
                    write.meshes.set_or_add_instance(
                        MeshInstance::new(
                            get_float_pos(get_voxel_pos(ray_end.end)),
                            Orientation::zero(),
                            MeshID::Named("Selection_Cube".to_string()),
                            true,
                            false,
                            false
                        ),
                        5,
                        0
                    );
                }
                
            },
            TileEditingTool::MakeLight { strength, color } => {
                let real_light = (((color.0 as f32)/255.0 * *strength as f32) as u8, ((color.1 as f32)/255.0 * *strength as f32) as u8, ((color.2 as f32)/255.0 * *strength as f32) as u8);
                write.meshes.set_or_add_mesh(&MeshID::Named("Light_Selection_cube".to_string()), get_lit_selection_cube(real_light));
                write.meshes.set_or_add_instance(
                    MeshInstance::new(
                        get_float_pos(get_voxel_pos(ray_end.end)),
                        Orientation::zero(),
                        MeshID::Named("Light_Selection_cube".to_string()),
                        true,
                        false,
                        false
                    ),
                    8,
                    0
                );
            }
        }
    }
    fn change_chosen_unwrap(&mut self, new_chosen:usize, ui:&mut SimpleUI<GameUserEvent>) {
        match self {
            TileEditingTool::PlaceAndDestroy { chosen, empty_voxel } => {
                *chosen = new_chosen;
                let type_used = get_tile_voxels()[new_chosen].clone();
                ui.change_content_of(UIElementID::Name(
                    "PlaceAndDestroy Image Title".to_string()),
                    0,
                    UIElementContent::Text { text: type_used.name.clone(), font: "rien".to_string(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255), centering:TextCentering::Neither }
                );
                ui.change_content_background_of(UIElementID::Name(
                    "PlaceAndDestroy Image Show".to_string()),
                    0,
                    UIElementBackground::Image(type_used.name.clone())
                );
            },
            TileEditingTool::ZonedPlaceAndDestroy { chosen, empty_voxel, start, end, action } => {
                *chosen = new_chosen;
                let type_used = get_tile_voxels()[new_chosen].clone();
                ui.change_content_of(UIElementID::Name(
                    "ZonedPlaceAndDestroy Image Title".to_string()),
                    0,
                    UIElementContent::Text { text: type_used.name.clone(), font: "rien".to_string(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255), centering:TextCentering::Neither }
                );
                ui.change_content_background_of(UIElementID::Name(
                    "ZonedPlaceAndDestroy Image Show".to_string()),
                    0,
                    UIElementBackground::Image(type_used.name.clone())
                );
            },
            _ => ()
        }
    }
}

pub struct TileEditorData {
    ui_variables:HashMap<String, i32>,
    ui_list_choices:HashMap<String, HashMap<String,bool>>,
    pub cam:Camera,
    mouse_state:MouseState,
    tile_chunks:HashSet<WorldChunkPos>,
    tools:HashMap<String, TileEditingTool>,
    chosen_tool:String,
    ui:SimpleUI<GameUserEvent>,
    action_queue:VecDeque<EditorAction>
}

pub enum EditorAction {
    ModifyVoxel {position:WorldVoxelPos, previous_state:CoolVoxel},
    ChooseTiles {position:WorldChunkPos, added:bool},
    ModifyVoxels {positions_previous: Vec<(WorldChunkPos, CoolVoxel)>}
}

impl EditorAction {
    pub fn reverse_action(self, editor_data:&mut TileEditorData, chunks:&mut GameMap<CoolVoxel, Road>, tunnels:WorldTunnelsOut<GameMap<CoolVoxel, Road>, CoolGameEngineTID>) {
        match self {
            EditorAction::ModifyVoxel { position, previous_state } => {
                tunnels.send_event(GameMapEvent::UpdateVoxelAt(position, previous_state));
            },
            EditorAction::ChooseTiles { position, added } => {
                if added {
                    editor_data.tile_chunks.remove(&position);
                }
                else {
                    editor_data.tile_chunks.insert(position);
                }
            },
            EditorAction::ModifyVoxels { positions_previous } => {
                for (position, previous_state) in positions_previous {
                    tunnels.send_event(GameMapEvent::UpdateVoxelAt(position, previous_state));
                }
            }
        }
    }
}

impl TileEditorData {
    
    pub fn new(ui:SimpleUI<GameUserEvent>, cam:Camera, mouse_state:MouseState) -> Self {
        Self {
            ui_variables: HashMap::with_capacity(128),
            ui_list_choices: HashMap::with_capacity(8),
            cam,
            mouse_state,
            tile_chunks:HashSet::with_capacity(128),
            tools:HashMap::from([("LightSpreader".to_string(), TileEditingTool::MakeLight { strength: 255, color: (255,255,255) }),("TerrainModifier".to_string(), TileEditingTool::PlaceAndDestroy { chosen: 0, empty_voxel: 0 }), ("TileChooser".to_string(), TileEditingTool::ChooseTileChunks), ("TerrainZoneModifier".to_string(), TileEditingTool::ZonedPlaceAndDestroy { chosen: 0, empty_voxel: 0, start: Vec3D::zero(), end: Vec3D::zero(), action:None })]),
            chosen_tool: "TerrainModifier".to_string(),
            ui,
            action_queue:VecDeque::with_capacity(128)
        }
    }
    pub fn do_mouse_handling(&mut self, chunks: &mut GameMap<CoolVoxel, Road>, tunnels:WorldTunnelsOut<GameMap<CoolVoxel, Road>, CoolGameEngineTID>) {
        let new_tool = self.tools.get(&self.chosen_tool.clone()).unwrap().clone().handle_mouse_state(self, chunks, tunnels);
        *self.tools.get_mut(&self.chosen_tool.clone()).unwrap() = new_tool;
    }
    pub fn do_rendering(&mut self, vectorinator:&Vectorinator<GameShader>, chunks: &GameMap<CoolVoxel, Road>) {
        let tool = self.tools.get(&self.chosen_tool.clone()).unwrap().ui_elems();
        self.ui.change_visibility_of_widgets(tool, true);
        self.tools.get(&self.chosen_tool.clone()).unwrap().add_viewmodel(vectorinator, chunks, self);
    }
    pub fn handle_keyboard(&mut self, game_input:&GameInputHandler, chunks:&mut GameMap<CoolVoxel, Road>, tunnels:WorldTunnelsOut<GameMap<CoolVoxel, Road>, CoolGameEngineTID>) {
        if game_input.get_current_keyboard().contains(&Button::Ctrl) && game_input.is_newly_pressed(&Button::Z) && self.action_queue.len() > 0 {
            let latest_action = self.action_queue.pop_back().unwrap();
            latest_action.reverse_action(self, chunks, tunnels);
        } 
    }
    pub fn handle_user_event(&mut self, evt:GameUserEvent) {
        match evt {
            GameUserEvent::ChoseThatValue(name, value) => {
                if name.trim() == "Tools" {
                    let currently_chosen_tool = self.tools.get(&self.chosen_tool.clone()).unwrap().ui_elems();
                    self.ui.change_visibility_of_widgets(currently_chosen_tool, false);
                    self.chosen_tool = value.clone();

                }
                else if name.trim() == "PlaceAndDestroy" || name.trim() == "ZonedPlaceAndDestroy" {
                    let voxel_type = get_tile_voxels().into_iter().enumerate().find(|(i, voxel_t)| { voxel_t.name == value.trim()}).unwrap().0;
                    self.tools.get_mut(&self.chosen_tool.clone()).unwrap().change_chosen_unwrap(voxel_type, &mut self.ui);
                }
                match self.ui_list_choices.get_mut(&name) {
                    Some(choice_map) => {choice_map.insert(value, true);},
                    None => {self.ui_list_choices.insert(name.clone(), HashMap::from([(value, true)]));},
                }
            },
            GameUserEvent::DecreasedThatValue(val) => {
                match self.ui_variables.get_mut(&val) {
                    Some(value) => *value = (*value - 1).max(0),
                    None =>  {self.ui_variables.insert(val.clone(), 254);}
                }
                self.ui.change_content_of(UIElementID::Name(format!("{} Counter Show", val.clone())), 0, UIElementContent::Text { text: format!("{}", self.ui_variables.get(&val).unwrap()), centering:TextCentering::Neither, font: "rien".to_string(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255) });
            }
            GameUserEvent::IncreasedThatValue(val) => {
                match self.ui_variables.get_mut(&val) {
                    Some(value) => *value = (*value + 1).min(255),
                    None =>  {self.ui_variables.insert(val.clone(), 255);}
                }
                self.ui.change_content_of(UIElementID::Name(format!("{} Counter Show", val.clone())), 0, UIElementContent::Text { text: format!("{}", self.ui_variables.get(&val).unwrap()), centering:TextCentering::Neither, font: "rien".to_string(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255) });
            },
            _ => ()
        }
    }

    pub fn initial_ui_work(&mut self, textures:&Textures) {
        for voxel_type in get_tile_voxels() {
            match voxel_type.texture_path {
                Some(path) => self.ui.add_image(PathBuf::from(path), Some(voxel_type.name)),
                None => self.ui.add_image_from_id(textures, voxel_type.texture),
            }
        }
        for tool in self.tools.values() {
            self.ui.add_multiple_widgets(tool.ui_elems());
            self.ui.change_visibility_of_widgets(tool.ui_elems(), false);
        }
    }
}