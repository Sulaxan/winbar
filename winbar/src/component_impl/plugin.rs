use std::{
    collections::HashMap,
    ffi::CStr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use anyhow::{bail, Result};
use async_trait::async_trait;
use lazy_static::lazy_static;
use winbar::{styles::StyleOptions, util::rect::Rect, Component, WinbarContext, WindowEvent};
use winbar_plugin::{plugin::Plugin, ComponentId};
use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

lazy_static! {
    /// Represents the current component id for plugin components. Components ids are globally
    /// unique.
    static ref COMPONENT_ID: AtomicU32 = AtomicU32::new(0);
}

pub struct PluginComponent {
    plugin: Arc<Plugin>,
    component_id: ComponentId,
    styles: Arc<StyleOptions>,
}

impl PluginComponent {
    pub fn new(plugin: Arc<Plugin>, styles: StyleOptions) -> Self {
        let component_id = COMPONENT_ID.fetch_add(1, Ordering::SeqCst);

        Self {
            plugin,
            component_id,
            styles: Arc::new(styles),
        }
    }
}

impl PluginComponent {
    pub fn load_config(&self, config: &HashMap<String, serde_json::Value>) -> Result<()> {
        let config_res = self
            .plugin
            .load_config(self.component_id, &serde_json::to_string(config)?)?;
        if !config_res.ok {
            unsafe {
                let msg = CStr::from_ptr(config_res.error_msg).to_str()?.to_string();
                bail!(msg)
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Component for PluginComponent {
    fn styles(&self) -> Arc<StyleOptions> {
        self.styles.clone()
    }

    fn width(&self, hwnd: HWND, hdc: HDC) -> i32 {
        self.plugin.width(self.component_id, hwnd, hdc).unwrap()
    }

    fn draw(&self, hwnd: HWND, rect: Rect, hdc: HDC) {
        self.plugin
            .draw(self.component_id, hwnd, rect.into(), hdc)
            .unwrap()
    }

    fn start(&self, _ctx: WinbarContext, hwnd: HWND) {
        self.plugin.start(self.component_id, hwnd).unwrap();
    }

    fn handle_event(&self, _event: WindowEvent) {}
}
