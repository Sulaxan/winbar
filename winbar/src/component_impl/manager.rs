use std::sync::{atomic::Ordering, Arc};

use getset::Getters;
use serde::{Deserialize, Serialize};
use tokio::task::LocalSet;
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
}

pub struct ComponentManager {
    components: Vec<ComponentState>,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
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

    pub fn start(&mut self, ctx: WinbarContext, hwnd: HWND) -> LocalSet {
        let set = LocalSet::new();

        for winbar_comp in self.components.iter_mut() {
            let component = winbar_comp.component.clone();
            let location = winbar_comp.location;
            let cloned_ctx = ctx.clone();

            set.spawn_local(async move {
                component.start(cloned_ctx, hwnd, location).await;
            });
        }

        set
    }

    pub fn add(
        &mut self,
        location: ComponentLocation,
        component: Arc<dyn Component + Send + Sync>,
    ) {
        self.components.push(ComponentState {
            location_intention: location,
            location: Rect::default(),
            component,
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
