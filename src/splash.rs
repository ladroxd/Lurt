//! Instant startup skeleton.
//!
//! The WebView2 engine takes 0.3 s (warm) to 2.5 s (first launch after boot) to start.
//! Instead of an empty window, Lurt draws a shimmering skeleton of its own layout in a
//! tiny per-pixel-alpha window laid over the real one, using no engine at all. When the
//! real UI has painted, the skeleton fades out.

use std::{cell::RefCell, ffi::c_void, time::Instant};

use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM},
        Graphics::Gdi::{
            AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
            CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, HBITMAP,
            HDC, HGDIOBJ, SelectObject,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, KillTimer, RegisterClassExW,
            SW_SHOWNOACTIVATE, SetTimer, ShowWindow, ULW_ALPHA, UpdateLayeredWindow, WNDCLASSEXW,
            WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
        },
    },
    core::w,
};

/// Whether Windows apps are set to dark mode (Settings > Personalization > Colors).
pub fn system_dark() -> bool {
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
    let mut value = 1u32;
    let mut size = size_of::<u32>() as u32;
    let ok = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        )
    };
    ok.is_ok() && value == 0
}

/// What the skeleton should look like; mirrors the CSS layout in ui/style.css.
pub struct Look {
    pub dark: bool,
    /// Background alpha, same formula as `--tint` in the UI.
    pub tint: f32,
    pub radius: f32,
    pub compact: bool,
    /// Active tab is the start page (vs. a website).
    pub blank: bool,
    pub scale: f32,
}

struct Splash {
    hwnd: HWND,
    dc: HDC,
    bmp: HBITMAP,
    old: HGDIOBJ,
    bits: *mut u32,
    base: Vec<u32>,
    mask: Vec<u8>,
    w: i32,
    h: i32,
    pos: POINT,
    shine: u32,
    started: Instant,
    fade_from: Option<Instant>,
}

thread_local! {
    static SPLASH: RefCell<Option<Splash>> = const { RefCell::new(None) };
}

const TIMER: usize = 0x4C55;
const FADE_MS: f32 = 160.0;

unsafe extern "system" fn wndproc(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(h, m, w, l) }
}

/// Show the skeleton over the client area at (`x`, `y`), `w`×`h` physical pixels.
pub fn show(owner: HWND, x: i32, y: i32, w: i32, h: i32, look: &Look) {
    if w <= 0 || h <= 0 {
        return;
    }
    unsafe {
        let Ok(module) = GetModuleHandleW(None) else { return };
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: module.into(),
            lpszClassName: w!("LurtSplash"),
            ..Default::default()
        };
        RegisterClassExW(&class);
        let Ok(hwnd) = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            w!("LurtSplash"),
            w!(""),
            WS_POPUP,
            x,
            y,
            w,
            h,
            Some(owner),
            None,
            Some(module.into()),
            None,
        ) else {
            return;
        };

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h, // top-down rows
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let dc = CreateCompatibleDC(None);
        let mut bits: *mut c_void = std::ptr::null_mut();
        let Ok(bmp) = CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) else {
            let _ = DeleteDC(dc);
            let _ = DestroyWindow(hwnd);
            return;
        };
        let old = SelectObject(dc, bmp.into());

        let (base, mask) = paint(w, h, look);
        let s = Splash {
            hwnd,
            dc,
            bmp,
            old,
            bits: bits.cast(),
            base,
            mask,
            w,
            h,
            pos: POINT { x, y },
            shine: if look.dark { 0x0E } else { 0x30 },
            started: Instant::now(),
            fade_from: None,
        };
        s.frame();
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        SPLASH.with(|c| *c.borrow_mut() = Some(s));
        // Timer callbacks also run while wry is busy starting WebView2, because wry
        // pumps window messages while it waits.
        SetTimer(Some(hwnd), TIMER, 16, Some(on_timer));
    }
}

/// Fade the skeleton out; the real UI is on screen underneath.
pub fn dismiss() {
    SPLASH.with(|c| {
        if let Some(s) = c.borrow_mut().as_mut() {
            s.fade_from.get_or_insert_with(Instant::now);
        }
    });
}

unsafe extern "system" fn on_timer(_: HWND, _: u32, _: usize, _: u32) {
    let done = SPLASH.with(|c| {
        let b = c.borrow();
        let Some(s) = b.as_ref() else { return false };
        s.frame()
    });
    if done {
        if let Some(s) = SPLASH.with(|c| c.borrow_mut().take()) {
            s.destroy();
        }
    }
}

impl Splash {
    /// Draw one animation frame. Returns true once fully faded out.
    fn frame(&self) -> bool {
        let opacity = match self.fade_from {
            Some(t) => 1.0 - t.elapsed().as_secs_f32() * 1000.0 / FADE_MS,
            None => 1.0,
        };
        if opacity <= 0.0 {
            return true;
        }
        let n = (self.w * self.h) as usize;
        let out = unsafe { std::slice::from_raw_parts_mut(self.bits, n) };
        out.copy_from_slice(&self.base);

        // Shimmer: a soft diagonal band of light sweeping across the skeleton shapes.
        let period = 1.4;
        let t = (self.started.elapsed().as_secs_f32() % period) / period;
        let span = (self.w + self.h) as f32;
        let center = -0.3 * span + t * 1.6 * span;
        let half = 0.18 * self.w as f32;
        for y in 0..self.h {
            let row = (y * self.w) as usize;
            let x0 = ((center - half - y as f32 * 0.5).max(0.0) as i32).min(self.w);
            let x1 = ((center + half - y as f32 * 0.5).max(0.0) as i32).min(self.w);
            for x in x0..x1 {
                let i = row + x as usize;
                let m = self.mask[i] as u32;
                if m == 0 {
                    continue;
                }
                let d = ((x as f32 + y as f32 * 0.5 - center) / half).abs();
                let k = ((1.0 - d).max(0.0) * m as f32 / 255.0 * self.shine as f32) as u32;
                out[i] = add_light(out[i], k);
            }
        }

        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: (opacity.clamp(0.0, 1.0) * 255.0) as u8,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let size = SIZE { cx: self.w, cy: self.h };
        let src = POINT { x: 0, y: 0 };
        unsafe {
            let _ = UpdateLayeredWindow(
                self.hwnd,
                None,
                Some(&self.pos),
                Some(&size),
                Some(self.dc),
                Some(&src),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
        }
        false
    }

    fn destroy(self) {
        unsafe {
            let _ = KillTimer(Some(self.hwnd), TIMER);
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(self.bmp.into());
            let _ = DeleteDC(self.dc);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Add `k` of white to a premultiplied BGRA pixel (alpha grows to keep it valid).
fn add_light(p: u32, k: u32) -> u32 {
    let a = (p >> 24).saturating_add(k).min(255);
    let ch = |s: u32| (((p >> s) & 0xFF) + k).min(a);
    (a << 24) | (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

// ---- software rendering ---------------------------------------------------------

struct Canvas {
    w: i32,
    h: i32,
    px: Vec<u32>,
    mask: Vec<u8>,
}

impl Canvas {
    /// Anti-aliased rounded rectangle, composited "over" in premultiplied alpha.
    /// `skeleton` shapes also receive the shimmer.
    #[allow(clippy::too_many_arguments)]
    fn rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, rgb: (u8, u8, u8), alpha: f32, skeleton: bool) {
        let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        let (hx, hy) = (w / 2.0 - r, h / 2.0 - r);
        let x0 = (x.floor() as i32).max(0);
        let x1 = ((x + w).ceil() as i32).min(self.w);
        let y0 = (y.floor() as i32).max(0);
        let y1 = ((y + h).ceil() as i32).min(self.h);
        for py in y0..y1 {
            for px in x0..x1 {
                let dx = ((px as f32 + 0.5 - cx).abs() - hx).max(0.0);
                let dy = ((py as f32 + 0.5 - cy).abs() - hy).max(0.0);
                let dist = (dx * dx + dy * dy).sqrt() - r;
                let cov = (0.5 - dist).clamp(0.0, 1.0);
                if cov <= 0.0 {
                    continue;
                }
                let i = (py * self.w + px) as usize;
                let a = alpha * cov;
                self.px[i] = over(self.px[i], rgb, a);
                if skeleton {
                    self.mask[i] = self.mask[i].max((cov * 255.0) as u8);
                }
            }
        }
    }
}

fn over(dst: u32, (r, g, b): (u8, u8, u8), a: f32) -> u32 {
    let inv = 1.0 - a;
    let ch = |s: u32, c: u8| ((c as f32 * a) + ((dst >> s) & 0xFF) as f32 * inv).round() as u32;
    let da = ((a * 255.0) + (dst >> 24) as f32 * inv).round() as u32;
    (da.min(255) << 24) | (ch(16, r).min(255) << 16) | (ch(8, g).min(255) << 8) | ch(0, b).min(255)
}

fn paint(w: i32, h: i32, look: &Look) -> (Vec<u32>, Vec<u8>) {
    let n = (w * h) as usize;
    let mut c = Canvas { w, h, px: vec![0; n], mask: vec![0; n] };
    let s = look.scale;
    // Opaque, in the colors the finished UI settles on. A see-through skeleton showed
    // whatever was behind the not-yet-painted window, which looked washed out.
    let (base, page, glass, bone) = if look.dark {
        ((20, 21, 25), (27, 29, 34), (255, 255, 255), (255, 255, 255))
    } else {
        ((222, 226, 234), (232, 235, 242), (255, 255, 255), (20, 30, 60))
    };
    let _ = look.tint;
    let (glass_a, bone_a) = if look.dark { (0.07, 0.07) } else { (0.6, 0.07) };
    let r = look.radius * s;
    let (fw, fh) = (w as f32, h as f32);

    // Window tint (the native blur shows through it, exactly as in the real UI).
    c.rrect(0.0, 0.0, fw, fh, 0.0, base, 1.0, false);

    // Toolbar: tab, new-tab button, nav buttons, address bar, actions.
    let nav_w = 4.0 * 34.0 * s + 3.0 * s;
    let acts_w = 69.0 * s;
    let (row_y, omni_x0, omni_x1);
    if look.compact {
        row_y = 10.0 * s;
        omni_x0 = 8.0 * s + nav_w + 6.0 * s;
        omni_x1 = omni_x0 + (440.0 * s).min(fw * 0.35);
        let tab_x = omni_x1 + 6.0 * s;
        c.rrect(tab_x, row_y + 3.0 * s, 210.0 * s, 32.0 * s, r * 0.78, glass, glass_a, true);
    } else {
        c.rrect(10.0 * s, 7.0 * s, 210.0 * s, 32.0 * s, r * 0.78, glass, glass_a, true);
        c.rrect(28.0 * s, 18.0 * s, 110.0 * s, 10.0 * s, 5.0 * s, bone, bone_a, true);
        c.rrect(228.0 * s, 11.0 * s, 24.0 * s, 24.0 * s, 7.0 * s, bone, bone_a * 0.7, true);
        row_y = 42.0 * s;
        omni_x0 = 8.0 * s + nav_w + 6.0 * s;
        // The address bar's grid column ends where the window buttons column begins.
        let end = fw - 184.0 * s;
        let avail = (end - omni_x0).max(0.0);
        let ow = avail.min(920.0 * s);
        let ox = omni_x0 + (avail - ow) / 2.0;
        omni_x1 = ox + ow;
        c.rrect(ox, row_y, ow, 38.0 * s, r * 0.95, glass, glass_a, true);
        c.rrect(ox + 36.0 * s, row_y + 14.0 * s, (ow * 0.28).min(240.0 * s), 10.0 * s, 5.0 * s, bone, bone_a, true);
    }
    let _ = omni_x1;
    for i in 0..4 {
        let bx = 8.0 * s + i as f32 * 35.0 * s + 8.0 * s;
        c.rrect(bx, row_y + 10.0 * s, 18.0 * s, 18.0 * s, 6.0 * s, bone, bone_a, true);
    }
    for i in 0..2 {
        let bx = fw - 8.0 * s - acts_w + i as f32 * 35.0 * s + 8.0 * s;
        c.rrect(bx, row_y + 10.0 * s, 18.0 * s, 18.0 * s, 6.0 * s, bone, bone_a, true);
    }

    // Page area.
    let top = if look.compact { 54.0 * s } else { 86.0 * s };
    let (vx, vy, vw, vh) = (8.0 * s, top, fw - 16.0 * s, fh - top - 8.0 * s);
    let pr = r * 0.7;
    c.rrect(vx - 1.0 * s, vy - 1.0 * s, vw + 2.0 * s, vh + 2.0 * s, pr + 1.0 * s, glass, if look.dark { 0.06 } else { 0.5 }, false);
    c.rrect(vx, vy, vw, vh, pr, page, 1.0, false);

    let (mx, my) = (vx + vw / 2.0, vy + vh / 2.0);
    if look.blank {
        // Start page: clock, date, search pill.
        c.rrect(mx - 120.0 * s, my - 110.0 * s, 240.0 * s, 56.0 * s, 16.0 * s, bone, bone_a, true);
        c.rrect(mx - 80.0 * s, my - 42.0 * s, 160.0 * s, 12.0 * s, 6.0 * s, bone, bone_a, true);
        let pw = (580.0 * s).min(vw * 0.84);
        c.rrect(mx - pw / 2.0, my, pw, 50.0 * s, 25.0 * s, glass, glass_a, true);
    } else {
        // A generic page: header bar, title, paragraphs.
        let px = vx + vw * 0.08;
        let pw = vw * 0.84;
        c.rrect(vx + 16.0 * s, vy + 16.0 * s, vw - 32.0 * s, 44.0 * s, 12.0 * s, bone, bone_a * 0.8, true);
        c.rrect(px, vy + 100.0 * s, pw * 0.55, 26.0 * s, 10.0 * s, bone, bone_a, true);
        let mut y = vy + 150.0 * s;
        for (i, frac) in [0.95, 0.88, 0.92, 0.6, 0.0, 0.9, 0.82, 0.94, 0.45].iter().enumerate() {
            if *frac > 0.0 && y + 14.0 * s < vy + vh - 16.0 * s {
                c.rrect(px, y, pw * frac, 12.0 * s, 6.0 * s, bone, bone_a * if i % 2 == 0 { 1.0 } else { 0.8 }, true);
            }
            y += 26.0 * s;
        }
    }
    (c.px, c.mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render the skeleton to a BMP for eyeballing: `LURT_SPLASH_OUT=dir cargo test splash -- --ignored`
    #[test]
    #[ignore]
    fn render_preview() {
        let dir = std::env::var("LURT_SPLASH_OUT").unwrap_or_else(|_| ".".into());
        for (name, dark, blank) in [("dark-start", true, true), ("light-page", false, false)] {
            let (w, h) = (1280, 800);
            let look = Look { dark, tint: 0.6, radius: 14.0, compact: false, blank, scale: 1.0 };
            let (px, _) = paint(w, h, &look);
            // Composite over a mid-tone "desktop" so transparency is visible.
            let bg = if dark { 70u32 } else { 160u32 };
            let mut data = Vec::with_capacity((w * h * 3) as usize);
            for y in (0..h).rev() {
                for x in 0..w {
                    let p = px[(y * w + x) as usize];
                    let a = p >> 24;
                    for s in [0, 8, 16] {
                        data.push((((p >> s) & 0xFF) + bg * (255 - a) / 255).min(255) as u8);
                    }
                }
            }
            let mut f = Vec::new();
            let size = 54 + data.len() as u32;
            f.extend_from_slice(b"BM");
            f.extend_from_slice(&size.to_le_bytes());
            f.extend_from_slice(&[0; 4]);
            f.extend_from_slice(&54u32.to_le_bytes());
            f.extend_from_slice(&40u32.to_le_bytes());
            f.extend_from_slice(&(w as i32).to_le_bytes());
            f.extend_from_slice(&(h as i32).to_le_bytes());
            f.extend_from_slice(&1u16.to_le_bytes());
            f.extend_from_slice(&24u16.to_le_bytes());
            f.extend_from_slice(&[0; 24]);
            f.extend_from_slice(&data);
            std::fs::write(format!("{dir}/splash-{name}.bmp"), f).unwrap();
        }
    }
}

#[cfg(test)]
mod theme_probe {
    #[test]
    #[ignore]
    fn print_system_dark() {
        println!("system_dark = {}", super::system_dark());
    }
}
