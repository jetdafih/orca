//! The in-game overlays, laid out from orca.html's `.al`, `.ks` and `.toast` (CSS pixels) at
//! the largest window scale.
use super::*;
use crate::anim;
use crate::gfx::{self, Face};
use std::time::Instant;

const K: f32 = 0.6;
const FAINT: u32 = 0xFF85_858F;
const CARD_BG: u32 = 0xFF15_1518;
const RING: u32 = 0xFF26_262C;

const fn reg(p: u8) -> Face {
    Face(p, false)
}

pub struct Toast {
    pub name: String,
    pub off: bool,
    pub at: Instant,
}

const LIFE: f32 = 2.2;
const FADE_IN: f32 = 0.18;
const FADE_OUT: f32 = 0.25;

fn txt(d: &mut dyn DrawContext, t: &str, x: f32, cy: f32, c: u32, f: Face) {
    gfx::text(d, t, [x, gfx::text_y(cy, f)], c, f);
}

pub fn draw(s: &mut State, d: &mut dyn DrawContext, host: &dyn ClientHost, size: [f32; 2]) {
    gfx::set_tier(0);
    s.tick();
    let n = s.mods.len();
    let lit: Vec<f32> = (0..n)
        .map(|i| {
            let on = (s.on[i] && s.mods[i].name != "Watermark") as u8 as f32;
            s.anim(anim::HUD | i as u64, on, on, 18.0)
        })
        .collect();
    let bound: Vec<f32> = (0..n)
        .map(|i| {
            let on = s.on[i] as u8 as f32;
            s.anim(anim::HUD | 1 << 20 | i as u64, on, on, 18.0)
        })
        .collect();
    let w = |t: &str, f: Face| gfx::text_width(host, t, f);

    if s.arraylist {
        let mut names: Vec<(&str, f32)> = (0..n).filter(|&i| lit[i] > 0.01).map(|i| (s.mods[i].name, lit[i])).collect();
        names.sort_by_key(|(n, _)| std::cmp::Reverse(n.len()));
        let mut y = 10.0 * K;
        for (name, v) in names {
            gfx::set_alpha(v);
            let rw = w(name, reg(14)) + 18.0 * K;
            let h = 22.0 * K;
            let x = size[0] - 12.0 * K - rw + (1.0 - v) * 24.0 * K;
            d.fill([x, y, rw, h], 0x730A_0A0E);
            d.fill([x + rw - 2.0 * K, y, 2.0 * K, h], ACCENT);
            txt(d, name, x + 8.0 * K, y + h / 2.0 + 0.6 * K, 0x8F00_0000, reg(14));
            txt(d, name, x + 8.0 * K, y + h / 2.0, 0xFFFF_FFFF, reg(14));
            y += h * v;
        }
        gfx::set_alpha(1.0);
    }

    if s.keybinds {
        let binds: Vec<usize> = (0..s.mods.len()).filter(|&i| s.key[i].is_some()).collect();
        let kbd = |t: &str| w(t, reg(11)) / K + 12.0;
        let rows: Vec<(&str, String, f32)> = binds.iter().map(|&i| (s.mods[i].name, key_label(s.key[i].as_deref().unwrap()), bound[i])).collect();
        let mut content = kbd("Delete") + 14.0 + w("GUI", reg(11)) / K;
        for (n, k, _) in &rows {
            content = content.max(w(n, reg(12)) / K + 14.0 + kbd(k));
        }
        let pw = (content + 20.0).clamp(150.0, 210.0);
        let body_rows = rows.len().max(1) as f32;
        let ph = 8.0 + body_rows * 18.0 + (body_rows - 1.0) + 1.0 + 4.0 + 18.0 + 8.0;
        let (px, py) = (12.0 * K, 10.0 * K);
        gfx::round(d, [px, py, pw * K, ph * K], 8.0 * K, 0x8C0A_0A0E);

        let right = px + (pw - 10.0) * K;
        let mut y = py + 8.0 * K;
        let kbd_row = |d: &mut dyn DrawContext, y: f32, label: &str| {
            let bw = (w(label, reg(11)) / K + 12.0) * K;
            gfx::round(d, [right - bw, y + 1.0 * K, bw, 16.0 * K], 4.0 * K, 0x12FF_FFFF);
            txt(d, label, right - bw + 6.0 * K, y + 9.0 * K, ACCENT, reg(11));
        };
        if rows.is_empty() {
            txt(d, "No keybinds set", px + 10.0 * K, y + 9.0 * K, FAINT, reg(11));
            y += 19.0 * K;
        }
        for (n, k, on) in &rows {
            txt(d, n, px + 10.0 * K, y + 9.0 * K, gfx::mix(FAINT, 0xFFFF_FFFF, *on), reg(12));
            kbd_row(d, y, k);
            y += 19.0 * K;
        }
        y += 4.0 * K - 1.0 * K;
        txt(d, "GUI", px + 10.0 * K, y + 9.0 * K, FAINT, reg(11));
        kbd_row(d, y, "Delete");
    }

    s.toasts.retain(|t| t.at.elapsed().as_secs_f32() < LIFE + FADE_OUT);
    let mut bottom = size[1] - 16.0 * K;
    for t in s.toasts.iter().rev() {
        let age = t.at.elapsed().as_secs_f32();
        let a = (age / FADE_IN).min(1.0) * (1.0 - ((age - LIFE) / FADE_OUT).clamp(0.0, 1.0));
        let h = 33.0 * K;
        let tw = w(&t.name, reg(13));
        let bw = tw + 30.0 * K;
        let x = size[0] - 16.0 * K - bw;
        let y = bottom - h + (1.0 - (age / FADE_IN).min(1.0)) * 6.0 * K;
        gfx::round(d, [x - 0.6, y - 0.6, bw + 1.2, h + 1.2], 10.0 * K + 0.6, gfx::alpha(RING, a));
        gfx::round(d, [x, y, bw, h], 10.0 * K, gfx::alpha(CARD_BG, a));
        let (col, tx) = if t.off { (gfx::alpha(FAINT, a), x + 15.0 * K) } else { (gfx::alpha(TEXT, a), x + 15.0 * K) };
        txt(d, &t.name, tx, y + h / 2.0, col, reg(13));
        if t.off {
            d.fill([tx, (y + h / 2.0).round(), tw, 1.0], col);
        }
        bottom -= h + 6.0 * K;
    }
}

pub fn push_toast(s: &mut State, name: &str, off: bool) {
    s.toasts.push(Toast { name: name.to_owned(), off, at: Instant::now() });
    while s.toasts.len() > 4 {
        s.toasts.remove(0);
    }
}
