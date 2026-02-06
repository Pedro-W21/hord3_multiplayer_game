use crate::cutscene::entity_movement::EntityMovement;

pub struct CutsceneSupport {
    movement_index:Option<usize>,
    movment_copy:Option<EntityMovement>
}