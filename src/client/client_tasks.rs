use hord3::{defaults::{default_rendering::vectorinator_binned::{Vectorinator}, default_ui::simple_ui::{SimpleUI, UserEvent}}, horde::{frontend::WindowingHandler, scheduler::{HordeTask, HordeTaskData, HordeTaskHandler, IndividualTask}, sound::ARWWaves}};
use task_derive::HordeTask;

use crate::{cutscene::game_shader::GameShader, game_engine::{CoolGameEngine, CoolGameEngineBase}};

#[derive(Clone, PartialEq, Hash, Eq, Debug, HordeTask)]
pub enum ClientTask {
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
    #[type_task_id = 30]
    SendMustSync,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 20]
    MultiFirstPart,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 21]
    MultiSecondPart,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 3]
    PrepareRendering,

    #[uses_type = "WindowingHandler"]
    #[max_threads = 1]
    #[type_task_id = 0]
    SendFramebuf,

    #[uses_type = "WindowingHandler"]
    #[max_threads = 1]
    #[type_task_id = 1]
    WaitForPresent,

    #[uses_type = "WindowingHandler"]
    #[max_threads = 1]
    #[type_task_id = 2]
    DoEventsAndMouse,

    #[uses_type = "Vectorinator"]
    #[uses_generic = "GameShader"]
    #[max_threads = 16]
    #[type_task_id = 0]
    RenderEverything,

    #[uses_type = "Vectorinator"]
    #[uses_generic = "GameShader"]
    #[max_threads = 1]
    #[type_task_id = 1]
    TickAllSets,

    #[uses_type = "Vectorinator"]
    #[uses_generic = "GameShader"]
    #[max_threads = 1]
    #[type_task_id = 2]
    ResetCounters,

    #[uses_type = "Vectorinator"]
    #[uses_generic = "GameShader"]
    #[max_threads = 1]
    #[type_task_id = 3]
    ClearFramebuf,

    #[uses_type = "Vectorinator"]
    #[uses_generic = "GameShader"]
    #[max_threads = 1]
    #[type_task_id = 4]
    ClearZbuf,

    #[uses_type = "Vectorinator"]
    #[uses_generic = "GameShader"]
    #[max_threads = 1]
    #[type_task_id = 5]
    ChangePhase,

    #[uses_type = "SimpleUI"]
    #[uses_generic = "GameUserEvent"]
    #[max_threads = 1]
    #[type_task_id = 0]
    DoAllUIRead,

    #[uses_type = "SimpleUI"]
    #[uses_generic = "GameUserEvent"]
    #[max_threads = 1]
    #[type_task_id = 1]
    DoAllUIWrite,

    #[uses_type = "ARWWaves"]
    #[uses_generic = "CoolGameEngine"]
    #[max_threads = 1]
    #[type_task_id = 0]
    UpdateSoundPositions,

    #[uses_type = "ARWWaves"]
    #[uses_generic = "CoolGameEngine"]
    #[max_threads = 1]
    #[type_task_id = 1]
    UpdateSoundEverythingElse,
    
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