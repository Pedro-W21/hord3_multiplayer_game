use std::{collections::{HashMap, HashSet}, f32::consts::PI, sync::{Arc, atomic::{AtomicI32, AtomicU8, Ordering}}};

use crossbeam::channel::Receiver;
use hord3::horde::{frontend::{MouseState, WindowingEvent, WindowingEventVariant, interact::Button}, game_engine::multiplayer::MustSync, geometry::{rotation::{Orientation, Rotation}, vec3d::{Vec3D, Vec3Df}}, rendering::camera::Camera};

use crate::{driver::{GameEntityEvent, GameEntityVecRead, actions::{Action, ActionKind, ActionSource, ActionTimer, ActionsEvent, ActionsUpdate}}, game_engine::CoolGameEngineTID, vehicle::VehicleEntityVecRead};

pub struct GameInputHandler {
    last_mouse_pos:(i32,i32,i8),
    current_mouse_pos:MouseState,
    outside_events:Receiver<WindowingEvent>,
    last_camera_used:Camera,
    sensitivity:f32,
    current_keyboard:HashSet<Button>,
    previous_keyboard:HashSet<Button>,
    throttle_inertia:HashMap<Button, f32>,
    on_car:bool,
}


impl GameInputHandler {
    pub fn new(current_mouse_pos:MouseState, sensitivity:f32, receiver:Receiver<WindowingEvent>) -> Self {
        Self {throttle_inertia:HashMap::with_capacity(16),current_keyboard:HashSet::new(), previous_keyboard:HashSet::new(), last_mouse_pos: (0,0,0), current_mouse_pos, last_camera_used: Camera::new(Vec3Df::new(15.0, 50.0, -60.0), Orientation::zero()), sensitivity, outside_events:receiver, on_car:false }
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
                    println!("PRESSING A BUTTON");
                    self.current_keyboard.insert(button);
                },
                _ => ()
            }
        }
    }
    pub fn get_current_keyboard(&self) -> HashSet<Button> {
        self.current_keyboard.clone()
    }
    pub fn get_new_camera<'a>(&mut self, first_ent:&GameEntityVecRead<'a, CoolGameEngineTID>, second_ent:&VehicleEntityVecRead<'a, CoolGameEngineTID>, tick:usize) -> Camera {
        let mut other_input = false;
        for button in &self.current_keyboard {
            let entry = self.throttle_inertia.entry(*button).or_insert(0.0);
            *entry += 0.50;
            *entry = entry.clamp(0.0, 1.0);
        }
        for (button, throttle) in &mut self.throttle_inertia {
            if *throttle >= 0.3 {
                if *button == Button::I {
                    other_input = true;
                    //println!("THROTTLING ON CLIENT");
                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Throttle(4.5 * *throttle), ActionSource::Director).make_parallel()))));
                }
                else if *button == Button::K {
                    other_input = true;
                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Throttle(-0.3 * *throttle), ActionSource::Director).make_parallel()))));
                }
                if *button == Button::J {
                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Turn(-0.02 * *throttle), ActionSource::Director).make_parallel()))));
                }
                else if *button == Button::L {
                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Turn(0.02 * *throttle), ActionSource::Director).make_parallel()))));
                }
                *throttle *= 0.8;
                //dbg!(throttle);
            }
        }
        

        if let Some(vehicle) = first_ent.stats[0].personal_vehicle && self.on_car {
            if self.current_keyboard.contains(&Button::V) {
                self.on_car = false;
            }
            let vehicle_pos = &second_ent.position[vehicle];
            let vehicle_rotat = Rotation::from_orientation(vehicle_pos.orientation);
            self.last_camera_used = Camera { pos: vehicle_rotat.rotate(Vec3Df::new(-5.0, 0.0, 4.0)) + vehicle_pos.pos, orient: vehicle_pos.orientation + Orientation::new(PI/2.0, -vehicle_pos.orientation.pitch, vehicle_pos.orientation.pitch + PI * 0.65), fov: 90.0 };
            self.last_camera_used.clone()
        }
        else {
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


            if self.current_keyboard.contains(&Button::C) {
                self.on_car = true;
            }


            
            /*else if let Some(vehicle) = first_ent.stats[0].personal_vehicle && !other_input {
                let current_orient = second_ent.locomotion[vehicle].equipment[1].current_local_orient;
                if current_orient.yaw > 0.0 {
                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::FlushActions)));
                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Turn(-0.01), ActionSource::Director)))));
                }
                else {
                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::FlushActions)));
                    first_ent.tunnels.actions_out.send(GameEntityEvent::new(MustSync::Client, ActionsEvent::new(0, None, ActionsUpdate::AddAction(Action::new(0, tick, ActionTimer::Infinite, ActionKind::Turn(0.01), ActionSource::Director)))));
                }
            }*/
            
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
            dbg!(self.last_camera_used.clone())
        }
        
    }
}