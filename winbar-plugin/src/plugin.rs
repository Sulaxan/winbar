use std::{collections::HashMap, ffi::CStr};

use anyhow::{bail, Context, Result};
use getset::Getters;
use libloading::Library;
use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

use crate::{ComponentId, FnDraw, FnId, FnStart, FnWidth, PRect};

pub struct Plugin {
    pub id: String,
    pub lib: Library,
}

impl Plugin {
    pub fn unload(self) -> Result<()> {
        self.lib
            .close()
            .with_context(|| "Error while unloading plugin")
    }

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
                .with_context(|| "Could not find width function")?;

            func(id, hwnd, rect, hdc);
        }

        Ok(())
    }

    pub fn start(&self, id: ComponentId, hwnd: HWND, rect: PRect) -> Result<()> {
        unsafe {
            let func: libloading::Symbol<FnStart> = self
                .lib
                .get(b"start\0")
                .with_context(|| "Could not find width function")?;

            func(id, hwnd, rect)
        }

        Ok(())
    }
}

#[derive(Getters)]
pub struct PluginManager {
    #[getset(get = "pub")]
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Loads a plugin given its path.
    pub fn load(&mut self, path: &str) -> Result<()> {
        unsafe {
            let lib = Library::new(path)?;
            let id: libloading::Symbol<FnId> = lib
                .get(b"id\0")
                .with_context(|| "Could not find id function")?;

            let id_string = CStr::from_ptr(id()).to_str()?.to_string();

            let plugin = Plugin {
                id: id_string.to_owned(),
                lib,
            };

            self.plugins.insert(id_string, plugin);
        }

        Ok(())
    }

    pub fn unload(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(plugin_id) {
            return plugin.unload();
        }

        bail!("Could not find plugin with id: {}", plugin_id)
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
