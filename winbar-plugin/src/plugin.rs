use std::ffi::CStr;

use anyhow::{Context, Result};
use libloading::Library;
use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

use crate::{FnDraw, FnId, FnStart, FnWidth, PRect};

// #[cfg(feature = "impl")]
pub struct Plugin {
    pub id: String,
    pub lib: Library,
}

impl Plugin {
    pub fn width(&self, hwnd: HWND, hdc: HDC) -> Result<i32> {
        unsafe {
            let func: libloading::Symbol<FnWidth> = self
                .lib
                .get(b"width\0")
                .with_context(|| "Could not find width function")?;

            Ok(func(hwnd, hdc))
        }
    }

    pub fn draw(&self, hwnd: HWND, rect: PRect, hdc: HDC) -> Result<()> {
        unsafe {
            let func: libloading::Symbol<FnDraw> = self
                .lib
                .get(b"draw\0")
                .with_context(|| "Could not find width function")?;

            func(hwnd, rect, hdc);
        }

        Ok(())
    }

    pub fn start(&self, hwnd: HWND, rect: PRect) -> Result<()> {
        unsafe {
            let func: libloading::Symbol<FnStart> = self
                .lib
                .get(b"start\0")
                .with_context(|| "Could not find width function")?;

            func(hwnd, rect);
        }

        Ok(())
    }
}

pub struct PluginManager {
    pub plugins: Vec<Plugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Loads a plugin given its path.
    pub fn load(&mut self, path: &str) -> Result<()> {
        unsafe {
            let lib = Library::new(path)?;
            let id: libloading::Symbol<FnId> = lib
                .get(b"id\0")
                .with_context(|| "Could not find id function")?;

            let plugin = Plugin {
                id: CStr::from_ptr(id()).to_str()?.to_string(),
                lib,
            };

            self.plugins.push(plugin);
        }

        Ok(())
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

        println!("{}", manager.plugins[0].width(HWND(0), HDC(1)).unwrap());
    }
}
