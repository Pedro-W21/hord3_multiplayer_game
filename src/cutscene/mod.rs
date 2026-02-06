use std::sync::atomic::Ordering;

use camera_movement::CameraSequence;
use cutscene_gui::UISequence;
use cutscene_shader::{ShaderChange, ShaderSequence};
use entity_movement::EntitySequence;
use game_shader::{GameShader, GameShaderFrameData};
use hord3::{defaults::{default_rendering::vectorinator_binned::{shaders::ShaderData, Vectorinator}, default_ui::simple_ui::SimpleUI}, horde::{game_engine::{engine, multiplayer::Identify}, geometry::vec3d::Vec3Df, rendering::camera::Camera}};

use crate::{game_engine::{CoolGameEngineBase, CoolGameEngineTID}, client::client_tasks::GameUserEvent};

pub mod camera_movement;
pub mod written_texture;
pub mod game_shader;
pub mod write_in_the_air;
pub mod demo_cutscene;
pub mod entity_movement;
pub mod cutscene_gui;
pub mod cutscene_shader;
pub mod reverse_camera_coords;

pub mod real_demo_cutscene;



pub fn execute_camera_sequence(sequence:&mut CameraSequence,vectorinator:&Vectorinator<GameShader>) -> bool { // Is the cutscene continuing ?
    let mut writer = vectorinator.get_write();
    match sequence.advance_sequence() {
        Some(cam) => {*writer.camera = cam;true},// cam,
        None => false
    }
}

pub fn execute_shader_sequence(change:&mut ShaderSequence, vectorinator:&Vectorinator<GameShader>) -> bool {
    match change.advance_sequence(vectorinator.shader_data.get_raw_frame_data()) {
        Some(cam) => {
            vectorinator.shader_data.activated.store(cam.activated, Ordering::Relaxed);
            *vectorinator.shader_data.fog_color.write().unwrap() = cam.fog_color;
            *vectorinator.shader_data.fog_distance.write().unwrap() = cam.fog_distance;
            *vectorinator.shader_data.sun_dir.write().unwrap() = cam.sun_dir;
            true
        },// cam,
        None => false
    }
}

pub fn execute_entity_sequence(sequences:&mut Vec<EntitySequence<CoolGameEngineTID>>, engine:&CoolGameEngineBase) -> bool {
    let mut finished = false;
    for sequence in sequences {
        finished = match sequence.advance_sequence() {
            Some(entity_desc) => {
                match sequence.entity_id.clone() {
                    CoolGameEngineTID::entity_1(id) => {
                        let mut writer = engine.entity_1.get_write();
                        writer.movement[id].orient = entity_desc.orient;
                        writer.movement[id].pos = entity_desc.pos;
                    },
                    CoolGameEngineTID::vehicles(id) => {
                        let mut writer = engine.vehicles.get_write();
                        writer.position[id].orientation = entity_desc.orient;
                        writer.position[id].pos = entity_desc.pos;
                    },
                    CoolGameEngineTID::world => {}
                }
                true
            },
            None => false
        } || finished
    }
    finished
    
}

pub struct ShaderCutscene {
    sequences:Vec<ShaderSequence>,
    current_sequence:usize,
    latest_shaders:GameShaderFrameData
}

impl ShaderCutscene {
    pub fn new(sequences:Vec<ShaderSequence>) -> Self {
        Self { sequences, current_sequence:0, latest_shaders:GameShaderFrameData { activated: false, inv_fog_distance: 1.0, fog_distance: 1.0, fog_color: 0, fog_color_f: (0.0,0.0,0.0), sun_dir: Vec3Df::all_ones().normalise(), sun_dir_norm:1.0, do_normals:false } }
    }
    pub fn execute_shader_cutscene(&mut self, vectorinator:&Vectorinator<GameShader>) -> bool {
        if self.current_sequence < self.sequences.len() {
    
            if !execute_shader_sequence(&mut self.sequences[self.current_sequence], vectorinator) {
                self.current_sequence += 1
            }
            true
        }
        else {
            false
        }
    }
}

pub struct CameraCutscene {
    sequences:Vec<CameraSequence>,
    current_sequence:usize,
    latest_camera:Camera
}

impl CameraCutscene {
    pub fn new(sequences:Vec<CameraSequence>) -> Self {
        Self { sequences, current_sequence:0, latest_camera:Camera::empty() }
    }
    pub fn execute_camera_cutscene(&mut self, vectorinator:&Vectorinator<GameShader>) -> bool {
        if self.current_sequence < self.sequences.len() {
    
            if !execute_camera_sequence(&mut self.sequences[self.current_sequence], vectorinator) {
                self.current_sequence += 1
            }
            true
        }
        else {
            false
        }
    }
}

pub struct EntityCutscene {
    sequences:Vec<Vec<EntitySequence<CoolGameEngineTID>>>,
    current_sequence:usize
}

impl EntityCutscene {
    pub fn new(sequences:Vec<Vec<EntitySequence<CoolGameEngineTID>>>) -> Self {
        Self { sequences, current_sequence:0 }
    }
    pub fn execute_entity_cutscene(&mut self, engine:&CoolGameEngineBase) -> bool {
        if self.current_sequence < self.sequences.len() {
            if !execute_entity_sequence(&mut self.sequences[self.current_sequence], engine) {
                self.current_sequence += 1
            }
            true
        }
        else {
            false
        }
    }
}

pub struct FullCutscene {
    gui:GUICutscene,
    entity:EntityCutscene,
    camera:CameraCutscene,
    shader:ShaderCutscene
}

impl FullCutscene {
    pub fn new(gui:GUICutscene, entity:EntityCutscene, camera:CameraCutscene, shader:ShaderCutscene) -> Self {
        Self { gui, entity, camera, shader }
    }
    pub fn advance_everything(&mut self, vectorinator:&Vectorinator<GameShader>, engine:&CoolGameEngineBase, ui:&mut SimpleUI<GameUserEvent>) -> bool {
        let gui = self.gui.execute_gui_cutscenes(ui);
        let cam = self.camera.execute_camera_cutscene(vectorinator);
        let entity = self.entity.execute_entity_cutscene(engine);
        let shader = self.shader.execute_shader_cutscene(vectorinator);
        gui || cam || entity || shader
    }
    pub fn get_latest_camera(&self) -> Camera {
        self.camera.latest_camera.clone()
    }
}

pub struct GUICutscene {
    sequences:Vec<UISequence>,
    current_sequence:usize
}

impl GUICutscene {
    pub fn new(sequences:Vec<UISequence>) -> Self {
        Self { sequences, current_sequence:0 }
    }
    pub fn execute_gui_cutscenes(&mut self, ui:&mut SimpleUI<GameUserEvent>) -> bool {
        if self.current_sequence < self.sequences.len() {
    
            if !execute_gui_sequence(&mut self.sequences[self.current_sequence], ui) {
                self.current_sequence += 1
            }
            true
        }
        else {
            false
        }
    }
}

pub fn execute_gui_sequence(sequence:&mut UISequence, ui:&mut SimpleUI<GameUserEvent>) -> bool {
    let element_id = sequence.get_element().get_name_as_id();
    if !ui.does_element_exist(element_id.clone()) {
        ui.add_many_connected_elements((sequence.get_elems_func)());
    }
    match sequence.advance_sequence() {
        Some(ui_elem_info) => {
            ui.change_position_of(element_id.clone(), ui_elem_info.pos);
            match ui_elem_info.dim {
                Some(dim) => {dbg!(dim.clone());ui.change_dimensions_of(element_id, dim);},
                None => ()
            }
            true
        },
        None => {
            ui.change_visibility_of_group((sequence.get_elems_func)(), false);
            false
        }
    }
}