use std::mem::MaybeUninit;

use anyhow::{bail, Result};
use tracing::instrument;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::GdiPlus::{GdiplusShutdown, GdiplusStartup, GdiplusStartupInput, Status},
    UI::WindowsAndMessaging::{PostMessageW, ShowWindow, SW_HIDE, SW_SHOW, WM_CLOSE},
};

pub struct WindowsApi {}

impl WindowsApi {
    pub fn str_to_u16_slice(s: &str) -> Vec<u16> {
        s.encode_utf16().collect::<Vec<u16>>()
    }

    pub fn show_window(hwnd: HWND) {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
    }

    pub fn hide_window(hwnd: HWND) {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }

    #[instrument]
    pub fn send_window_shutdown_msg(hwnd: HWND) {
        if let Err(e) = unsafe { PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)) } {
            tracing::error!("Error posting WM_CLOSE message: {}", e);
        }
    }

    // inspired from: https://github.com/davidrios/gdiplus-rs
    #[instrument(name = "windows_api_gdiplus_init")]
    pub fn startup_gdiplus() -> Result<usize> {
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            SuppressBackgroundThread: false.into(),
            SuppressExternalCodecs: false.into(),
            ..Default::default()
        };

        let mut token: usize = 0;
        let mut output = MaybeUninit::uninit();
        unsafe {
            let status = GdiplusStartup(&mut token, &input, output.as_mut_ptr());
            if status != Status(0) {
                bail!("GDI+ startup returned non-zero: {:?}", status);
            }
        }

        Ok(token)
    }

    pub fn shutdown_gdiplus(token: usize) {
        unsafe {
            GdiplusShutdown(token);
        }
    }
}
