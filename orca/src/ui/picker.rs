//! The item picker and the colour picker: modal panels over the click GUI (CSS pixels).
use super::*;
use crate::items::{ITEMS, TAGS};

#[derive(Clone)]
pub enum Pick {
    /// Tag 0 is All, 1 is Selected, and 2.. are `TAGS`.
    Items { mi: usize, si: usize, query: String, tag: usize, scroll: f32 },
    /// `hv` is hue in degrees, saturation and value, kept apart from the colour so greys keep their hue.
    Color { mi: usize, si: usize, hv: [f32; 3] },
}

const PX: f32 = 50.0;
const PY: f32 = 14.0;
const PW: f32 = 700.0;
const PH: f32 = 512.0;
const IN_X: f32 = PX + 20.0;
const IN_W: f32 = PW - 40.0;
const ROW: f32 = 40.0;
const SWATCH_COLS: usize = 12;
const SWATCH_ROWS: usize = 6;

fn chosen(list: &str) -> Vec<&str> {
    list.split(',').map(str::trim).filter(|t| !t.is_empty()).collect()
}

fn shown(query: &str, tag: usize, sel: &[&str]) -> Vec<usize> {
    let q = query.trim().to_lowercase().replace(' ', "_");
    ITEMS
        .iter()
        .enumerate()
        .filter(|(_, (id, t, _))| {
            (q.is_empty() || id.contains(q.as_str())) && match tag {
                0 => true,
                1 => sel.contains(id),
                n => *t as usize == n - 2,
            }
        })
        .map(|(i, _)| i)
        .collect()
}

fn hsv(h: f32, s: f32, v: f32) -> u32 {
    let c = v * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    let f = |a: f32| ((a + m) * 255.0).round() as u32;
    0xFF00_0000 | f(r) << 16 | f(g) << 8 | f(b)
}

fn swatch(i: usize) -> u32 {
    let (r, c) = (i / SWATCH_COLS, i % SWATCH_COLS);
    const SV: [(f32, f32); 5] = [(0.25, 1.0), (0.55, 1.0), (1.0, 1.0), (1.0, 0.75), (1.0, 0.5)];
    if r < SV.len() {
        hsv(c as f32 * 30.0, SV[r].0, SV[r].1)
    } else {
        let g = (c as f32 / (SWATCH_COLS - 1) as f32 * 255.0).round() as u32;
        0xFF00_0000 | g << 16 | g << 8 | g
    }
}

pub fn to_hsv(c: u32) -> [f32; 3] {
    let [r, g, b] = [16, 8, 0].map(|s| (c >> s & 0xFF) as f32 / 255.0);
    let (hi, lo) = (r.max(g).max(b), r.min(g).min(b));
    let d = hi - lo;
    let h = if d == 0.0 {
        0.0
    } else if hi == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if hi == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    [h, if hi == 0.0 { 0.0 } else { d / hi }, hi]
}

impl State {
    /// Moves the picker's square or hue bar to the mouse while one is held.
    pub fn color_drag(&mut self, at: [f32; 2]) {
        let (Some(kind), Some(Pick::Color { mi, si, hv })) = (self.cdrag, self.pick.clone()) else { return };
        let Some(r) = self.rects.iter().find(|(h, _)| if kind == 1 { matches!(h, Hit::PkSv) } else { matches!(h, Hit::PkHue) }).map(|(_, r)| *r) else { return };
        let (tx, ty) = (((at[0] - r[0]) / r[2]).clamp(0.0, 1.0), ((at[1] - r[1]) / r[3]).clamp(0.0, 1.0));
        let n = if kind == 1 { [hv[0], tx, 1.0 - ty] } else { [tx * 360.0, hv[1], hv[2]] };
        self.vals[mi][si] = Val::Argb(hsv(n[0], n[1], n[2]));
        self.pick = Some(Pick::Color { mi, si, hv: n });
    }

    pub fn pick_press(&mut self, hit: Option<Hit>, at: [f32; 2]) {
        let Some(pick) = self.pick.clone() else { return };
        match (pick, hit) {
            (_, None | Some(Hit::PkClose | Hit::PkScrim)) => self.pick = None,
            (Pick::Items { mi, si, query, tag, .. }, Some(h)) => match h {
                Hit::PkTag(t) => {
                    if let Some(Pick::Items { tag, scroll, .. }) = &mut self.pick {
                        *tag = t;
                        *scroll = 0.0;
                    }
                }
                Hit::PkItem(k) => {
                    let id = ITEMS[k].0;
                    if let Val::Text(t) = &mut self.vals[mi][si] {
                        let mut v = chosen(t);
                        match v.iter().position(|x| *x == id) {
                            Some(p) => {
                                v.remove(p);
                            }
                            None => v.push(id),
                        }
                        let s = v.join(", ");
                        *t = s;
                    }
                }
                Hit::PkClear => self.vals[mi][si] = Val::Text(String::new()),
                Hit::PkAll => {
                    if let Val::Text(t) = &mut self.vals[mi][si] {
                        let mut v = chosen(t);
                        for k in shown(&query, tag, &v) {
                            if !v.contains(&ITEMS[k].0) {
                                v.push(ITEMS[k].0);
                            }
                        }
                        let s = v.join(", ");
                        *t = s;
                    }
                }
                _ => {}
            },
            (Pick::Color { mi, si, .. }, Some(Hit::PkSwatch(i))) => {
                let c = swatch(i);
                self.vals[mi][si] = Val::Argb(c);
                self.pick = Some(Pick::Color { mi, si, hv: to_hsv(c) });
            }
            (Pick::Color { .. }, Some(h @ (Hit::PkSv | Hit::PkHue))) => {
                self.cdrag = Some(if matches!(h, Hit::PkSv) { 1 } else { 2 });
                self.color_drag(at);
            }
            _ => {}
        }
    }
}

pub(super) fn draw(p: &mut Pen<'_>, win: f32) {
    if p.s.pick.is_none() {
        return;
    }
    let a = p.s.anim(anim::POPUP | 0xFFFF, 1.0, 0.0, 20.0);
    gfx::set_alpha(win * a);
    p.o[1] += ((1.0 - a) * 8.0 * k()).round();
    match p.s.pick.clone() {
        Some(Pick::Items { mi, si, query, tag, scroll }) => {
            let scroll = items(p, mi, si, &query, tag, scroll);
            if let Some(Pick::Items { scroll: s, .. }) = &mut p.s.pick {
                *s = scroll;
            }
        }
        Some(Pick::Color { mi, si, hv }) => color(p, mi, si, hv),
        None => {}
    }
    p.o[1] -= ((1.0 - a) * 8.0 * k()).round();
}

fn scrim(p: &mut Pen<'_>) {
    p.round([0.0, 0.0, WIN_W, WIN_H], 14.0, 0x9906_0608);
    p.hit(Hit::PkScrim, [0.0, 0.0, WIN_W, WIN_H]);
}

fn button(p: &mut Pen<'_>, label: &str, right: f32, y: f32, hit: Hit, primary: bool) -> f32 {
    let w = p.width(label, REG(12)) + 24.0;
    let r = [right - w, y, w, 26.0];
    let hv = p.hov(r);
    p.round(r, 7.0, if primary { gfx::mix(ACCENT, 0xFF93_C0F7, hv) } else { gfx::mix(FIELD, 0xFF27_272E, hv) });
    p.text_center(label, r[0] + w / 2.0, y + 13.0, if primary { INK } else { SOFT }, if primary { BOLD(12) } else { REG(12) });
    p.hit(hit, r);
    r[0] - 8.0
}

fn items(p: &mut Pen<'_>, mi: usize, si: usize, query: &str, tag: usize, scroll: f32) -> f32 {
    let list = match &p.s.vals[mi][si] {
        Val::Text(t) => t.clone(),
        _ => return scroll,
    };
    let sel = chosen(&list);
    scrim(p);
    p.round([PX - 1.0, PY - 1.0, PW + 2.0, PH + 2.0], 17.0, POP_RING);
    p.round([PX, PY, PW, PH], 16.0, POP);
    p.hit(Hit::PkNone, [PX, PY, PW, PH]);

    let title = format!("{} · {}", p.s.mods[mi].name, p.s.mods[mi].settings[si].label);
    p.text(&title, IN_X, 44.0, TEXT, BOLD(16));
    p.text(&format!("{} selected", sel.len()), IN_X, 63.0, FAINT, REG(12));
    let cx = [PX + PW - 20.0 - 32.0, 34.0, 32.0, 32.0];
    p.round(cx, 9.0, FIELD);
    let hv = p.hov(cx);
    p.text_center("×", cx[0] + 16.0, cx[1] + 16.0, gfx::mix(MUTED, TEXT, hv), REG(16));
    p.hit(Hit::PkClose, cx);

    // Search field.
    p.round([IN_X, 84.0, IN_W, 42.0], 10.0, FIELD);
    p.ring([IN_X + 14.0, 97.0, 13.0, 13.0], 6.5, MUTED);
    let a = p.at([IN_X + 14.0 + 11.5, 97.0 + 11.5, 0.0, 0.0]);
    gfx::line(p.d, [a[0], a[1]], [a[0] + 3.5 * k(), a[1] + 3.5 * k()], 1.2 * k().max(0.8), MUTED);
    let blink = (p.s.started.elapsed().as_millis() / 500) % 2 == 0;
    if query.is_empty() {
        p.text("Search items", IN_X + 40.0, 105.0, FAINT, REG(14));
    } else {
        let shown_q = if blink { format!("{query}|") } else { query.to_owned() };
        p.text(&shown_q, IN_X + 40.0, 105.0, TEXT, REG(14));
    }

    // Tag pills, wrapping.
    let names: Vec<&str> = ["All", "Selected"].into_iter().chain(TAGS.iter().copied()).collect();
    let (mut x, mut y) = (0.0f32, 140.0f32);
    for (i, n) in names.iter().enumerate() {
        let count = shown("", i, &sel).len().to_string();
        let (nw, cw) = (p.width(n, REG(12)), p.width(&count, REG(10)));
        let w = nw + cw + 13.0 * 2.0 + 7.0;
        if x > 0.0 && x + w > IN_W {
            x = 0.0;
            y += 36.0;
        }
        let r = [IN_X + x, y, w, 30.0];
        let on = i == tag;
        p.round(r, 15.0, if on { ACCENT } else { FIELD });
        let hv = p.hov(r);
        p.text(n, r[0] + 13.0, y + 15.0, if on { INK } else { gfx::mix(MUTED, TEXT, hv) }, if on { BOLD(12) } else { REG(12) });
        p.text(&count, r[0] + 13.0 + nw + 7.0, y + 15.0, if on { 0xA614_1413 } else { FAINT }, REG(10));
        p.hit(Hit::PkTag(i), r);
        x += w + 6.0;
    }

    // The list: three columns, scrolled, cut to its box.
    let top = y + 30.0 + 14.0;
    let bottom = PY + PH - 20.0 - 26.0 - 14.0;
    let lh = bottom - top;
    let idx = shown(query, tag, &sel);
    let rows = idx.len().div_ceil(3);
    let content = (rows as f32 * ROW - 4.0).max(0.0);
    let max = (content - lh).max(0.0);
    let scroll = scroll.clamp(0.0, max);
    if idx.is_empty() {
        p.text_center("No items found", PX + PW / 2.0, top + 48.0, FAINT, REG(14));
    } else if !p.dry {
        let r = p.at([IN_X, top, IN_W, lh]);
        let clipped = p.d.push_clip(r).is_ok();
        p.clip = Some([top, bottom]);
        let cw = (IN_W - 8.0) / 3.0;
        let first = (scroll / ROW) as usize;
        let last = (((scroll + lh) / ROW) as usize + 1).min(rows);
        for row in first..last {
            for col in 0..3 {
                let Some(&k) = idx.get(row * 3 + col) else { break };
                let (id, _, swatch_col) = ITEMS[k];
                let r = [IN_X + col as f32 * (cw + 4.0), top + row as f32 * ROW - scroll, cw, 36.0];
                let on = sel.contains(&id);
                let hv = p.hov(r);
                if on {
                    p.round(r, 9.0, SEL);
                    p.ring(r, 9.0, 0x80_80B4F5);
                } else if hv > 0.01 {
                    p.round(r, 9.0, gfx::alpha(0xFF1F_1F25, hv));
                }
                if !p.item(id, r[0] + 10.0, r[1] + 10.0, 16.0) {
                    p.round([r[0] + 12.0, r[1] + 12.0, 12.0, 12.0], 3.0, swatch_col);
                }
                let face = if on { BOLD(12) } else { REG(12) };
                let name = fit(p, &id.replace('_', " "), cw - 34.0 - 12.0, face);
                p.text(&name, r[0] + 34.0, r[1] + 18.0, if on { TEXT } else { gfx::mix(MUTED, LABEL, hv) }, face);
                p.hit(Hit::PkItem(k), r);
            }
        }
        p.clip = None;
        if clipped {
            let _ = p.d.pop_clip();
        }
        if max > 0.0 {
            let th = (lh * lh / content).max(18.0);
            p.round([IN_X + IN_W + 3.0, top + (lh - th) * scroll / max, 3.0, th], 2.0, TRACK);
        }
    }

    // Footer.
    let fy = PY + PH - 20.0 - 26.0;
    p.text(&format!("{} items", idx.len()), IN_X, fy + 13.0, FAINT, REG(12));
    let r = button(p, "Done", IN_X + IN_W, fy, Hit::PkClose, true);
    let r = button(p, "Select shown", r, fy, Hit::PkAll, false);
    button(p, "Clear", r, fy, Hit::PkClear, false);
    scroll
}

fn color(p: &mut Pen<'_>, mi: usize, si: usize, hv: [f32; 3]) {
    let Val::Argb(cur) = p.s.vals[mi][si] else { return };
    const N: usize = 71;
    const CW: f32 = 4.0;
    let (iw, sv_h, hue_h) = (N as f32 * CW, 120.0, 14.0);
    let (w, h) = (16.0 * 2.0 + iw, 292.0);
    let (x, y) = ((WIN_W - w) / 2.0, (WIN_H - h) / 2.0);
    scrim(p);
    p.round([x - 1.0, y - 1.0, w + 2.0, h + 2.0], 15.0, POP_RING);
    p.round([x, y, w, h], 14.0, POP);
    p.hit(Hit::PkNone, [x, y, w, h]);
    let title = format!("{} \u{b7} {}", p.s.mods[mi].name, p.s.mods[mi].settings[si].label);
    p.text(&title, x + 16.0, y + 26.0, TEXT, BOLD(14));
    // Saturation runs left to right and value top to bottom, one gradient column per step.
    let sv = [x + 16.0, y + 46.0, iw, sv_h];
    let hue = [x + 16.0, sv[1] + sv_h + 10.0, iw, hue_h];
    if !p.dry {
        for i in 0..N {
            let t = i as f32 / (N - 1) as f32;
            let mut r = p.at([sv[0] + i as f32 * CW, sv[1], CW, sv_h]);
            r[2] += 0.6;
            gfx::gradient(p.d, r, hsv(hv[0], t, 1.0), 0xFF00_0000);
            let mut r = p.at([hue[0] + i as f32 * CW, hue[1], CW, hue_h]);
            r[2] += 0.6;
            gfx::fill(p.d, r, hsv(t * 360.0, 1.0, 1.0));
        }
    }
    p.hit(Hit::PkSv, sv);
    p.hit(Hit::PkHue, hue);
    let (cx, cy) = (sv[0] + hv[1] * iw, sv[1] + (1.0 - hv[2]) * sv_h);
    p.round([cx - 4.0, cy - 4.0, 8.0, 8.0], 4.0, cur | 0xFF00_0000);
    p.ring([cx - 5.0, cy - 5.0, 10.0, 10.0], 5.0, TEXT);
    p.round([hue[0] + hv[0] / 360.0 * iw - 2.0, hue[1] - 2.0, 4.0, hue_h + 4.0], 2.0, TEXT);
    // Presets: the vivid row and the greys of the old palette.
    let py = hue[1] + hue_h + 14.0;
    for (ri, row) in [2usize, 5].into_iter().enumerate() {
        for c in 0..SWATCH_COLS {
            let i = row * SWATCH_COLS + c;
            let sr = [x + 16.0 + c as f32 * 24.0, py + ri as f32 * 24.0, 20.0, 20.0];
            let col = swatch(i);
            p.round(sr, 5.0, col);
            if col == cur | 0xFF00_0000 {
                p.ring([sr[0] - 2.0, sr[1] - 2.0, 24.0, 24.0], 6.0, TEXT);
            }
            p.hit(Hit::PkSwatch(i), sr);
        }
    }
    let cy = py + 2.0 * 24.0 + 8.0;
    p.round([x + 16.0, cy, 20.0, 20.0], 5.0, cur | 0xFF00_0000);
    p.text(&format!("#{:06X}", cur & 0xFF_FFFF), x + 44.0, cy + 10.0, SOFT, REG(12));
}
