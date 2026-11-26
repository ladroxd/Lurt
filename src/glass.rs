//! Native window backdrop (the blur behind the glass) and rounded page corners.

use tao::window::Window;
use tao::platform::windows::WindowExtWindows;
use windows::Win32::{
    Foundation::{COLORREF, HWND},
    Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn},
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetForegroundWindow, GetWindowLongPtrW, LWA_ALPHA, SWP_HIDEWINDOW, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SetForegroundWindow,
        SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, WS_EX_LAYERED,
    },
};

/// The Lurt icon from the exe's resources (id 1), for the title bar, taskbar and Alt+Tab.
pub fn set_icon(window: &Window) {
    use windows::Win32::{
        Foundation::{HINSTANCE, LPARAM, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            GetSystemMetrics, ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_DEFAULTCOLOR, LoadImageW, SM_CXICON,
            SM_CXSMICON, SendMessageW, WM_SETICON,
        },
    };
    let hwnd = HWND(window.hwnd() as _);
    unsafe {
        let Ok(module) = GetModuleHandleW(None) else { return };
        for (kind, metric) in [(ICON_BIG, SM_CXICON), (ICON_SMALL, SM_CXSMICON)] {
            let size = GetSystemMetrics(metric);
            let id = windows::core::PCWSTR(1 as _);
            if let Ok(icon) = LoadImageW(Some(HINSTANCE(module.0)), id, IMAGE_ICON, size, size, LR_DEFAULTCOLOR) {
                SendMessageW(hwnd, WM_SETICON, Some(WPARAM(kind as usize)), Some(LPARAM(icon.0 as isize)));
            }
        }
    }
}

/// The work area (screen minus taskbar) of the monitor the window is on, in physical
/// pixels: (x, y, width, height).
pub fn work_area(window: &Window) -> Option<(i32, i32, i32, i32)> {
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow};
    unsafe {
        let mon = MonitorFromWindow(HWND(window.hwnd() as _), MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
        if !GetMonitorInfoW(mon, &mut info).as_bool() {
            return None;
        }
        let r = info.rcWork;
        Some((r.left, r.top, r.right - r.left, r.bottom - r.top))
    }
}

/// Where the mouse is on screen, in physical pixels.
pub fn cursor() -> (i32, i32) {
    let mut p = windows::Win32::Foundation::POINT::default();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut p);
    }
    (p.x, p.y)
}

/// Windows 10 keeps the blur at the size the window had when it was last shown, so
/// after a maximize or resize part of the window loses its glass (a visible rectangle).
/// Re-presenting the window makes the compositor pick up the new size.
pub fn refresh(window: &Window) {
    let hwnd = HWND(window.hwnd() as _);
    // Hiding and showing back to back, with no frame in between, so nothing flickers.
    // (Re-applying the blur, cloaking or a frame change don't help.)
    unsafe {
        let was_front = GetForegroundWindow() == hwnd;
        let keep = SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE;
        let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0, keep | SWP_HIDEWINDOW);
        let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0, keep | SWP_SHOWWINDOW);
        if was_front {
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

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
