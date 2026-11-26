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
    /// Open tabs at startup.
    pub tabs: usize,
    pub show_clock: bool,
    /// The bookmarks bar is on screen at startup.
    pub bm_bar: bool,
    /// Sizes the UI measured last time (see `Shape`).
    pub shape: Shape,
}

/// Sizes of things that depend on your data, measured by the UI while it runs and
/// saved to `skeleton.json`, so the skeleton matches them on the next launch.
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct Shape {
    /// Width of each start-page tile's label (the + Add tile included), CSS pixels.
    pub tiles: Vec<f32>,
    /// Width of each item shown on the bookmarks bar (folders included), CSS pixels.
    pub bm: Vec<f32>,
    pub time_w: f32,
    pub date_w: f32,
}

impl Default for Shape {
    fn default() -> Self {
        Self { tiles: vec![22.0], bm: Vec::new(), time_w: 259.0, date_w: 148.0 }
    }
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
    let r = look.radius;
    // Laid out in CSS pixels, mirroring ui/style.css; `rect` scales to the screen.
    let (cw, ch) = (w as f32 / s, h as f32 / s);
    let mut rect = |x: f32, y: f32, ww: f32, hh: f32, rad: f32, rgb: (u8, u8, u8), a: f32, sk: bool| {
        c.rrect(x * s, y * s, ww * s, hh * s, rad * s, rgb, a, sk);
    };

    // Window tint (the native blur shows through it, exactly as in the real UI).
    rect(0.0, 0.0, cw, ch, 0.0, base, 1.0, false);

    // Window buttons: 46x34 each in the top-right corner, with 10px glyphs.
    for i in 0..3 {
        let x = cw - 138.0 + i as f32 * 46.0;
        rect(x + 18.0, 12.0, 10.0, 10.0, 2.0, bone, bone_a * 1.4, true);
    }
    // Groups of four 34px buttons, 1px apart: back/forward/reload/home and
    // shield/panel/settings/account.
    let group = 4.0 * 34.0 + 3.0;
    let four = |rect: &mut dyn FnMut(f32, f32, f32, f32, f32, (u8, u8, u8), f32, bool), x0: f32, y: f32| {
        for i in 0..4 {
            rect(x0 + i as f32 * 35.0 + 8.0, y + 8.0, 18.0, 18.0, 6.0, bone, bone_a, true);
        }
    };

    let top;
    if look.compact {
        // One row: nav, address bar, tabs, actions.
        let row_y = 10.0;
        four(&mut rect, 8.0, row_y);
        let ox = 8.0 + group + 6.0;
        let ow = 440.0_f32.min(cw * 0.35);
        rect(ox, row_y, ow, 38.0, r * 0.95, glass, glass_a, true);
        rect(ox + 38.0, row_y + 14.0, 142.0_f32.min(ow * 0.5), 10.0, 5.0, bone, bone_a, true);
        rect(ox + ow + 6.0, row_y + 3.0, 210.0, 32.0, r * 0.78, glass, glass_a, true);
        four(&mut rect, cw - 138.0 - 8.0 - group, row_y);
        top = 54.0;
    } else {
        // Tabs: up to 210px each, 3px apart, shrinking to fit before the window buttons.
        let count = look.tabs.max(1);
        let room = (cw - 10.0 - 138.0 - 40.0).max(42.0);
        let tw = ((room - (count as f32 - 1.0) * 3.0) / count as f32).clamp(42.0, 210.0);
        let mut x = 10.0;
        for i in 0..count {
            if x + tw > cw - 138.0 {
                break;
            }
            rect(x, 7.0, tw, 32.0, r * 0.78, glass, if i == 0 { glass_a } else { glass_a * 0.45 }, true);
            rect(x + 12.0, 16.0, 14.0, 14.0, 4.0, bone, bone_a, true);
            if tw >= 74.0 {
                rect(x + 34.0, 18.0, (tw - 70.0).min(64.0), 10.0, 5.0, bone, bone_a, true);
            }
            x += tw + 3.0;
        }
        rect(x + 7.0, 11.0, 24.0, 24.0, 7.0, bone, bone_a * 0.7, true);

        let row_y = 44.0;
        four(&mut rect, 8.0, row_y);
        four(&mut rect, cw - 8.0 - group, row_y);
        // The address bar sits in the middle column, at most 920px wide and centered.
        let col_x = 8.0 + group + 6.0;
        let avail = (cw - 2.0 * col_x).max(0.0);
        let ow = avail.min(920.0);
        let ox = col_x + (avail - ow) / 2.0;
        rect(ox, 42.0, ow, 38.0, r * 0.95, glass, glass_a, true);
        rect(ox + 16.0, 54.0, 14.0, 14.0, 7.0, bone, bone_a, true);
        rect(ox + 39.0, 56.0, 142.0_f32.min(ow * 0.4), 10.0, 5.0, bone, bone_a, true);

        top = if look.bm_bar {
            // Bookmarks bar: 30px tall at y 84; items are 26px tall, 2px apart.
            if look.shape.bm.is_empty() {
                rect(16.0, 94.0, 370.0_f32.min(cw - 40.0), 10.0, 5.0, bone, bone_a * 0.8, true);
            }
            let mut x = 10.0;
            for &bw in &look.shape.bm {
                if x + bw > cw - 40.0 {
                    break;
                }
                rect(x + 7.0, 91.0, 16.0, 16.0, 4.0, bone, bone_a, true);
                if bw > 40.0 {
                    rect(x + 30.0, 94.0, bw - 38.0, 10.0, 5.0, bone, bone_a, true);
                }
                x += bw + 2.0;
            }
            120.0
        } else {
            86.0
        };
    }

    // Page area.
    let (vx, vy, vw, vh) = (8.0, top, cw - 16.0, ch - top - 8.0);
    let pr = r * 0.7;
    rect(vx - 1.0, vy - 1.0, vw + 2.0, vh + 2.0, pr + 1.0, glass, if look.dark { 0.06 } else { 0.5 }, false);
    rect(vx, vy, vw, vh, pr, page, 1.0, false);

    let mx = vx + vw / 2.0;
    if look.blank {
        // Start page: a centered column (30px apart, 6vh padding at the bottom) of the
        // clock, the search pill and the tile grid.
        let tiles = if look.shape.tiles.is_empty() { vec![22.0] } else { look.shape.tiles.clone() };
        let grid_w = 660.0_f32.min(vw * 0.92);
        let cols = (((grid_w + 12.0) / 104.0).floor() as usize).max(1);
        let rows = tiles.len().div_ceil(cols);
        let tiles_h = rows as f32 * 95.0 + rows.saturating_sub(1) as f32 * 10.0;
        let clock_h = if look.show_clock { 96.0 + 30.0 } else { 0.0 };
        let total = clock_h + 50.0 + 30.0 + tiles_h;
        let mut y = vy + ((vh - 0.06 * ch) - total) / 2.0;
        if look.show_clock {
            let tw = look.shape.time_w;
            rect(mx - tw / 2.0, y + 9.0, tw, 52.0, 14.0, bone, bone_a, true);
            let dw = look.shape.date_w;
            rect(mx - dw / 2.0, y + 80.0, dw, 12.0, 6.0, bone, bone_a, true);
            y += clock_h;
        }
        let pw = 580.0_f32.min(vw * 0.84);
        let px0 = mx - pw / 2.0;
        rect(px0, y, pw, 50.0, 25.0, glass, glass_a, true);
        rect(px0 + 22.0, y + 17.0, 16.0, 16.0, 8.0, bone, bone_a, true);
        rect(px0 + 52.0, y + 20.0, 156.0_f32.min(pw * 0.4), 11.0, 5.5, bone, bone_a, true);
        rect(px0 + pw - 51.0, y + 14.0, 41.0, 21.0, 6.0, bone, bone_a, true);
        y += 50.0 + 30.0;
        // Tile grid: 92px columns, 12px apart, centered; rows 95px tall, 10px apart.
        let row_w = cols as f32 * 92.0 + (cols as f32 - 1.0) * 12.0;
        let gx = mx - row_w / 2.0;
        for (i, &lw) in tiles.iter().enumerate() {
            let (tx, ty) = (gx + (i % cols) as f32 * 104.0, y + (i / cols) as f32 * 105.0);
            let add = i == tiles.len() - 1;
            rect(tx + 19.0, ty + 8.0, 54.0, 54.0, r * 1.05, glass, if add { glass_a * 0.4 } else { glass_a }, true);
            if !add {
                rect(tx + 34.0, ty + 23.0, 24.0, 24.0, 6.0, bone, bone_a, true);
            }
            let lw = lw.clamp(16.0, 88.0);
            rect(tx + 46.0 - lw / 2.0, ty + 72.0, lw, 10.0, 5.0, bone, bone_a, true);
        }
    } else {
        // A generic page: header bar, title, paragraphs.
        let px = vx + vw * 0.08;
        let pw = vw * 0.84;
        rect(vx + 16.0, vy + 16.0, vw - 32.0, 44.0, 12.0, bone, bone_a * 0.8, true);
        rect(px, vy + 100.0, pw * 0.55, 26.0, 10.0, bone, bone_a, true);
        let mut y = vy + 150.0;
        for (i, frac) in [0.95, 0.88, 0.92, 0.6, 0.0, 0.9, 0.82, 0.94, 0.45].iter().enumerate() {
            if *frac > 0.0 && y + 14.0 < vy + vh - 16.0 {
                rect(px, y, pw * frac, 12.0, 6.0, bone, bone_a * if i % 2 == 0 { 1.0 } else { 0.8 }, true);
            }
            y += 26.0;
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
            let (w, h) = (1480, 929);
            let look = Look { dark, tint: 0.6, radius: 11.0, compact: false, blank, scale: 1.0, tabs: 1, show_clock: true, bm_bar: true, shape: Shape { tiles: vec![34.0, 31.0, 46.0, 75.0, 51.0, 75.0, 56.0, 71.0, 22.0], bm: vec![168.0], ..Shape::default() } };
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
