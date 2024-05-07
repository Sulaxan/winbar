use std::{
    fs,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use winbar::{color::Color, styles::StyleOptions, Component};

use crate::{
    component_impl::{
        datetime::DateTimeComponent, manager::ComponentLocation, static_text::StaticTextComponent,
    },
    COMPONENT_GAP, DEFAULT_BG_COLOR, DEFAULT_FG_COLOR, DEFAULT_FONT, DEFAULT_FONT_SIZE, HEIGHT,
    POSITION_X, POSITION_Y, STATUS_BAR_BG_COLOR, WIDTH,
};

fn default_component_gap() -> i32 {
    10
}

fn default_font_size() -> i32 {
    18
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// The width of the window
    pub window_width: i32,
    /// The height of the window
    pub window_height: i32,
    /// The x position of the window
    #[serde(default)]
    pub position_x: i32,
    /// The y position of the window
    #[serde(default)]
    pub position_y: i32,
    /// The gap, in pixels, between components
    #[serde(default = "default_component_gap")]
    pub component_gap: i32,
    /// The background color of the status bar
    pub status_bar_bg_color: Color,
    /// The default background color of components
    pub default_bg_color: Color,
    /// The default foreground color of components
    pub default_fg_color: Color,
    /// The default font of components
    pub default_font: String,
    /// The default font size of components
    #[serde(default = "default_font_size")]
    pub default_font_size: i32,
    /// All components that should be displayed in the status bar
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
            let mut status_bar_bg_color = STATUS_BAR_BG_COLOR
                .lock()
                .map_err(|e| anyhow!("Could not obtain status bar background color lock: {}", e))?;
            *status_bar_bg_color = self.status_bar_bg_color.clone();
        }
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
            status_bar_bg_color: Color::Transparent,
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
                        styles: StyleOptions {
                            padding_x: 10,
                            ..Default::default()
                        },
                    },
                },
                ComponentConfig {
                    location: ComponentLocation::LEFT,
                    component: ComponentData::DateTime {
                        format: "%F %r".to_string(),
                        styles: StyleOptions {
                            padding_x: 10,
                            ..Default::default()
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
        styles: StyleOptions,
    },
    DateTime {
        format: String,
        styles: StyleOptions,
    },
}

impl ComponentData {
    pub fn to_component(&self) -> Arc<dyn Component + Sync + Send> {
        match self {
            Self::StaticText { text, styles } => {
                Arc::new(StaticTextComponent::new(text.to_string(), styles.clone()))
            }
            Self::DateTime { format, styles } => {
                Arc::new(DateTimeComponent::new(format.to_string(), styles.clone()))
            }
        }
    }
}
