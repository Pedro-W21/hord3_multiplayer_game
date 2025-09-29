use cosmic_text::{Color, Metrics};
use hord3::defaults::default_ui::simple_ui::{TextCentering, UIDimensions, UIElement, UIElementBackground, UIElementContent, UIElementID, UIEvent, UIUnit, UIUserAction, UIVector};

use crate::client::client_tasks::GameUserEvent;

use super::{DEFAULT_BACKGROUND_COLOR, DEFAULT_CONTENT_BACKGROUND_COLOR, DEFAULT_REACT_COLOR};

pub fn get_number_config(origin:UIVector, outer_dimensions:UIDimensions, name:String, font:String, default_value:Option<i32>) -> Vec<UIElement<GameUserEvent>> {
    let mut elements = Vec::with_capacity(5);
    elements.push(
        UIElement::new(origin, outer_dimensions, UIVector::new(UIUnit::RelativeToParentOrigin(0), UIUnit::RelativeToParentOrigin(0)), None, format!("{} Counter Outer", name.clone()))
        .change_visibility(false)
        .with_child(UIElementID::Index(1))
    );
    elements.push(
        UIElement::new(UIVector::new(UIUnit::ParentHeightProportion(0.0), UIUnit::ParentHeightProportion(0.0)), UIDimensions::Decided(
            UIVector::new(
                UIUnit::ParentWidthProportion(1.0), 
                UIUnit::ParentHeightProportion(0.3)
            )),
            UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} Counter Title", name.clone()))
        .change_visibility(true)
        .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
        .with_content_background(UIElementBackground::Color(DEFAULT_CONTENT_BACKGROUND_COLOR))
        .with_content(UIElementContent::Text { text: name.clone(), centering:TextCentering::Neither, font: font.clone(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255) })
    );
    elements.push(
        UIElement::new(UIVector::new(
            UIUnit::RelativeToParentOrigin(0), 
            UIUnit::ParentHeightProportion(0.3)
        ), UIDimensions::Decided(
            UIVector::new(
                UIUnit::ParentWidthProportion(0.33), 
                UIUnit::ParentHeightProportion(0.7)
            )),
            UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} Counter Minus", name.clone()))
        .change_visibility(true)
        .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
        .with_content_background(UIElementBackground::Color(DEFAULT_CONTENT_BACKGROUND_COLOR))
        .with_content(UIElementContent::Text { text: String::from("-"), centering:TextCentering::Neither, font: font.clone(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255) })
        .with_content_background(UIElementBackground::Color(DEFAULT_REACT_COLOR))
        .with_reaction((UIUserAction::Clicking, UIEvent::User(GameUserEvent::DecreasedThatValue(name.clone()))))
        .with_reaction((UIUserAction::Clicking, UIEvent::ChangeContentBackground(1)))
    );
    elements.push(
        UIElement::new(UIVector::new(
            UIUnit::ParentWidthProportion(0.67), 
            UIUnit::ParentHeightProportion(0.3)
        ), UIDimensions::Decided(
            UIVector::new(
                UIUnit::ParentWidthProportion(0.33), 
                UIUnit::ParentHeightProportion(0.7)
            )),
            UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} Counter Plus", name.clone()))
        .change_visibility(true)
        .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
        .with_content_background(UIElementBackground::Color(DEFAULT_CONTENT_BACKGROUND_COLOR))
        .with_content(UIElementContent::Text { text: String::from("+"), centering:TextCentering::Neither, font: font.clone(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255) })
        .with_content_background(UIElementBackground::Color(DEFAULT_REACT_COLOR))
        .with_reaction((UIUserAction::Clicking, UIEvent::User(GameUserEvent::IncreasedThatValue(name.clone()))))
        .with_reaction((UIUserAction::Clicking, UIEvent::ChangeContentBackground(1)))
    );
    elements.push(
        UIElement::new(UIVector::new(
            UIUnit::ParentWidthProportion(0.33), 
            UIUnit::ParentHeightProportion(0.3)
        ), UIDimensions::Decided(
            UIVector::new(
                UIUnit::ParentWidthProportion(0.34), 
                UIUnit::ParentHeightProportion(0.7)
            )),
            UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} Counter Show", name.clone()))
        .change_visibility(true)
        .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
        .with_content_background(UIElementBackground::Color(DEFAULT_CONTENT_BACKGROUND_COLOR))
        .with_content(UIElementContent::Text { text: format!("{}", default_value.unwrap_or(0)), centering:TextCentering::Neither, font: font.clone(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255) })
    );

    elements
}