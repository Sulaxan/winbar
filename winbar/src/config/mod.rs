use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use winbar_core::{
    styles::{BorderStyle, StyleOptions},
    Component,
};

use crate::{
    component_impl::{
        datetime::DateTimeComponent, plugin::PluginComponent, static_text::StaticTextComponent,
    },
    status_bar, PLUGIN_DIR, PLUGIN_MANAGER,
};

use self::color::ColorConfig;

mod color;

fn default_component_gap() -> i32 {
    10
}

fn default_font_size() -> i32 {
    18
}

#[derive(Serialize, Deserialize)]
pub struct StatusBarConfig {
    /// The x position of the status bar
    #[serde(default)]
    pub x: i32,
    /// The y position of the status bar
    #[serde(default)]
    pub y: i32,
    /// The width of the status bar
    pub width: i32,
    /// The height of the status bar
    pub height: i32,
    /// The layout id to apply to this status bar (as defined in `layouts`)
    pub layout_id: String,
    /// The gap, in pixels, between components
    #[serde(default = "default_component_gap")]
    pub component_gap: i32,
    /// The background color of the status bar
    #[serde(deserialize_with = "color::parse_string_or_color_config")]
    pub status_bar_bg_color: ColorConfig,
    /// The default background color of components
    #[serde(deserialize_with = "color::parse_string_or_color_config")]
    pub default_component_bg_color: ColorConfig,
    /// The default foreground color of components
    #[serde(deserialize_with = "color::parse_string_or_color_config")]
    pub default_component_fg_color: ColorConfig,
    /// The default font of components
    pub default_font: String,
    /// The default font size of components
    #[serde(default = "default_font_size")]
    pub default_font_size: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Status bar defintions
    pub status_bars: Vec<StatusBarConfig>,
    /// The path of the plugin directory
    pub plugin_dir: PathBuf,
    /// Component layout definitions
    pub layouts: Vec<ComponentLayout>,
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
        {
            if !self.plugin_dir.is_dir() {
                bail!(
                    "Invalid plugin directory: {}",
                    self.plugin_dir
                        .to_str()
                        .ok_or_else(|| anyhow!("Directory string not valid unicode"))?
                );
            }

            let mut plugin_dir = PLUGIN_DIR.lock().unwrap();
            *plugin_dir = self.plugin_dir.clone();
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            status_bars: Vec::new(),
            plugin_dir: PathBuf::new(),
            // layouts: vec![
            //     ComponentLayout {
            //         location: ComponentLocation::LEFT,
            //         component: ComponentType::StaticText {
            //             text: "Winbar!".to_string(),
            //             styles: StyleConfig {
            //                 padding_x: 10,
            //                 ..Default::default()
            //             },
            //         },
            //     },
            //     ComponentLayout {
            //         location: ComponentLocation::LEFT,
            //         component: ComponentType::DateTime {
            //             format: "%F %r".to_string(),
            //             styles: StyleConfig {
            //                 padding_x: 10,
            //                 ..Default::default()
            //             },
            //         },
            //     },
            // ],
            layouts: Vec::new(),
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
    #[serde(deserialize_with = "color::parse_string_or_color_config", default)]
    pub bg_color: ColorConfig,
    #[serde(deserialize_with = "color::parse_string_or_color_config", default)]
    pub fg_color: ColorConfig,
    #[serde(default)]
    pub border_style: BorderStyleConfig,
    pub font: Option<String>,
    pub font_size: Option<i32>,
    #[serde(default)]
    pub padding_x: i32,
}

impl From<StyleConfig> for StyleOptions {
    fn from(value: StyleConfig) -> Self {
        Self {
            bg_color: value.bg_color.into_color_option(),
            fg_color: value.fg_color.into_color_option(),
            border_style: value.border_style.into(),
            font: value.font,
            font_size: value.font_size,
            padding_x: value.padding_x,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ComponentLayout {
    /// The unique id of this config
    pub id: String,
    pub components: Vec<ComponentData>,
}

#[derive(Serialize, Deserialize)]
pub struct ComponentData {
    pub location: ComponentLocation,
    pub component: ComponentType,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ComponentLocation {
    Left,
    Middle,
    Right,
}

impl From<ComponentLocation> for status_bar::ComponentLocation {
    fn from(value: ComponentLocation) -> Self {
        match value {
            ComponentLocation::Left => Self::Left,
            ComponentLocation::Middle => Self::Middle,
            ComponentLocation::Right => Self::Right,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ComponentType {
    StaticText {
        text: String,
        styles: StyleConfig,
    },
    DateTime {
        format: String,
        styles: StyleConfig,
    },
    Plugin {
        id: String,
        styles: StyleConfig,
        #[serde(flatten)]
        other: HashMap<String, serde_json::Value>,
    },
}

impl ComponentType {
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
            Self::Plugin { id, styles, other } => {
                let mut manager = PLUGIN_MANAGER.lock().unwrap();
                let plugin_dir = PLUGIN_DIR.lock().unwrap();
                let path = plugin_dir.join(format!("{}.dll", id));
                if !path.is_file() {
                    panic!(
                        "Plugin id {} is not a valid plugin in directory {}, ({})",
                        id,
                        plugin_dir.to_str().unwrap(),
                        path.to_str().unwrap()
                    );
                }

                let plugin = manager.load(path.to_str().unwrap()).unwrap();

                let component = PluginComponent::new(plugin, styles.clone().into());
                component.load_config(other).unwrap();

                Arc::new(component)
            }
        }
    }
}
