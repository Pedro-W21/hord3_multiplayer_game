use hord3::defaults::default_ui::simple_ui::{UIDimensions, UIElement, UIUnit, UIVector};

use crate::{game_engine::CoolVoxelType, client::client_tasks::GameUserEvent, gui_elements::image_title_desc::get_image_title_desc};

use super::{list_choice::get_list_choice, number_config::get_number_config};

pub fn light_spreader_elts() -> Vec<Vec<UIElement<GameUserEvent>>> {
    let mut elements = Vec::with_capacity(16);
    elements.push(get_number_config(
        UIVector::new(UIUnit::RelativeToParentOrigin(0), UIUnit::RelativeToParentOrigin(0)),
        UIDimensions::Decided(UIVector::new(UIUnit::ParentWidthProportion(0.15), UIUnit::ParentHeightProportion(0.1))),
        "Light Strength".to_string(),
        "rien".to_string(),
        Some(255)
    ));
    elements.push(get_number_config(
        UIVector::new(UIUnit::RelativeToParentOrigin(0), UIUnit::ParentHeightProportion(0.1)),
        UIDimensions::Decided(UIVector::new(UIUnit::ParentWidthProportion(0.15), UIUnit::ParentHeightProportion(0.1))),
        "Light Red Color".to_string(),
        "rien".to_string(),
        Some(255)
    ));
    elements.push(get_number_config(
        UIVector::new(UIUnit::RelativeToParentOrigin(0), UIUnit::ParentHeightProportion(0.2)),
        UIDimensions::Decided(UIVector::new(UIUnit::ParentWidthProportion(0.15), UIUnit::ParentHeightProportion(0.1))),
        "Light Green Color".to_string(),
        "rien".to_string(),
        Some(255)
    ));
    elements.push(get_number_config(
        UIVector::new(UIUnit::RelativeToParentOrigin(0), UIUnit::ParentHeightProportion(0.3)),
        UIDimensions::Decided(UIVector::new(UIUnit::ParentWidthProportion(0.15), UIUnit::ParentHeightProportion(0.1))),
        "Light Blue Color".to_string(),
        "rien".to_string(),
        Some(255)
    ));
    elements
}

pub fn voxel_type_choice(voxels:Vec<CoolVoxelType>, name_of_choice:String) -> Vec<Vec<UIElement<GameUserEvent>>> {
    let mut names = Vec::with_capacity(voxels.len());
    for voxel in voxels {
        names.push(voxel.name.clone());
    }
    vec![
        get_list_choice(
            names,
            UIVector::new(UIUnit::ParentWidthProportion(0.0), UIUnit::ParentHeightProportion(0.0)),
            UIDimensions::Decided(UIVector::new(UIUnit::ParentWidthProportion(0.1), UIUnit::ParentHeightProportion(0.5))),
            name_of_choice.clone(),
            "rien".to_string()
        ),
        get_image_title_desc(
            UIVector::new(UIUnit::ParentWidthProportion(0.8), UIUnit::RelativeToParentOrigin(0)),
            UIDimensions::Decided(UIVector::new(UIUnit::ParentWidthProportion(0.2), UIUnit::ParentWidthProportion(0.1))),
            name_of_choice,
            "arbre.png".to_string(),
            "Default title".to_string(),
            "Default Description".to_string(),
            "rien".to_string()
        )
    ]
    
}