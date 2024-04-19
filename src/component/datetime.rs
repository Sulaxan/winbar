use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::{DateTime, Local};
use windows::Win32::{
    Foundation::{HWND, RECT, SIZE},
    Graphics::Gdi::{
        DrawTextW, GetDC, GetTextExtentPoint32W, ReleaseDC, RoundRect, DT_CENTER, DT_SINGLELINE,
        DT_VCENTER,
    },
};

use crate::windows_api::WindowsApi;

use super::Component;

pub struct DateTimeComponent {
    pub format: String,
    current_time: Arc<Mutex<DateTime<Local>>>,
    update_thread: Option<JoinHandle<()>>,
}

impl DateTimeComponent {
    pub fn new(format: String) -> Self {
        Self {
            format: format,
            current_time: Arc::new(Mutex::new(Local::now())),
            update_thread: None,
        }
    }
}

impl Component for DateTimeComponent {
    fn width(&self, hwnd: HWND) -> i32 {
        let time = self.current_time.lock().unwrap();
        let formatted_time = time.format(&self.format).to_string();
        drop(time);

        unsafe {
            let hdc = GetDC(hwnd);
            let mut length: SIZE = SIZE::default();

            GetTextExtentPoint32W(
                hdc,
                &WindowsApi::str_to_u16_slice(&formatted_time),
                &mut length,
            );

            length.cx + 20 // padding
        }
    }

    fn draw(&self, hwnd: HWND, rect: &mut RECT) {
        let current_time = self.current_time.lock().unwrap();
        let formatted_time = current_time.format(&self.format).to_string();
        drop(current_time);
        unsafe {
            let hdc = GetDC(hwnd);
            WindowsApi::set_default_styles(hdc);

            RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);
            DrawTextW(
                hdc,
                &mut WindowsApi::str_to_u16_slice(&formatted_time),
                rect,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );

            ReleaseDC(hwnd, hdc);
        }
    }

    fn start(&mut self, hwnd: HWND, rect: &mut RECT) {
        if self.update_thread.is_some() {
            return;
        }

        let time = self.current_time.clone();
        self.update_thread = Some(thread::spawn(move || loop {
            {
                let mut current = time.lock().unwrap();
                *current = Local::now();
            }

            // FIXME: need to change it so it doesn't use self
            // self.draw(hwnd, &mut rect);

            // FIXME: maybe make this configurable
            thread::sleep(Duration::from_millis(500));
        }));
    }

    fn stop(mut self) {
        if self.update_thread.is_none() {
            return;
        }

        self.update_thread.unwrap().join().unwrap();
        self.update_thread = None;
    }
}
