use std::{
    sync::{atomic::Ordering, Arc},
    thread::{self, JoinHandle},
};

use anyhow::{anyhow, Result};
use getset::Getters;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use winbar::{util::rect::Rect, Component, WinbarContext};
use windows::Win32::{Foundation::HWND, Graphics::Gdi::HDC};

use crate::{COMPONENT_GAP, HEIGHT, WIDTH};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ComponentLocation {
    LEFT,
    MIDDLE,
    RIGHT,
}

#[derive(Getters)]
pub struct ComponentState {
    #[getset(get = "pub")]
    location_intention: ComponentLocation,
    #[getset(get = "pub")]
    location: Rect,
    #[getset(get = "pub")]
    component: Arc<dyn Component + Send + Sync>,
    thread: JoinHandle<()>,
}

impl ComponentState {
    pub fn stop(self) -> Result<()> {
        self.thread
            .join()
            .map_err(|e| anyhow!("component thread join error: {:?}", e))
    }
}

pub struct ComponentManager {
    components: Vec<ComponentState>,
    hwnd: HWND,
}

impl ComponentManager {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            components: Vec::new(),
            hwnd,
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub fn draw_all(&self, hwnd: HWND, hdc: HDC) {
        tracing::debug!("Drawing {} components", self.components.len());
        self.components
            .iter()
            .for_each(|state| state.component.draw(hwnd, state.location, hdc))
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: Fn(&ComponentState),
    {
        self.components.iter().for_each(f);
    }

    /// Add a new component to be managed. The component will be started immediately.
    pub fn add(
        &mut self,
        location: ComponentLocation,
        component: Arc<dyn Component + Send + Sync>,
        ctx: WinbarContext,
    ) {
        let hwnd = self.hwnd;
        let cloned_component = component.clone();

        let handle = thread::spawn(move || cloned_component.start(ctx, hwnd));

        self.components.push(ComponentState {
            location_intention: location,
            location: Rect::default(),
            component,
            thread: handle,
        })
    }

    #[instrument(level = "trace", skip(self))]
    pub fn compute_locations(&mut self, hwnd: HWND, hdc: HDC) {
        let width = WIDTH.load(Ordering::SeqCst);
        let height = HEIGHT.load(Ordering::SeqCst);
        let gap = COMPONENT_GAP.load(Ordering::SeqCst);

        let mut curr_loc_x = 0;

        // left
        self.components
            .iter_mut()
            .filter(|state| state.location_intention == ComponentLocation::LEFT)
            .for_each(|state| {
                let component_width = state.component.width(hwnd, hdc);
                state.location = Rect {
                    x: curr_loc_x,
                    y: 0,
                    width: component_width,
                    height,
                };
                curr_loc_x += component_width + gap;
            });

        // right
        curr_loc_x = width;
        self.components
            .iter_mut()
            .filter(|state| state.location_intention == ComponentLocation::RIGHT)
            .for_each(|state| {
                let component_width = state.component.width(hwnd, hdc);
                state.location = Rect {
                    x: curr_loc_x - component_width,
                    y: 0,
                    width: component_width,
                    height,
                };
                curr_loc_x -= component_width + gap;
            });

        // middle
        // FIXME: a bit inefficient, change in the future...
        let mut total_components = 0;
        let total_width = if let Some(width) = self
            .components
            .iter_mut()
            .filter_map(|state| {
                if state.location_intention == ComponentLocation::MIDDLE {
                    total_components += 1;
                    Some(state.component.width(hwnd, hdc))
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
        curr_loc_x = width / 2 - (total_width + (gap - 1) * total_components) / 2;
        self.components
            .iter_mut()
            .filter(|c| c.location_intention == ComponentLocation::MIDDLE)
            .for_each(|state| {
                let component_width = state.component.width(hwnd, hdc);
                state.location = Rect {
                    x: curr_loc_x,
                    y: 0,
                    width: component_width,
                    height,
                };
                curr_loc_x += component_width + gap;
            });
    }
}
