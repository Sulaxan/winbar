use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use tracing::instrument;
use winbar_core::{color::Color, Component};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{GetWindowRect, SetWindowPos, HWND_BOTTOM, SWP_NOSIZE},
};

#[derive(Clone, PartialEq)]
pub enum ComponentLocation {
    Left,
    Middle,
    Right,
}

pub struct ComponentLayout {
    pub location: ComponentLocation,
    pub component: Arc<dyn Component + Send + Sync>,
}

pub struct ComponentWindow {
    pub hwnd: HWND,
    pub location: ComponentLocation,
}

#[derive(Default)]
pub struct StatusBar {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    component_gap: i32,
    layout: Vec<ComponentLayout>,

    component_windows: Arc<Mutex<Vec<ComponentWindow>>>,
}

impl StatusBar {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        component_gap: i32,
        layout: Vec<ComponentLayout>,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            component_gap,
            layout,
            component_windows: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&mut self) {
        let component_windows = self.component_windows.clone();

        self.layout.iter().for_each(|l| {
            let component = l.component.clone();
            let location = l.location.clone();

            thread::spawn(move || {
                let hwnd = component.create_window();
                {
                    let mut cw = component_windows.lock().unwrap();
                    cw.push(ComponentWindow { hwnd, location })
                }

                component.start(hwnd);
            });
        });

        self.update_locations();
    }

    #[instrument(level = "trace", skip(self))]
    pub fn update_locations(&mut self) {
        let mut curr_loc_x = 0;
        let mut rect = RECT::default();

        let component_windows = self.component_windows.lock().unwrap();

        // left
        component_windows
            .iter()
            .filter(|c| c.location == ComponentLocation::Left)
            .for_each(|c| {
                unsafe {
                    GetWindowRect(c.hwnd, &mut rect).unwrap();
                }
                let component_width = rect.right - rect.left;

                unsafe {
                    SetWindowPos(c.hwnd, HWND_BOTTOM, rect.left, rect.top, 0, 0, SWP_NOSIZE);
                }
                curr_loc_x += component_width + self.component_gap;
            });

        // right
        curr_loc_x = self.width;
        component_windows
            .iter()
            .filter(|c| c.location == ComponentLocation::Right)
            .for_each(|c| {
                unsafe {
                    GetWindowRect(c.hwnd, &mut rect).unwrap();
                }
                let component_width = rect.right - rect.left;

                unsafe {
                    SetWindowPos(c.hwnd, HWND_BOTTOM, rect.left, rect.top, 0, 0, SWP_NOSIZE);
                }
                curr_loc_x -= component_width + self.component_gap;
            });

        // middle
        // FIXME: a bit inefficient, change in the future...
        let mut total_components = 0;
        let total_width = if let Some(width) = component_windows
            .iter()
            .filter_map(|c| {
                if c.location == ComponentLocation::Middle {
                    total_components += 1;

                    unsafe {
                        GetWindowRect(c.hwnd, &mut rect).unwrap();
                    }

                    let component_width = rect.right - rect.left;
                    Some(component_width)
                } else {
                    None
                }
            })
            .reduce(|acc, width| acc + width)
        {
            width
        } else {
            0
        };

        // we multiply by 1 less gap since it's in between the components
        curr_loc_x =
            self.width / 2 - (total_width + (self.component_gap - 1) * total_components) / 2;
        component_windows
            .iter()
            .filter(|c| c.location == ComponentLocation::Middle)
            .for_each(|c| {
                unsafe {
                    GetWindowRect(c.hwnd, &mut rect).unwrap();
                }
                let component_width = rect.right - rect.left;

                unsafe {
                    SetWindowPos(c.hwnd, HWND_BOTTOM, rect.left, rect.top, 0, 0, SWP_NOSIZE);
                }
                curr_loc_x += component_width + self.component_gap;
            });
    }
}
