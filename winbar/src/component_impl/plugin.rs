use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use async_trait::async_trait;
use lazy_static::lazy_static;
use winbar::{styles::StyleOptions, util::rect::Rect, Component, WinbarContext};
use winbar_plugin::ComponentId;
use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

use crate::PLUGIN_MANAGER;

lazy_static! {
    /// Represents the current component id for plugin components. Components ids are globally
    /// unique.
    static ref COMPONENT_ID: AtomicU32 = AtomicU32::new(0);
}

pub struct PluginComponent {
    plugin_id: String,
    component_id: ComponentId,
    styles: Arc<StyleOptions>,
}

impl PluginComponent {
    pub fn new(plugin_id: String, styles: Arc<StyleOptions>) -> Self {
        let component_id = COMPONENT_ID.fetch_add(1, Ordering::SeqCst);

        Self {
            plugin_id: plugin_id,
            component_id,
            styles,
        }
    }
}

#[async_trait]
impl Component for PluginComponent {
    fn styles(&self) -> Arc<StyleOptions> {
        self.styles.clone()
    }

    fn width(&self, hwnd: HWND, hdc: HDC) -> i32 {
        let manager = PLUGIN_MANAGER.lock().unwrap();
        let plugin = manager.plugins().get(&self.plugin_id).unwrap();
        plugin.width(self.component_id, hwnd, hdc).unwrap()
    }

    fn draw(&self, hwnd: HWND, rect: Rect, hdc: HDC) {
        let manager = PLUGIN_MANAGER.lock().unwrap();
        let plugin = manager.plugins().get(&self.plugin_id).unwrap();
        plugin
            .draw(self.component_id, hwnd, rect.into(), hdc)
            .unwrap()
    }

    async fn start(&self, _ctx: WinbarContext, hwnd: HWND, rect: Rect) {
        // FIXME: this will make everything hang since it is expected that the start function is a
        // long running task, and thus the lock is never released
        let manager = PLUGIN_MANAGER.lock().unwrap();
        let plugin = manager.plugins().get(&self.plugin_id).unwrap();
        plugin.start(self.component_id, hwnd, rect.into()).unwrap();
    }
}
