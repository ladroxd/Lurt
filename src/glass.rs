//! Native window backdrop (the blur behind the glass) and rounded page corners.

use tao::window::Window;
use tao::platform::windows::WindowExtWindows;
use windows::Win32::{
    Foundation::{COLORREF, HWND},
    Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn},
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, LWA_ALPHA, SetLayeredWindowAttributes, SetWindowLongPtrW,
        WS_EX_LAYERED,
    },
};

/// Fade the whole window, pages included. `percent` is 10..=100; 100 turns it off.
pub fn set_window_opacity(window: &Window, percent: u8) {
    let hwnd = HWND(window.hwnd() as _);
    let percent = percent.clamp(10, 100);
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let layered = WS_EX_LAYERED.0 as isize;
        if percent >= 100 {
            if ex & layered != 0 {
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex & !layered);
            }
            return;
        }
        if ex & layered == 0 {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | layered);
        }
        let alpha = (u32::from(percent) * 255 / 100) as u8;
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

/// Apply a backdrop and return the one that actually took effect.
/// Mica only exists on Windows 11; on Windows 10 it falls back to blur.
///
/// `dark` tints the blur itself (dark glass in dark mode, frosted in light mode), so
/// lowering the UI's background opacity shows more of the desktop without washing it
/// out to gray when what is behind the window is bright.
pub fn apply(window: &Window, mode: &str, dark: bool) -> &'static str {
    clear(window);
    let tint = if dark { (12, 13, 17, 165) } else { (238, 240, 245, 120) };
    let ok = |r: Result<(), window_vibrancy::Error>| r.is_ok();
    match mode {
        "none" => "none",
        "mica" if ok(window_vibrancy::apply_mica(window, Some(dark))) => "mica",
        "acrylic" if ok(window_vibrancy::apply_acrylic(window, Some(tint))) => "acrylic",
        _ if ok(window_vibrancy::apply_blur(window, Some(tint))) => "blur",
        _ => "none",
    }
}

fn clear(window: &Window) {
    let _ = window_vibrancy::clear_mica(window);
    let _ = window_vibrancy::clear_acrylic(window);
    let _ = window_vibrancy::clear_blur(window);
}

/// Clip a page view to a rounded rectangle. `w`/`h`/`radius` are physical pixels.
pub fn round(hwnd: HWND, w: i32, h: i32, radius: i32) {
    unsafe {
        if radius <= 0 {
            SetWindowRgn(hwnd, None, true);
            return;
        }
        let d = radius * 2;
        let rgn = CreateRoundRectRgn(0, 0, w + 1, h + 1, d, d);
        // The system owns the region after this call.
        SetWindowRgn(hwnd, Some(rgn), true);
    }
}
