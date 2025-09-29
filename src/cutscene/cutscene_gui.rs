use std::time::{Duration, Instant};

use hord3::{defaults::default_ui::simple_ui::{UIDimensions, UIElement, UIUnit, UIVector}, horde::{game_engine::multiplayer::Identify, geometry::rotation::Orientation}};

use crate::client::client_tasks::GameUserEvent;

pub struct UIPos {
    pub pos:UIVector,
    pub dim:Option<UIDimensions>,
}

impl UIPos {
    pub fn empty() -> Self {
        Self { pos: UIVector::zero(), dim:None}
    }
}

#[derive(Clone)]
pub enum UIMovementElement {
    MoveFromToLinear {from:UIVector, to:UIVector},
    ChangeDimsFromToLinear {from:UIVector, to: UIVector},
    MovementShake {ranges:UIVector},
    StayPut,
}


#[derive(Clone)]
pub enum UIMovementDuration {
    Ticks{number:usize},
    RealTime{duration:Duration}
}

impl UIMovementDuration {
    pub fn from_start(&self, start:StartTime) -> Self {
        match start {
            StartTime::Tick { number } => match self {
                UIMovementDuration::Ticks { number:number_elapsed } => Self::Ticks { number: *number_elapsed + 1 },
                _ => panic!("NOT TICKS"),
            },
            StartTime::RealTime { instant } => match self {
                UIMovementDuration::RealTime { duration } => Self::RealTime { duration: instant.elapsed() },
                _ => panic!("NOT REAL TIME")
            }
        }
    }
}

#[derive(Clone)]
pub enum StartTime {
    Tick{number:usize},
    RealTime {instant:Instant}
}

pub struct UISequence {
    entity_movements:Vec<UIMovement>,
    current_sequence_number:usize,
    current_movement:Option<UIMovement>,
    movement_start_time:StartTime,
    duration_since_start_time:UIMovementDuration,
    ui_element:UIElement<GameUserEvent>,
    pub get_elems_func:Box<dyn Fn() -> Vec<UIElement<GameUserEvent>>>
}

impl UISequence {
    pub fn advance_sequence(&mut self) -> Option<UIPos> {
        self.duration_since_start_time = self.duration_since_start_time.from_start(self.movement_start_time.clone());
        match &self.current_movement {
            Some(movement) => {
                if movement.reached_total_time(self.duration_since_start_time.clone()) {
                    self.current_sequence_number += 1;
                    self.current_movement = self.entity_movements.get(self.current_sequence_number).cloned();
                    match &self.current_movement {
                        Some(movement) => {
                            match movement.duration {
                                UIMovementDuration::RealTime { duration } => {
                                    self.movement_start_time = StartTime::RealTime { instant: Instant::now() };
                                    self.duration_since_start_time = UIMovementDuration::RealTime { duration: Duration::from_secs_f32(0.0) };
                                },
                                UIMovementDuration::Ticks { number } => {
                                    self.movement_start_time = StartTime::Tick { number: 0 };
                                    self.duration_since_start_time = UIMovementDuration::Ticks { number: 0 }
                                }
                            }
                            Some(movement.get_position_and_rotation(self.duration_since_start_time.clone()))
                        },
                        None => None
                    }
                }
                else {
                    Some(movement.get_position_and_rotation(self.duration_since_start_time.clone()))
                }
            },
            None => {
                if self.current_sequence_number < self.entity_movements.len() {
                    self.current_movement = self.entity_movements.get(self.current_sequence_number).cloned();
                    match &self.current_movement {
                        Some(movement) => {
                            match movement.duration {
                                UIMovementDuration::RealTime { duration } => {
                                    self.movement_start_time = StartTime::RealTime { instant: Instant::now() };
                                    self.duration_since_start_time = UIMovementDuration::RealTime { duration: Duration::from_secs_f32(0.0) };
                                },
                                UIMovementDuration::Ticks { number } => {
                                    self.movement_start_time = StartTime::Tick { number: 0 };
                                    self.duration_since_start_time = UIMovementDuration::Ticks { number: 0 }
                                }
                            }
                            Some(movement.get_position_and_rotation(self.duration_since_start_time.clone()))
                        },
                        None => None
                    }
                }
                else {
                    None
                }
            }
        }
    }
    pub fn new<F:Fn() -> Vec<UIElement<GameUserEvent>> + 'static>(movements:Vec<UIMovement>, ui_element:UIElement<GameUserEvent>, elems_create_func:F) -> Self {
        Self {get_elems_func:Box::new(elems_create_func),entity_movements:movements, ui_element, current_sequence_number:0, current_movement:None, movement_start_time:StartTime::Tick { number: 0 }, duration_since_start_time: UIMovementDuration::Ticks { number: 0 }}
    }
    pub fn get_element(&self) -> UIElement<GameUserEvent> {
        self.ui_element.clone()
    }
}
#[derive(Clone)]
pub struct UIMovement {
    elements:Vec<UIMovementElement>,
    duration:UIMovementDuration,
}

impl UIMovement {
    pub fn new(elements:Vec<UIMovementElement>, duration:UIMovementDuration) -> Self {
        Self { elements, duration }
    }
    pub fn get_position_and_rotation(&self, time_since_started:UIMovementDuration) -> UIPos {
        let mut coefficient = match time_since_started {
            UIMovementDuration::Ticks { number } => match self.duration {
                UIMovementDuration::Ticks { number:number_total } => (number as f32)/(number_total as f32),
                _ => panic!("NOT POSSIBLE"), 
            },
            UIMovementDuration::RealTime { duration } => match self.duration {
                UIMovementDuration::RealTime { duration:duration_total } => duration.as_secs_f32()/duration_total.as_secs_f32(),
                _ => panic!("NOT POSSIBLE EITHER"),
            },
        };
        if coefficient < 1.0 {
            let mut cam = UIPos::empty();
            for element in &self.elements {
                match element {
                    UIMovementElement::MoveFromToLinear { from, to } => {
                        let x = match from.x {
                            UIUnit::ParentHeightProportion(start) => match to.x {
                                UIUnit::ParentHeightProportion(end) => UIUnit::ParentHeightProportion((end - start)*coefficient + start),
                                _ => panic!("Mismatch of unit types")
                            },
                            UIUnit::ParentWidthProportion(start) => match to.x {
                                UIUnit::ParentWidthProportion(end) => UIUnit::ParentWidthProportion((end - start)*coefficient + start),
                                _ => panic!("Mismatch of unit types")
                            },
                            UIUnit::RelativeToParentOrigin(start) => match to.x {
                                UIUnit::RelativeToParentOrigin(end) => UIUnit::RelativeToParentOrigin(((end as f32 - start as f32)*coefficient + start as f32) as i32),
                                _ => panic!("Mismatch of unit types")
                            }
                        };
                        let y = match from.y {
                            UIUnit::ParentHeightProportion(start) => match to.y {
                                UIUnit::ParentHeightProportion(end) => UIUnit::ParentHeightProportion((end - start)*coefficient + start),
                                _ => panic!("Mismatch of unit types")
                            },
                            UIUnit::ParentWidthProportion(start) => match to.y {
                                UIUnit::ParentWidthProportion(end) => UIUnit::ParentWidthProportion((end - start)*coefficient + start),
                                _ => panic!("Mismatch of unit types")
                            },
                            UIUnit::RelativeToParentOrigin(start) => match to.y {
                                UIUnit::RelativeToParentOrigin(end) => UIUnit::RelativeToParentOrigin(((end as f32 - start as f32)*coefficient + start as f32) as i32),
                                _ => panic!("Mismatch of unit types")
                            }
                        };
                        cam.pos = UIVector::new(x, y);
                        //cam.pos = from + (to - from) * coefficient;
                    },

                    UIMovementElement::ChangeDimsFromToLinear { from, to } => {
                        let x = match from.x {
                            UIUnit::ParentHeightProportion(start) => match to.x {
                                UIUnit::ParentHeightProportion(end) => UIUnit::ParentHeightProportion((end - start)*coefficient + start),
                                _ => panic!("Mismatch of unit types")
                            },
                            UIUnit::ParentWidthProportion(start) => match to.x {
                                UIUnit::ParentWidthProportion(end) => UIUnit::ParentWidthProportion((end - start)*coefficient + start),
                                _ => panic!("Mismatch of unit types")
                            },
                            UIUnit::RelativeToParentOrigin(start) => match to.x {
                                UIUnit::RelativeToParentOrigin(end) => UIUnit::RelativeToParentOrigin(((end as f32 - start as f32)*coefficient + start as f32) as i32),
                                _ => panic!("Mismatch of unit types")
                            }
                        };
                        let y = match from.y {
                            UIUnit::ParentHeightProportion(start) => match to.y {
                                UIUnit::ParentHeightProportion(end) => UIUnit::ParentHeightProportion((end - start)*coefficient + start),
                                _ => panic!("Mismatch of unit types")
                            },
                            UIUnit::ParentWidthProportion(start) => match to.y {
                                UIUnit::ParentWidthProportion(end) => UIUnit::ParentWidthProportion((end - start)*coefficient + start),
                                _ => panic!("Mismatch of unit types")
                            },
                            UIUnit::RelativeToParentOrigin(start) => match to.y {
                                UIUnit::RelativeToParentOrigin(end) => UIUnit::RelativeToParentOrigin(((end as f32 - start as f32)*coefficient + start as f32) as i32),
                                _ => panic!("Mismatch of unit types")
                            }
                        };
                        cam.dim = Some(UIDimensions::Decided(UIVector::new(x, y)));
                    },
                    UIMovementElement::MovementShake { ranges } => {
                        //cam.pos += UIVector::new(ranges.x * ((fastrand::f32() - 0.5) * 2.0),ranges.y * ((fastrand::f32() - 0.5) * 2.0),ranges.z * ((fastrand::f32() - 0.5) * 2.0))
                    },
                    _ => ()
                }
            }
            cam
        }
        else {
            let mut cam = UIPos::empty();
            for element in &self.elements {
                match element {
                    UIMovementElement::MoveFromToLinear { from, to } => {
                        cam.pos = *to;
                    },
                    _ => ()
                }
            }
            cam
        }
        
    }
    pub fn reached_total_time(&self, time_since_started:UIMovementDuration) -> bool {
        let mut coefficient = match time_since_started {
            UIMovementDuration::Ticks { number } => match self.duration {
                UIMovementDuration::Ticks { number:number_total } => (number as f32)/(number_total as f32),
                _ => panic!("NOT POSSIBLE"), 
            },
            UIMovementDuration::RealTime { duration } => match self.duration {
                UIMovementDuration::RealTime { duration:duration_total } => duration.as_secs_f32()/duration_total.as_secs_f32(),
                _ => panic!("NOT POSSIBLE EITHER"),
            },
        };
        coefficient >= 1.0
    }
}