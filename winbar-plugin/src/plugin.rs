use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    sync::Arc,
};

use anyhow::{Context, Result};
use getset::Getters;
use libloading::Library;
use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

use crate::{
    ComponentId, FnDraw, FnId, FnLoadConfig, FnStart, FnStop, FnWidth, LoadConfigResult, PRect,
};

pub struct Plugin {
    pub id: String,
    pub lib: Library,
}

impl Plugin {
    pub fn width(&self, id: ComponentId, hwnd: HWND, hdc: HDC) -> Result<i32> {
        unsafe {
            let func: libloading::Symbol<FnWidth> = self
                .lib
                .get(b"width\0")
                .with_context(|| "Could not find width function")?;

            Ok(func(id, hwnd, hdc))
        }
    }

    pub fn draw(&self, id: ComponentId, hwnd: HWND, rect: PRect, hdc: HDC) -> Result<()> {
        unsafe {
            let func: libloading::Symbol<FnDraw> = self
                .lib
                .get(b"draw\0")
                .with_context(|| "Could not find draw function")?;

            func(id, hwnd, rect, hdc);
        }

        Ok(())
    }

    pub fn load_config(&self, id: ComponentId, config: &str) -> Result<LoadConfigResult> {
        unsafe {
            let func: libloading::Symbol<FnLoadConfig> = self
                .lib
                .get(b"load_config\0")
                .with_context(|| "Could not find load_config function")?;

            let c_str_config = CString::new(config).unwrap().into_raw();
            let result = func(id, c_str_config);
            drop(CString::from_raw(c_str_config));

            Ok(result)
        }
    }

    pub fn start(&self, id: ComponentId, hwnd: HWND) -> Result<()> {
        unsafe {
            let func: libloading::Symbol<FnStart> = self
                .lib
                .get(b"start\0")
                .with_context(|| "Could not find start function")?;

            func(id, hwnd)
        }

        Ok(())
    }

    pub fn stop(&self, id: ComponentId) -> Result<()> {
        unsafe {
            let func: libloading::Symbol<FnStop> = self
                .lib
                .get(b"stop\0")
                .with_context(|| "Could not find stop function")?;

            func(id)
        }

        Ok(())
    }
}

#[derive(Getters)]
pub struct PluginManager {
    #[getset(get = "pub")]
    plugins: HashMap<String, Arc<Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Loads a plugin given its path, and returns a reference to it.
    pub fn load(&mut self, path: &str) -> Result<Arc<Plugin>> {
        unsafe {
            let lib = Library::new(path)?;
            let id: libloading::Symbol<FnId> = lib
                .get(b"id\0")
                .with_context(|| "Could not find id function")?;

            let id_string = CStr::from_ptr(id()).to_str()?.to_string();

            let plugin = Arc::new(Plugin {
                id: id_string.to_owned(),
                lib,
            });

            self.plugins.insert(id_string, Arc::clone(&plugin));
            Ok(plugin)
        }
    }

    /// Unloads a plugin. Note that this method does not unload the underlying dynamic library until
    /// all references to the plugin have been dropped.
    pub fn unload(&mut self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load() {
        let mut manager = PluginManager::new();
        manager
            .load("C:\\Users\\Encast\\development\\winbar\\target\\debug\\winbar_plugin_test.dll")
            .unwrap();

        println!(
            "{}",
            manager
                .plugins()
                .get("test")
                .unwrap()
                .width(0, HWND(0), HDC(1))
                .unwrap()
        );
    }
}
