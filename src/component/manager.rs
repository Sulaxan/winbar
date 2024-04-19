use std::sync::{atomic::Ordering, Mutex};

use tokio::task::JoinSet;
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::HDC,
};

use crate::{winbar::WinbarContext, COMPONENT_GAP, HEIGHT, WIDTH};

use super::Component;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComponentLocation {
    LEFT,
    MIDDLE,
    RIGHT,
}

struct ComponentState {
    location_intention: ComponentLocation,
    location: RECT,
    component: Box<dyn Component + Send>,
}

pub struct ComponentManager {
    components: Vec<ComponentState>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn draw_all(&self, hwnd: HWND, hdc: HDC) {
        self.components
            .iter()
            .for_each(|c| c.component.draw(hwnd, c.location, hdc))
    }

    pub fn start(&mut self, ctx: WinbarContext, hwnd: HWND) -> JoinSet<()> {
        let mut set = JoinSet::<()>::new();
        for winbar_comp in self.components.drain(0..) {
            let mut component = winbar_comp.component;
            let location = winbar_comp.location;
            let cloned_ctx = ctx.clone();

            set.spawn(async move {
                component.start(cloned_ctx, hwnd, location).await;
            });
        }

        set
    }

    pub fn add(&mut self, location: ComponentLocation, component: Box<dyn Component + Send>) {
        self.components.push(ComponentState {
            location_intention: location,
            location: RECT::default(),
            component,
        })
    }

    pub fn compute_locations(&mut self, hwnd: HWND, hdc: HDC) {
        let width = WIDTH.load(Ordering::SeqCst);
        let height = HEIGHT.load(Ordering::SeqCst);
        let gap = COMPONENT_GAP.load(Ordering::SeqCst);

        let mut curr_loc_x = 0;

        // left
        self.components
            .iter_mut()
            .filter(|c| c.location_intention == ComponentLocation::LEFT)
            .for_each(|c| {
                let component_width = c.component.width(hwnd, hdc);
                c.location = RECT {
                    top: 0,
                    bottom: height,
                    left: curr_loc_x,
                    right: curr_loc_x + component_width,
                };
                curr_loc_x += component_width + gap;
            });

        // right
        curr_loc_x = width;
        self.components
            .iter_mut()
            .filter(|c| c.location_intention == ComponentLocation::RIGHT)
            .for_each(|c| {
                let component_width = c.component.width(hwnd, hdc);
                c.location = RECT {
                    top: 0,
                    bottom: height,
                    left: curr_loc_x - component_width,
                    right: curr_loc_x,
                };
                curr_loc_x -= component_width + gap;
            });

        // middle
        // FIXME: a bit inefficient, change in the future...
        let mut total_components = 0;
        let total_width = if let Some(width) = self
            .components
            .iter_mut()
            .filter_map(|c| {
                if c.location_intention == ComponentLocation::MIDDLE {
                    total_components += 1;
                    Some(c.component.width(hwnd, hdc))
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
            .for_each(|c| {
                let component_width = c.component.width(hwnd, hdc);
                c.location = RECT {
                    top: 0,
                    bottom: height,
                    left: curr_loc_x,
                    right: curr_loc_x + component_width,
                };
                curr_loc_x += component_width + gap;
            });
    }
}
