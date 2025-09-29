use cosmic_text::{Color, Metrics};
use hord3::defaults::default_ui::simple_ui::{TextCentering, UIDimensions, UIElement, UIElementBackground, UIElementContent, UIElementID, UIEvent, UIUnit, UIUserAction, UIVector};

use crate::client::client_tasks::GameUserEvent;

use super::{DEFAULT_BACKGROUND_COLOR, DEFAULT_CONTENT_BACKGROUND_COLOR, DEFAULT_HOVER_COLOR, DEFAULT_REACT_COLOR};

pub fn get_list_choice(choices:Vec<String>, origin:UIVector, outer_dimensions:UIDimensions, name:String, font:String) -> Vec<UIElement<GameUserEvent>> {
    let mut elements = Vec::with_capacity(5);
    elements.push(
        UIElement::new(origin, outer_dimensions, UIVector::new(UIUnit::RelativeToParentOrigin(0), UIUnit::RelativeToParentOrigin(0)), None, format!("{} List Outer", name.clone()))
        .change_visibility(false)
        .with_child(UIElementID::Index(1))
    );
    let height_of_one = 1.0/((choices.len() + 1) as f32);
    elements.push(
        UIElement::new(UIVector::new(UIUnit::ParentHeightProportion(0.0), UIUnit::ParentHeightProportion(0.0)), UIDimensions::Decided(
            UIVector::new(
                UIUnit::ParentWidthProportion(1.0), 
                UIUnit::ParentHeightProportion(height_of_one)
            )),
            UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} Counter Title", name.clone()))
        .change_visibility(true)
        .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
        .with_content_background(UIElementBackground::Color(DEFAULT_CONTENT_BACKGROUND_COLOR))
        .with_content(UIElementContent::Text { text: name.clone(), centering:TextCentering::Neither, font: font.clone(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255) })
    );
    for i in 1..(choices.len() + 1) {
        elements.push(
            UIElement::new(UIVector::new(
                UIUnit::RelativeToParentOrigin(0), 
                UIUnit::ParentHeightProportion(height_of_one * i as f32)
            ), UIDimensions::Decided(
                UIVector::new(
                    UIUnit::ParentWidthProportion(1.0), 
                    UIUnit::ParentHeightProportion(height_of_one)
                )),
                UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} List choice {}", name.clone(), choices[i - 1].clone()))
            .change_visibility(true)
            .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
            .with_content_background(UIElementBackground::Color(DEFAULT_CONTENT_BACKGROUND_COLOR))
            .with_content(UIElementContent::Text { text: choices[i - 1].clone(), centering:TextCentering::Neither, font: font.clone(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255) })
            .with_content_background(UIElementBackground::Color(DEFAULT_HOVER_COLOR))
            .with_content_background(UIElementBackground::Color(DEFAULT_REACT_COLOR))
            .with_reaction((UIUserAction::Nothing, UIEvent::ChangeContentBackground(1)))
            .with_reaction((UIUserAction::Clicking, UIEvent::User(GameUserEvent::ChoseThatValue(name.clone(), choices[i - 1].clone()))))
            .with_reaction((UIUserAction::Clicking, UIEvent::ChangeContentBackground(2)))
        );
    }


    elements
}