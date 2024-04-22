use std::{
    fs,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    color::Color, component::{datetime::DateTimeComponent, static_text::StaticTextComponent, Component}, COMPONENT_GAP, DEFAULT_BG_COLOR, DEFAULT_FG_COLOR, DEFAULT_FONT, HEIGHT, POSITION_X, POSITION_Y, WIDTH
};

fn default_component_gap() -> i32 {
    10
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub window_width: i32,
    pub window_height: i32,
    #[serde(default)]
    pub position_x: i32,
    #[serde(default)]
    pub position_y: i32,
    #[serde(default = "default_component_gap")]
    pub component_gap: i32,
    pub default_bg_color: Color,
    pub default_fg_color: Color,
    pub default_font: String,
    pub components: Vec<ComponentConfig>,
}

impl Config {
    pub fn read(path: PathBuf) -> Result<Self> {
        serde_json::from_slice(&fs::read(path).with_context(|| "Could not read path")?)
            .with_context(|| "Could not parse config")
    }

    pub fn write(&self, path: PathBuf) -> Result<()> {
        fs::write(path, serde_json::to_vec(self)?).with_context(|| "Could not write config")
    }

    pub fn set_global_constants(&self) -> Result<()> {
        WIDTH.store(self.window_width, Ordering::SeqCst);
        HEIGHT.store(self.window_height, Ordering::SeqCst);
        POSITION_X.store(self.position_x, Ordering::SeqCst);
        POSITION_Y.store(self.position_y, Ordering::SeqCst);
        COMPONENT_GAP.store(self.component_gap, Ordering::SeqCst);
        {
            let mut bg_color = DEFAULT_BG_COLOR
                .lock()
                .map_err(|e| anyhow!("Could not obtain default background color lock: {}", e))?;
            *bg_color = self.default_bg_color;
        }
        {
            let mut fg_color = DEFAULT_FG_COLOR
                .lock()
                .map_err(|e| anyhow!("Could not obtain default foreground color lock: {}", e))?;
            *fg_color = self.default_fg_color;
        }
        {
            let mut font = DEFAULT_FONT
                .lock()
                .map_err(|e| anyhow!("Could not obtain default foreground color lock: {}", e))?;
            *font = self.default_font;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
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
        match self {
            Self::StaticText { text } => Arc::new(StaticTextComponent::new(text.to_string())),
            Self::DateTime {
                format,
                fg_color: _,
                bg_color: _,
            } => Arc::new(DateTimeComponent::new(format.to_string())),
        }
    }
}
