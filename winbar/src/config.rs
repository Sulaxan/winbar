use std::{
    fs,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use winbar::{color::Color, Component};

use crate::{
    component_impl::{
        datetime::DateTimeComponent, manager::ComponentLocation, static_text::StaticTextComponent,
    },
    COMPONENT_GAP, DEFAULT_BG_COLOR, DEFAULT_FG_COLOR, DEFAULT_FONT, DEFAULT_FONT_SIZE, HEIGHT,
    POSITION_X, POSITION_Y, WIDTH,
};

fn default_component_gap() -> i32 {
    10
}

fn default_font_size() -> i32 {
    18
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
    #[serde(default = "default_font_size")]
    pub default_font_size: i32,
    pub components: Vec<ComponentConfig>,
}

impl Config {
    pub fn read(path: &PathBuf) -> Result<Self> {
        serde_json::from_slice(&fs::read(path).with_context(|| "Could not read path")?)
            .with_context(|| "Could not parse config")
    }

    pub fn write(&self, path: &PathBuf) -> Result<()> {
        fs::write(path, serde_json::to_vec_pretty(self)?).with_context(|| "Could not write config")
    }

    pub fn set_global_constants(&self) -> Result<()> {
        WIDTH.store(self.window_width, Ordering::SeqCst);
        HEIGHT.store(self.window_height, Ordering::SeqCst);
        POSITION_X.store(self.position_x, Ordering::SeqCst);
        POSITION_Y.store(self.position_y, Ordering::SeqCst);
        COMPONENT_GAP.store(self.component_gap, Ordering::SeqCst);
        DEFAULT_FONT_SIZE.store(self.default_font_size, Ordering::SeqCst);
        {
            let mut bg_color = DEFAULT_BG_COLOR
                .lock()
                .map_err(|e| anyhow!("Could not obtain default background color lock: {}", e))?;
            *bg_color = self.default_bg_color.clone();
        }
        {
            let mut fg_color = DEFAULT_FG_COLOR
                .lock()
                .map_err(|e| anyhow!("Could not obtain default foreground color lock: {}", e))?;
            *fg_color = self.default_fg_color.clone();
        }
        {
            let mut font = DEFAULT_FONT
                .lock()
                .map_err(|e| anyhow!("Could not obtain default foreground color lock: {}", e))?;
            *font = self.default_font.to_string();
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_width: 1080,
            window_height: 20,
            position_x: 0,
            position_y: 0,
            component_gap: 10,
            default_bg_color: Color::Rgb {
                r: 23,
                g: 23,
                b: 23,
            },
            default_fg_color: Color::Rgb {
                r: 33,
                g: 181,
                b: 80,
            },
            default_font: "Segoe IO Variable".to_string(),
            default_font_size: 18,
            components: vec![
                ComponentConfig {
                    location: ComponentLocation::LEFT,
                    component: ComponentData::StaticText {
                        text: "Winbar!".to_string(),
                        padding_x: 10,
                    },
                },
                ComponentConfig {
                    location: ComponentLocation::LEFT,
                    component: ComponentData::DateTime {
                        format: "%F %r".to_string(),
                        bg_color: Color::Rgb {
                            r: 23,
                            g: 23,
                            b: 23,
                        },
                        fg_color: Color::Rgb {
                            r: 33,
                            g: 181,
                            b: 80,
                        },
                    },
                },
            ],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ComponentConfig {
    pub location: ComponentLocation,
    pub component: ComponentData,
}

#[derive(Serialize, Deserialize)]
pub enum ComponentData {
    StaticText {
        text: String,
        padding_x: i32,
    },
    DateTime {
        format: String,
        bg_color: Color,
        fg_color: Color,
    },
}

impl ComponentData {
    pub fn to_component(&self) -> Arc<dyn Component + Sync + Send> {
        match self {
            Self::StaticText { text, padding_x } => Arc::new(StaticTextComponent::new(
                text.to_string(),
                *padding_x,
            )),
            Self::DateTime {
                format,
                fg_color: _,
                bg_color: _,
            } => Arc::new(DateTimeComponent::new(format.to_string())),
        }
    }
}
