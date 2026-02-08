use std::{collections::HashSet, f32::consts::PI, sync::{atomic::{AtomicI32, AtomicU8, Ordering}, Arc}};

use crossbeam::channel::Receiver;
use hord3::horde::{frontend::{MouseState, WindowingEvent, WindowingEventVariant, interact::Button}, game_engine::multiplayer::MustSync, geometry::{rotation::Orientation, vec3d::{Vec3D, Vec3Df}}, rendering::camera::Camera};

use crate::{driver::{GameEntityEvent, GameEntityVecRead, actions::{Action, ActionKind, ActionSource, ActionTimer, ActionsEvent, ActionsUpdate}}, game_engine::CoolGameEngineTID};

pub struct GameInputHandler {
    last_mouse_pos:(i32,i32,i8),
    current_mouse_pos:MouseState,
    outside_events:Receiver<WindowingEvent>,
    last_camera_used:Camera,
    sensitivity:f32,
    current_keyboard:HashSet<Button>,
    previous_keyboard:HashSet<Button>,

}


impl GameInputHandler {
    pub fn new(current_mouse_pos:MouseState, sensitivity:f32, receiver:Receiver<WindowingEvent>) -> Self {
        Self {current_keyboard:HashSet::new(), previous_keyboard:HashSet::new(), last_mouse_pos: (0,0,0), current_mouse_pos, last_camera_used: Camera::new(Vec3Df::new(15.0, 50.0, -60.0), Orientation::zero()), sensitivity, outside_events:receiver }
    }
    pub fn is_newly_pressed(&self, button:&Button) -> bool {
        self.current_keyboard.contains(button) && !self.previous_keyboard.contains(button)
    }
    pub fn update_keyboard(&mut self) {
        self.previous_keyboard = self.current_keyboard.clone();
        self.current_keyboard.clear();
        while let Ok(evt) = self.outside_events.try_recv() {
            match evt.get_variant() {
                WindowingEventVariant::KeyPress(button) => {
                    self.current_keyboard.insert(button);
                },
                _ => ()
            }
        }
    }
    pub fn get_current_keyboard(&self) -> HashSet<Button> {
        self.current_keyboard.clone()
    }
    pub fn get_new_camera<'a>(&mut self, first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>, tick:usize) -> Camera {
        self.current_mouse_pos.update_local();
        let new_mouse_pos = (self.current_mouse_pos.get_current_state().x, self.current_mouse_pos.get_current_state().y, self.current_mouse_pos.get_current_state().left);
        let delta = (new_mouse_pos.0 - self.last_mouse_pos.0, new_mouse_pos.1 - self.last_mouse_pos.1);
        self.last_camera_used.orient.yaw += (delta.0 as f32 * 0.001 * self.sensitivity * PI);
        self.last_camera_used.orient.roll += (delta.1 as f32 * 0.001 * self.sensitivity * PI);
        self.last_camera_used.orient.roll = self.last_camera_used.orient.roll.clamp(0.0, PI);
        let speed_coef = if self.current_keyboard.contains(&Button::R) {
            2.1
        }
        else {
            0.2
        };
        if self.current_keyboard.contains(&Button::SpaceBar) {
            self.last_camera_used.pos.z += speed_coef;
        }
        if self.current_keyboard.contains(&Button::LShift) {
            self.last_camera_used.pos.z -= speed_coef;
        }
        
        if self.current_keyboard.contains(&Button::W) {
            self.last_camera_used.pos += Orientation::new(self.last_camera_used.orient.yaw - PI/2.0, PI/2.0, 0.0).into_vec() * speed_coef;
        }
        if self.current_keyboard.contains(&Button::S) {
            self.last_camera_used.pos += Orientation::new(self.last_camera_used.orient.yaw + PI/2.0, PI/2.0, 0.0).into_vec() * speed_coef;
        }

        if self.current_keyboard.contains(&Button::I) {
            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Throttle(1.0), ActionSource::Director)))));
        }
        if self.current_keyboard.contains(&Button::K) {
            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Throttle(-0.3), ActionSource::Director)))));
        }
        if self.current_keyboard.contains(&Button::J) {
            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Turn(-0.02), ActionSource::Director)))));
        }
        if self.current_keyboard.contains(&Button::L) {
            first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Turn(0.02), ActionSource::Director)))));
        }
        
        /*while let Ok(evt) = self.outside_events.try_recv() {
            match evt.get_variant() {
                WindowingEventVariant::KeyPress(button) => match button {
                    Button::SpaceBar => self.last_camera_used.pos.z += 0.3,
                    Button::LShift => self.last_camera_used.pos.z -= 0.3,
                    Button::W => self.last_camera_used.pos += Orientation::new(self.last_camera_used.orient.yaw - PI/2.0, PI/2.0, 0.0).into_vec() * 0.2,
                    Button::S => self.last_camera_used.pos += Orientation::new(self.last_camera_used.orient.yaw + PI/2.0, PI/2.0, 0.0).into_vec() * 0.2,
                    _ => ()
                },
                _ => ()
            }
        }*/
        self.last_mouse_pos = new_mouse_pos;
        self.last_camera_used.clone()
    }
}