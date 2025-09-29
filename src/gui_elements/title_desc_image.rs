use cosmic_text::{Color, Metrics};
use hord3::defaults::default_ui::simple_ui::{TextCentering, UIDimensions, UIElement, UIElementBackground, UIElementContent, UIElementID, UIUnit, UIVector};

use crate::client::client_tasks::GameUserEvent;

use super::{DEFAULT_BACKGROUND_COLOR, DEFAULT_CONTENT_BACKGROUND_COLOR};

pub fn get_title_desc_image(origin:UIVector, outside_dims:UIDimensions, widget_name:String, image_name:String, title:String, desc:String, font:String) -> Vec<UIElement<GameUserEvent>> {
    let mut elements = Vec::with_capacity(4);
    elements.push( UIElement::new(origin, outside_dims, UIVector::new(UIUnit::RelativeToParentOrigin(0), UIUnit::RelativeToParentOrigin(0)), None, format!("{} Image Outer", widget_name.clone()))
        .change_visibility(false)
        .with_child(UIElementID::Index(1))
        .with_child(UIElementID::Index(2))
        .with_child(UIElementID::Index(3))
    );
    elements.push(
        UIElement::new(UIVector::new(UIUnit::ParentWidthProportion(0.5), UIUnit::ParentHeightProportion(0.1)), UIDimensions::Decided(
            UIVector::new(
                UIUnit::ParentWidthProportion(0.5), 
                UIUnit::ParentHeightProportion(0.9)
            )),
            UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} Image Show", widget_name.clone()))
        .change_visibility(true)
        .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
        .with_content_background(UIElementBackground::Image(image_name))
    );
    elements.push(
        UIElement::new(UIVector::new(UIUnit::ParentWidthProportion(0.0), UIUnit::ParentHeightProportion(0.0)), UIDimensions::Decided(
            UIVector::new(
                UIUnit::ParentWidthProportion(1.0), 
                UIUnit::ParentHeightProportion(0.1)
            )),
            UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} Image Title", widget_name.clone()))
        .change_visibility(true)
        .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
        .with_content_background(UIElementBackground::Color(DEFAULT_CONTENT_BACKGROUND_COLOR))
        .with_content(UIElementContent::Text { text: title.clone(), font: font.clone(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255), centering:TextCentering::Neither })
    );
    elements.push(
        UIElement::new(UIVector::new(UIUnit::ParentWidthProportion(0.0), UIUnit::ParentHeightProportion(0.1)), UIDimensions::Decided(
            UIVector::new(
                UIUnit::ParentWidthProportion(0.5), 
                UIUnit::ParentHeightProportion(0.9)
            )),
            UIVector::new(UIUnit::RelativeToParentOrigin(2), UIUnit::RelativeToParentOrigin(2)), Some(UIElementID::Index(0)), format!("{} Image Desc", widget_name.clone()))
        .change_visibility(true)
        .with_background(UIElementBackground::Color(DEFAULT_BACKGROUND_COLOR))
        .with_content_background(UIElementBackground::Color(DEFAULT_CONTENT_BACKGROUND_COLOR))
        .with_content(UIElementContent::Text { text: desc.clone(), font: font.clone(), metrics: Metrics::new(25.0, 30.0), color: Color::rgb(255, 255, 255), centering:TextCentering::Neither })
    );
    elements
}