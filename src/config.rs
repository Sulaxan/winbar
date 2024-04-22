use std::sync::Arc;

use crate::{color::Color, component::Component};

pub struct Config {
    pub window_width: i32,
    pub window_height: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub default_fg_color: Color,
    pub default_bg_color: Color,
    pub default_font: String,
    pub components: Vec<ComponentConfig>,
}

pub enum ComponentConfig {
    StaticText {
        text: String,
    },
    DateTime {
        format: String,
        fg_color: Color,
        bg_color: Color,
    },
}

impl ComponentConfig {
    pub fn to_component(&self) -> Arc<dyn Component> {
        todo!();
    }
}
