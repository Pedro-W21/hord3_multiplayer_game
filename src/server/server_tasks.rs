use hord3::{defaults::{default_rendering::vectorinator_binned::{Vectorinator}, default_ui::simple_ui::{SimpleUI, UserEvent}}, horde::{frontend::WindowingHandler, scheduler::{HordeTask, HordeTaskData, HordeTaskHandler, IndividualTask}, sound::ARWWaves}};
use task_derive::HordeTask;

use crate::{cutscene::game_shader::GameShader, game_engine::{CoolGameEngine, CoolGameEngineBase}};

#[derive(Clone, PartialEq, Hash, Eq, Debug, HordeTask)]
pub enum ServerTask {
    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 0]
    ApplyEvents,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 8]
    #[type_task_id = 1]
    Main,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 8]
    #[type_task_id = 2]
    AfterMain,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 3]
    PrepareRendering,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 30]
    SendMustSync,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 10]
    MultiFirstPart,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 11]
    MultiSecondPart,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 12]
    MultiThirdPart,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 13]
    MultiFourthPart,


    #[uses_type = "WindowingHandler"]
    #[max_threads = 1]
    #[type_task_id = 0]
    SendFramebuf,

    #[uses_type = "WindowingHandler"]
    #[max_threads = 1]
    #[type_task_id = 1]
    WaitForPresent,
    
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum GameUserEvent {
    ClickedCoolButton,
    ClickedBadButton,
    IncreasedThatValue(String),
    DecreasedThatValue(String),
    ChoseThatValue(String, String)
}

impl UserEvent for GameUserEvent {

}

fn cool() {
    
}