use std::{
    fs,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use winbar::{
    color::Color,
    styles::{BorderStyle, StyleOptions},
    util::hex_parser,
    Component,
};

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

#[derive(Clone, Serialize, Deserialize)]
pub enum ColorConfig {
    Rgb { r: u32, g: u32, b: u32 },
    Argb { r: u32, g: u32, b: u32, alpha: u32 },
    Hex(String),
    Transparent,
}

impl From<ColorConfig> for Color {
    fn from(value: ColorConfig) -> Self {
        match value {
            ColorConfig::Rgb { r, g, b } => Color::Rgb { r, g, b },
            ColorConfig::Argb { r, g, b, alpha } => Color::Argb { r, g, b, alpha },
            ColorConfig::Hex(hex) => {
                let color = hex_parser::parse_color(&hex).unwrap();
                if let Some(alpha) = color.alpha() {
                    Color::Argb {
                        r: *color.r(),
                        g: *color.g(),
                        b: *color.b(),
                        alpha: *alpha,
                    }
                } else {
                    Color::Rgb {
                        r: *color.r(),
                        g: *color.g(),
                        b: *color.b(),
                    }
                }
            }
            ColorConfig::Transparent => Color::Transparent,
        }
    }
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
    pub status_bar_bg_color: ColorConfig,
    /// The default background color of components
    pub default_component_bg_color: ColorConfig,
    /// The default foreground color of components
    pub default_component_fg_color: ColorConfig,
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
            *status_bar_bg_color = self.status_bar_bg_color.clone().into();
        }
        {
            let mut bg_color = DEFAULT_BG_COLOR
                .lock()
                .map_err(|e| anyhow!("Could not obtain default background color lock: {}", e))?;
            *bg_color = self.default_component_bg_color.clone().into();
        }
        {
            let mut fg_color = DEFAULT_FG_COLOR
                .lock()
                .map_err(|e| anyhow!("Could not obtain default foreground color lock: {}", e))?;
            *fg_color = self.default_component_fg_color.clone().into();
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
            status_bar_bg_color: ColorConfig::Transparent,
            default_component_bg_color: ColorConfig::Rgb {
                r: 23,
                g: 23,
                b: 23,
            },
            default_component_fg_color: ColorConfig::Rgb {
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
                        styles: StyleConfig {
                            padding_x: 10,
                            ..Default::default()
                        },
                    },
                },
                ComponentConfig {
                    location: ComponentLocation::LEFT,
                    component: ComponentData::DateTime {
                        format: "%F %r".to_string(),
                        styles: StyleConfig {
                            padding_x: 10,
                            ..Default::default()
                        },
                    },
                },
            ],
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum BorderStyleConfig {
    #[default]
    Square,
    Rounded {
        radius: i32,
    },
}

impl From<BorderStyleConfig> for BorderStyle {
    fn from(value: BorderStyleConfig) -> Self {
        match value {
            BorderStyleConfig::Square => BorderStyle::Square,
            BorderStyleConfig::Rounded { radius } => BorderStyle::Rounded { radius },
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct StyleConfig {
    pub bg_color: Option<ColorConfig>,
    pub fg_color: Option<ColorConfig>,
    pub border_style: BorderStyleConfig,
    pub font: Option<String>,
    pub font_size: Option<i32>,
    pub padding_x: i32,
}

impl From<StyleConfig> for StyleOptions {
    fn from(value: StyleConfig) -> Self {
        Self {
            bg_color: value.bg_color.map(|c| c.into()),
            fg_color: value.fg_color.map(|c| c.into()),
            border_style: value.border_style.into(),
            font: value.font,
            font_size: value.font_size,
            padding_x: value.padding_x,
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
    StaticText { text: String, styles: StyleConfig },
    DateTime { format: String, styles: StyleConfig },
}

impl ComponentData {
    pub fn to_component(&self) -> Arc<dyn Component + Sync + Send> {
        match self {
            Self::StaticText { text, styles } => Arc::new(StaticTextComponent::new(
                text.to_string(),
                styles.clone().into(),
            )),
            Self::DateTime { format, styles } => Arc::new(DateTimeComponent::new(
                format.to_string(),
                styles.clone().into(),
            )),
        }
    }
}
