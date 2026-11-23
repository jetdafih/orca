//! The click GUI, laid out from orca.html's CSS (800x540 window, in CSS pixels). The scale is the
//! largest of `gfx::TIERS` that fits the screen, so every size below reads as the stylesheet's number.
use super::*;
use crate::gfx::{self, Face};
pub mod picker;

const POP: u32 = 0xFF18_181C;
const POP_RING: u32 = 0xFF26_262C;
const LINE: u32 = 0xFF22_2227;
const FAINT: u32 = 0xFF85_858F;
const INK: u32 = 0xFF14_1413;
const SOFT: u32 = 0xFFE4_E4E8;
const TAB_ON: u32 = 0xFF1A_1A1E;
const TG_RING: u32 = 0xFF34_343C;
const TG_KNOB: u32 = 0xFF7E_7E88;
const VALUE: u32 = 0xFFA3_A3AD;
const DASH: u32 = 0xFF3A_3A43;
const OPT_HOVER: u32 = 0xFF22_2228;

const REG: fn(u8) -> Face = |p| Face(p, false);
const BOLD: fn(u8) -> Face = |p| Face(p, true);

pub const WIN_W: f32 = 800.0;
pub const WIN_H: f32 = 540.0;

struct Pen<'a> {
    s: &'a mut State,
    d: &'a mut dyn DrawContext,
    host: &'a dyn ClientHost,
    /// Window origin in GUI pixels.
    o: [f32; 2],
    /// Measuring only: nothing is drawn and no hit boxes are recorded.
    dry: bool,
    popups: Vec<Popup>,
    /// Hit boxes are cut to this range of window y (CSS pixels) while a region scrolls.
    clip: Option<[f32; 2]>,
}

enum Popup {
    Options { mi: usize, si: usize, rect: [f32; 4] },
}

fn k() -> f32 {
    gfx::scale()
}

fn sc(r: [f32; 4]) -> [f32; 4] {
    [r[0] * k(), r[1] * k(), r[2] * k(), r[3] * k()]
}

impl Pen<'_> {
    fn at(&self, r: [f32; 4]) -> [f32; 4] {
        let r = sc(r);
        [self.o[0] + r[0], self.o[1] + r[1], r[2], r[3]]
    }

    fn width(&self, t: &str, f: Face) -> f32 {
        gfx::text_width(self.host, t, f) / k()
    }

    /// Text whose vertical centre is `cy` (CSS pixels from the window top), left edge `x`.
    fn text(&mut self, t: &str, x: f32, cy: f32, color: u32, f: Face) {
        if self.dry || t.is_empty() {
            return;
        }
        gfx::text(self.d, t, [self.o[0] + x * k(), gfx::text_y(self.o[1] + cy * k(), f)], color, f);
    }

    /// The item's own icon, `size` CSS pixels square; false when it can't be drawn (unknown item, mid-fade).
    fn item(&mut self, id: &str, x: f32, y: f32, size: f32) -> bool {
        if self.dry || !gfx::opaque() {
            return false;
        }
        let key = match self.s.icons.get(id) {
            Some(k) => *k,
            None => {
                let k = mod_api::Identifier::parse(&format!("minecraft:{id}")).ok().and_then(|i| self.host.item_key(&i));
                self.s.icons.insert(id.to_owned(), k);
                k
            }
        };
        // 16 texels, drawn at a whole number of screen pixels each and on a pixel boundary, so none come out wider.
        let (r, s) = (self.at([x, y, size, size]), gfx::gui_scale());
        let g = 16.0 * (r[2] * s / 16.0).round().max(1.0) / s;
        let at = [r[0] + (r[2] - g) / 2.0, r[1] + (r[2] - g) / 2.0].map(|v| (v * s).round() / s);
        key.is_some_and(|k| self.d.item(k, at, g))
    }

    /// Where the middle of a line drawn at `cy` really lands, in CSS pixels, after the game snaps it to a GUI pixel.
    fn mid(&self, cy: f32, f: Face) -> f32 {
        (gfx::text_mid(gfx::text_y(self.o[1] + cy * k(), f), f) - self.o[1]) / k()
    }

    fn text_right(&mut self, t: &str, right: f32, cy: f32, color: u32, f: Face) {
        let w = self.width(t, f);
        self.text(t, right - w, cy, color, f);
    }

    fn text_center(&mut self, t: &str, cx: f32, cy: f32, color: u32, f: Face) {
        let w = self.width(t, f);
        self.text(t, cx - w / 2.0, cy, color, f);
    }

    fn round(&mut self, r: [f32; 4], rad: f32, c: u32) {
        if !self.dry {
            let r = self.at(r);
            gfx::round(self.d, r, rad * k(), c);
        }
    }

    fn ring(&mut self, r: [f32; 4], rad: f32, c: u32) {
        if !self.dry {
            let r = self.at(r);
            gfx::ring(self.d, r, rad * k(), k().max(0.6), c);
        }
    }

    fn dashed(&mut self, r: [f32; 4], c: u32) {
        if self.dry {
            return;
        }
        let [x, y, w, h] = r;
        let (dash, gap, th): (f32, f32, f32) = (4.0, 3.0, 1.0);
        let run = |d: &mut Self, a: f32, b: f32, fixed: f32, horiz: bool| {
            let mut t = a;
            while t < b {
                let l = dash.min(b - t);
                let seg = if horiz { [t, fixed, l, th] } else { [fixed, t, th, l] };
                let seg = d.at(seg);
                gfx::fill(d.d, seg, c);
                t += dash + gap;
            }
        };
        run(self, x + 4.0, x + w - 4.0, y, true);
        run(self, x + 4.0, x + w - 4.0, y + h - th, true);
        run(self, y + 4.0, y + h - 4.0, x, false);
        run(self, y + 4.0, y + h - 4.0, x + w - th, false);
    }

    fn hit(&mut self, h: Hit, mut r: [f32; 4]) {
        if self.dry {
            return;
        }
        if let Some([a, b]) = self.clip {
            let (y0, y1) = (r[1].max(a), (r[1] + r[3]).min(b));
            if y1 <= y0 {
                return;
            }
            r[1] = y0;
            r[3] = y1 - y0;
        }
        let r = self.at(r);
        self.s.rects.push((h, r));
    }

    /// How far the mouse is over `r` (CSS pixels), eased 0 to 1.
    fn hov(&mut self, r: [f32; 4]) -> f32 {
        if self.dry {
            return 0.0;
        }
        let (sr, m) = (self.at(r), self.s.mouse);
        let mut inside = self.s.hover_ok && m[0] >= sr[0] && m[0] < sr[0] + sr[2] && m[1] >= sr[1] && m[1] < sr[1] + sr[3];
        if let Some([a, b]) = self.clip {
            inside &= m[1] >= self.o[1] + a * k() && m[1] < self.o[1] + b * k();
        }
        self.s.anim(anim::HOVER | anim::pos_key(r, 0), inside as u8 as f32, 0.0, 22.0)
    }

    /// A state of the element at `r` (`salt` tells states apart), eased 0 to 1.
    fn ease(&mut self, r: [f32; 4], salt: u64, target: bool) -> f32 {
        let t = target as u8 as f32;
        if self.dry {
            return t;
        }
        self.s.anim(anim::STATE | anim::pos_key(r, salt), t, t, 20.0)
    }

    fn toggle(&mut self, hit: Hit, right: f32, cy: f32, on: bool) {
        let r = [right - 28.0, cy - 8.0, 28.0, 16.0];
        let t = self.ease(r, 1, on);
        let hv = self.hov(r);
        self.round(r, 8.0, gfx::mix(TG_RING, ACCENT, t));
        if t < 0.999 {
            self.round([r[0] + 1.0, r[1] + 1.0, 26.0, 14.0], 7.0, gfx::alpha(FIELD, 1.0 - t));
            if hv > 0.01 {
                self.ring(r, 8.0, gfx::alpha(0xFF4A_4A55, hv * (1.0 - t)));
            }
        }
        let kx = r[0] + 3.0 + 12.0 * t;
        if !self.dry {
            let (kx, ky) = (self.o[0] + (kx + 5.0) * k(), self.o[1] + (r[1] + 8.0) * k());
            gfx::circle(self.d, kx, ky, 5.0 * k(), gfx::mix(TG_KNOB, INK, t));
        }
        self.hit(hit, r);
    }

    /// A section header: small caps with a hairline running on to the card's edge.
    fn section(&mut self, name: &str, x: f32, y: f32, w: f32) {
        let f = REG(10);
        let mut cx = x;
        for ch in name.to_uppercase().chars() {
            let s = ch.to_string();
            self.text(&s, cx, y + 6.0, FAINT, f);
            cx += self.width(&s, f) + 0.8;
        }
        if !self.dry {
            let r = self.at([cx + 8.0, y + 6.0, (x + w - cx - 8.0).max(0.0), 1.0]);
            gfx::fill(self.d, r, LINE);
        }
    }

    fn dropdown(&mut self, mi: usize, si: usize, text: &str, right: f32, cy: f32, w: f32, h: f32) {
        let r = [right - w, cy - h / 2.0, w, h];
        let open = self.s.open == Some((mi, si));
        self.round(r, 6.0, FIELD);
        if open {
            self.ring(r, 6.0, ACCENT);
        }
        let shown = fit(self, text, w - 20.0 - 14.0, REG(12));
        self.text(&shown, r[0] + 10.0, cy, SOFT, REG(12));
        // The caret: a small chevron made of two slanted bars.
        if !self.dry {
            let (cx, cy2) = (self.o[0] + (right - 18.0) * k(), self.o[1] + cy * k());
            let a = 3.0 * k();
            gfx::quad(self.d, [[cx - a - 0.3, cy2 - a * 0.4], [cx - a + 0.3, cy2 - a * 0.4 - 0.3], [cx + 0.3, cy2 + a * 0.6], [cx - 0.3, cy2 + a * 0.6 + 0.3]], FAINT);
            gfx::quad(self.d, [[cx + a + 0.3, cy2 - a * 0.4], [cx + a - 0.3, cy2 - a * 0.4 - 0.3], [cx - 0.3, cy2 + a * 0.6], [cx + 0.3, cy2 + a * 0.6 + 0.3]], FAINT);
        }
        self.hit(Hit::Setting(mi, si), r);
        if open {
            let n = match self.s.mods[mi].settings[si].kind {
                Kind::Choice(o) | Kind::Multi(o) => o.len(),
                _ => 0,
            };
            let ph = 12.0 + n as f32 * 26.0 + n.saturating_sub(1) as f32 * 2.0;
            if !self.dry {
                self.popups.push(Popup::Options { mi, si, rect: [right - 140.0, r[1] + 32.0, 140.0, ph] });
            }
        }
    }
}

fn fit(p: &Pen<'_>, t: &str, max: f32, f: Face) -> String {
    if p.width(t, f) <= max {
        return t.to_owned();
    }
    let mut s = t.to_owned();
    while !s.is_empty() && p.width(&format!("{s}…"), f) > max {
        s.pop();
    }
    format!("{s}…")
}

#[derive(Clone, Copy)]
enum Card {
    General(usize),
    Group(usize, &'static str),
    Profile,
    Options,
}

impl Card {
    /// How much of the column it is meant to fill, as orca.html balances its two columns.
    fn weight(self, p: &Pen<'_>) -> f32 {
        match self {
            Card::Profile => 120.0,
            Card::Options => 90.0,
            Card::General(_) => 80.0,
            Card::Group(mi, g) => {
                40.0 + p.s.mods[mi]
                    .settings
                    .iter()
                    .filter(|s| s.group == g)
                    .map(|s| match s.kind {
                        Kind::Switch => 34.0,
                        Kind::Choice(_) | Kind::Multi(_) => 40.0,
                        Kind::Hotbar => 66.0,
                        Kind::Items => 72.0,
                        _ => 40.0,
                    })
                    .sum::<f32>()
            }
        }
    }
}

const PAD: f32 = 10.0;
/// The section header (12 high) and the 4 below it.
const HEAD: f32 = 16.0;

/// One card at (x, y) CSS pixels, `w` wide; returns its height.
fn card(p: &mut Pen<'_>, c: Card, x: f32, y: f32, w: f32) -> f32 {
    let title = match c {
        Card::General(_) => "General",
        Card::Group(_, g) => g,
        Card::Profile => "Profile",
        Card::Options => "Options",
    };
    let inner = y + PAD + HEAD;
    // The rows are laid out dry first to find the height the background needs.
    let was = p.dry;
    p.dry = true;
    let h = body(p, c, x + PAD, inner, w - 2.0 * PAD);
    p.dry = was;
    let total = PAD + HEAD + h + PAD;
    p.round([x, y, w, total], 10.0, CARD);
    p.section(title, x + PAD, y + PAD, w - 2.0 * PAD);
    body(p, c, x + PAD, inner, w - 2.0 * PAD);
    total
}

/// Rows of one card from `y` down; returns their height.
fn body(p: &mut Pen<'_>, c: Card, x: f32, y: f32, w: f32) -> f32 {
    let right = x + w;
    let mut cur = y;
    let gap = 4.0;
    let mut first = true;
    let mut next = |cur: &mut f32, h: f32| -> f32 {
        if !first {
            *cur += gap;
        }
        first = false;
        let top = *cur;
        *cur += h;
        top
    };
    match c {
        Card::General(mi) => {
            let on = p.s.on[mi];
            let t = next(&mut cur, 24.0);
            p.text("Enable", x, t + 12.0, if on { LABEL } else { FAINT }, REG(12));
            p.toggle(Hit::Toggle(mi), right, t + 12.0, on);
            let t = next(&mut cur, 24.0);
            p.text("Bind", x, t + 12.0, LABEL, REG(12));
            let r = [right - 110.0, t, 110.0, 24.0];
            let cap = p.s.capturing;
            let (label, col, f) = match (&p.s.key[mi], cap) {
                (_, true) => ("Press a key".to_owned(), ACCENT, REG(11)),
                (Some(k), _) => (key_label(k), TEXT, BOLD(11)),
                (None, _) => ("Not Bound".to_owned(), FAINT, REG(11)),
            };
            if cap {
                p.dashed(r, ACCENT);
            } else {
                p.round(r, 6.0, FIELD);
            }
            p.text_center(&label, r[0] + 55.0, t + 12.0, col, f);
            p.hit(Hit::Bind, r);
        }
        Card::Group(mi, g) => {
            for si in 0..p.s.mods[mi].settings.len() {
                let (group, label, unit, kind) = {
                    let s = &p.s.mods[mi].settings[si];
                    (s.group, s.label, s.unit, s.kind)
                };
                if group != g {
                    continue;
                }
                let val = p.s.vals[mi][si].clone();
                match (kind, &val) {
                    (Kind::Switch, Val::On(b)) => {
                        let t = next(&mut cur, 24.0);
                        p.text(label, x, t + 12.0, if *b { LABEL } else { FAINT }, REG(12));
                        p.toggle(Hit::Setting(mi, si), right, t + 12.0, *b);
                    }
                    (Kind::Choice(_) | Kind::Multi(_), v) => {
                        let t = next(&mut cur, 28.0);
                        p.text(label, x, t + 14.0, LABEL, REG(12));
                        let shown = match v {
                            Val::Set(set) => {
                                let Kind::Multi(o) = kind else { unreachable!() };
                                let on: Vec<&str> = o.iter().zip(set).filter(|(_, b)| **b).map(|(n, _)| *n).collect();
                                if on.is_empty() { "None".to_owned() } else { on.join(", ") }
                            }
                            _ => value_text(kind, unit, v),
                        };
                        p.dropdown(mi, si, &shown, right, t + 14.0, 140.0, 28.0);
                    }
                    (Kind::Slider { min, max, .. }, Val::Num(n)) => {
                        let t = next(&mut cur, 32.0);
                        p.text(label, x, t + 9.0, LABEL, REG(12));
                        p.text_right(&value_text(kind, unit, &val), right, t + 9.0, VALUE, REG(12));
                        let tr = [x, t + 21.0, w, 7.0];
                        p.round(tr, 4.0, TRACK);
                        let f = ((n - min) / (max - min)).clamp(0.0, 1.0);
                        let fw = (tr[2] * f).max(7.0);
                        p.round([tr[0], tr[1], fw, 7.0], 4.0, ACCENT);
                        knob(p, tr[0] + fw - 3.5, tr[1] + 3.5);
                        p.hit(Hit::Setting(mi, si), [tr[0], tr[1] - 8.0, tr[2], 23.0]);
                    }
                    (Kind::Range { min, max, .. }, Val::Range(a, b)) => {
                        let t = next(&mut cur, 32.0);
                        p.text(label, x, t + 9.0, LABEL, REG(12));
                        p.text_right(&value_text(kind, unit, &val), right, t + 9.0, VALUE, REG(12));
                        let tr = [x, t + 21.0, w, 7.0];
                        p.round(tr, 4.0, TRACK);
                        let (x0, x1) = (tr[2] * (a - min) / (max - min), tr[2] * (b - min) / (max - min));
                        p.round([tr[0] + x0, tr[1], (x1 - x0).max(7.0), 7.0], 4.0, ACCENT);
                        knob(p, tr[0] + x0 + 3.5, tr[1] + 3.5);
                        knob(p, tr[0] + x1.max(x0 + 7.0) - 3.5, tr[1] + 3.5);
                        p.hit(Hit::Setting(mi, si), [tr[0], tr[1] - 8.0, tr[2], 23.0]);
                    }
                    (Kind::Color, Val::Argb(col)) => {
                        let t = next(&mut cur, 24.0);
                        p.text(label, x, t + 12.0, LABEL, REG(12));
                        let r = [right - 110.0, t, 110.0, 24.0];
                        p.round(r, 6.0, FIELD);
                        p.round([r[0] + 10.0, t + 5.0, 14.0, 14.0], 4.0, *col | 0xFF00_0000);
                        p.text(&value_text(kind, unit, &val), r[0] + 32.0, t + 12.0, SOFT, REG(12));
                        p.hit(Hit::Setting(mi, si), r);
                    }
                    (Kind::Hotbar, Val::Set(v)) => {
                        let t = next(&mut cur, 4.0 + 15.0 + 6.0 + 28.0 + 4.0);
                        p.text(label, x, t + 2.0 + 7.5, LABEL, REG(12));
                        let sw = (w - 8.0 * 4.0) / 9.0;
                        for (i, on) in v.iter().enumerate() {
                            let r = [x + i as f32 * (sw + 4.0), t + 2.0 + 15.0 + 6.0, sw, 28.0];
                            p.round(r, 6.0, if *on { ACCENT } else { FIELD });
                            p.text_center(&(i + 1).to_string(), r[0] + sw / 2.0, r[1] + 14.0, if *on { INK } else { FAINT }, if *on { BOLD(11) } else { REG(11) });
                            p.hit(Hit::Slot(mi, si, i), r);
                        }
                    }
                    (Kind::Items, Val::Text(list)) => {
                        let items: Vec<&str> = list.split(',').map(str::trim).filter(|t| !t.is_empty()).collect();
                        // Chips wrap like the tag list: 24 high, 6 apart, 9 of padding to the left of the name.
                        let mut rows = 1;
                        let mut cx = 0.0;
                        let mut places = Vec::new();
                        let mut widths: Vec<f32> = items.iter().map(|n| p.width(&n.replace('_', " "), REG(12)) + 28.0 + 2.0 + 18.0 + 4.0).collect();
                        widths.push(p.width("+ Add", REG(12)) + 20.0);
                        for cw in &widths {
                            if cx > 0.0 && cx + cw > w {
                                rows += 1;
                                cx = 0.0;
                            }
                            places.push((cx, rows - 1, *cw));
                            cx += cw + 6.0;
                        }
                        let tags_h = rows as f32 * 24.0 + (rows - 1) as f32 * 6.0 + 4.0;
                        let t = next(&mut cur, 4.0 + 15.0 + 6.0 + tags_h + 2.0);
                        p.text(label, x, t + 2.0 + 7.5, LABEL, REG(12));
                        p.text_right(&format!("{} selected", items.len()), right, t + 2.0 + 7.5, VALUE, REG(12));
                        let top = t + 2.0 + 15.0 + 6.0 + 2.0;
                        for (i, (cx, row, cw)) in places.iter().enumerate() {
                            let r = [x + cx, top + *row as f32 * 30.0, *cw, 24.0];
                            if let Some(n) = items.get(i) {
                                p.round(r, 6.0, FIELD);
                                if !p.item(n, r[0] + 6.0, r[1] + 4.0, 16.0) {
                                    if let Some(c) = crate::items::ITEMS.iter().find(|i| i.0 == *n).map(|i| i.2) {
                                        p.round([r[0] + 8.0, r[1] + 6.0, 12.0, 12.0], 3.0, c);
                                    }
                                }
                                p.text(&n.replace('_', " "), r[0] + 28.0, r[1] + 12.0, SOFT, REG(12));
                                let cr = [r[0] + cw - 22.0, r[1] + 3.0, 18.0, 18.0];
                                let hv = p.hov(cr);
                                p.round(cr, 5.0, gfx::alpha(0xFF2C_2C33, hv));
                                p.text_center("×", cr[0] + 9.0, cr[1] + 9.0, gfx::mix(FAINT, TEXT, hv), REG(14));
                                p.hit(Hit::Chip(mi, si, i), cr);
                            } else {
                                let hv = p.hov(r);
                                p.dashed(r, gfx::mix(DASH, ACCENT, hv));
                                p.text_center("+ Add", r[0] + cw / 2.0, r[1] + 12.0, gfx::mix(MUTED, ACCENT, hv), REG(12));
                                p.hit(Hit::Chip(mi, si, i), r);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Card::Profile => {
            let sel = p.s.profile;
            let t = next(&mut cur, 24.0);
            p.text("Selected", x, t + 12.0, LABEL, REG(12));
            p.text_right(PROFILES[sel], right, t + 12.0, TEXT, REG(12));
            let t = next(&mut cur, 24.0);
            p.text("Status", x, t + 12.0, LABEL, REG(12));
            p.text_right(if p.s.active == sel { "Active" } else { "Not loaded" }, right, t + 12.0, MUTED, REG(12));
            let t = next(&mut cur, 26.0);
            let names = ["Save", "Load", "New", "Delete"];
            let widths: Vec<f32> = names.iter().map(|n| p.width(n, REG(12)) + 24.0).collect();
            let mut bx = x;
            for (i, (n, bw)) in names.iter().zip(widths).enumerate() {
                let r = [bx, t, bw, 26.0];
                p.round(r, 7.0, FIELD);
                p.text_center(n, bx + bw / 2.0, t + 13.0, SOFT, REG(12));
                p.hit(Hit::ProfileBtn(i), r);
                bx += bw + 6.0;
            }
        }
        Card::Options => {
            let rows = [("Auto save", p.s.auto, Hit::Auto), ("Load on start", p.s.start, Hit::Start), ("Array list", p.s.arraylist, Hit::Arraylist), ("Keybind list", p.s.keybinds, Hit::Keybinds)];
            for (name, on, hit) in rows {
                let t = next(&mut cur, 24.0);
                p.text(name, x, t + 12.0, if on { LABEL } else { FAINT }, REG(12));
                p.toggle(hit, right, t + 12.0, on);
            }
        }
    }
    cur - y
}

/// The 5px ink dot on a slider's end.
fn knob(p: &mut Pen<'_>, cx: f32, cy: f32) {
    if !p.dry {
        let (x, y) = (p.o[0] + cx * k(), p.o[1] + cy * k());
        gfx::circle(p.d, x, y, 2.5 * k(), INK);
    }
}

fn uptime(s: &State) -> String {
    let m = s.started.elapsed().as_secs() / 60;
    format!("Uptime {}hr {}min", m / 60, m % 60)
}

pub fn draw_window(s: &mut State, draw: &mut dyn DrawContext, host: &dyn ClientHost, size: [f32; 2]) {
    s.rects.clear();
    s.tick();
    // A window that was closed starts clean: no stale popup, capture or drag.
    if !s.anim.contains_key(&anim::WIN) {
        s.pick = None;
        s.open = None;
        s.capturing = false;
        s.drag = None;
        s.cdrag = None;
    }
    let win = s.anim(anim::WIN, 1.0, 0.0, 18.0);
    gfx::set_alpha(win);
    s.hover_ok = s.pick.is_none();
    let tier = gfx::TIERS.iter().position(|k| WIN_W * k <= size[0] - 8.0 && WIN_H * k <= size[1] - 8.0).unwrap_or(gfx::TIERS.len() - 1);
    gfx::set_tier(tier);
    gfx::fill(draw, [0.0, 0.0, size[0], size[1]], 0x8C08_090C);
    let o = [((size[0] - WIN_W * k()) / 2.0).floor(), ((size[1] - WIN_H * k()) / 2.0).floor() + ((1.0 - win) * 10.0 * k()).round()];
    let mut p = Pen { s, d: draw, host, o, dry: false, popups: Vec::new(), clip: None };
    p.round([0.0, 0.0, WIN_W, WIN_H], 14.0, 0xFF0F_0F11);

    // Tab bar: 52 high, the tabs centred, 2 apart; each 30 high with 12 of padding either side.
    let names: Vec<&str> = CATS.iter().map(|c| c.1).collect();
    let widths: Vec<f32> = names.iter().map(|n| p.width(n, REG(12)) + 24.0 + 14.0 + 7.0).collect();
    let total: f32 = widths.iter().sum::<f32>() + 2.0 * (names.len() - 1) as f32;
    let mut tx = (WIN_W - total) / 2.0;
    for (i, (n, tw)) in names.iter().zip(&widths).enumerate() {
        let r = [tx, 11.0, *tw, 30.0];
        let active = i == p.s.cat;
        let t = p.ease(r, 2, active);
        let hv = p.hov(r);
        if t > 0.01 {
            p.round(r, 7.0, gfx::alpha(TAB_ON, t));
        }
        let col = gfx::mix(MUTED, TEXT, t.max(hv * 0.7));
        let mid = p.mid(26.0, REG(12));
        icon(&mut p, i, tx + 12.0, mid - 7.0, col);
        p.text(n, tx + 12.0 + 14.0 + 7.0, 26.0, col, REG(12));
        p.hit(Hit::Tab(i), r);
        tx += tw + 2.0;
    }

    // Side list: 104 wide from x = 12, rows 28 high, 2 apart.
    let config = p.s.cat == CATS.len() - 1;
    let sel = p.s.selected();
    let mut ly = 52.0;
    let rows: Vec<(String, bool, bool, Option<String>, Hit)> = if config {
        PROFILES.iter().enumerate().map(|(i, n)| ((*n).to_owned(), i == p.s.active, i == p.s.profile, None, Hit::Profile(i))).collect()
    } else {
        (0..p.s.mods.len())
            .filter(|&i| p.s.mods[i].cat == CATS[p.s.cat].0)
            .map(|i| (p.s.mods[i].name.to_owned(), p.s.on[i], Some(i) == sel, p.s.key[i].as_deref().map(key_label), Hit::Module(i)))
            .collect()
    };
    for (name, on, selected, key, hit) in rows {
        let r = [12.0, ly, 104.0, 28.0];
        let t = p.ease(r, 3, selected);
        let hv = p.hov(r);
        let bg = t.max(hv * 0.6);
        if bg > 0.01 {
            p.round(r, 8.0, gfx::alpha(SEL, bg));
        }
        let onv = p.ease(r, 4, on);
        // The name leaves room for the key hint: smaller first, then cut short.
        let kw = key.as_ref().map_or(0.0, |k| p.width(k, REG(10)) + 8.0);
        let face = if p.width(&name, REG(12)) > 84.0 - kw { REG(11) } else { REG(12) };
        let shown = fit(&p, &name, 84.0 - kw, face);
        p.text(&shown, 22.0, ly + 14.0, gfx::mix(MUTED, TEXT, onv.max(hv * 0.5)), face);
        if let Some(k) = key {
            p.text_right(&k, 106.0, ly + 14.0, FAINT, REG(10));
        }
        p.hit(hit, r);
        ly += 30.0;
    }
    // The user block at the foot of the side.
    let me = 528.0 - 24.0;
    let who = p.host.local_player();
    let name = who
        .as_ref()
        .and_then(|w| p.host.client_session().client_tab_list().and_then(|l| l.entries.iter().find(|e| e.uuid == w.uuid()).map(|e| e.name.clone())))
        .unwrap_or_else(|| "User".to_owned());
    if !p.dry {
        let (cx, cy) = (p.o[0] + 28.0 * k(), p.o[1] + (me + 12.0) * k());
        let half = 12.0 * k();
        let head = who.as_ref().map(|w| {
            // The head fills the box (a bit over, so the chin cuts off the neck); the model's centre sits 0.74 block below the head's.
            let view = mod_api::GuiEntityView::new([cx - half, cy - half, cx + half, cy + half], 4.4 * half, 0.74, mod_api::GuiEntityLook::Turned { yaw: 0.0, pitch: 0.0 });
            p.d.entity(&mod_api::GuiEntity::Existing(w.clone().into()), view).is_ok()
        });
        if head != Some(true) {
            gfx::circle(p.d, cx, cy, half, ACCENT);
        }
    }
    let shown = fit(&p, &name, 72.0, REG(12));
    p.text(&shown, 44.0, me + 5.0, SOFT, REG(12));
    let up = uptime(p.s);
    p.text(&up, 44.0, me + 19.0, FAINT, REG(10));

    // Main: two columns of cards, each card going under the column that is lighter so far.
    let (colw, x0, top) = (327.0, 124.0, 52.0);
    let cards: Vec<Card> = if config {
        vec![Card::Profile, Card::Options]
    } else if let Some(mi) = sel {
        let mut v = vec![Card::General(mi)];
        let mut seen: Vec<&'static str> = Vec::new();
        for s in &p.s.mods[mi].settings {
            if !seen.contains(&s.group) {
                seen.push(s.group);
                v.push(Card::Group(mi, s.group));
            }
        }
        v
    } else {
        Vec::new()
    };
    let (mut wa, mut wb) = (0.0, 0.0);
    let (mut ya, mut yb) = (top, top);
    for c in cards {
        let w = c.weight(&p);
        let (x, y, ws, ys) = if wa <= wb { (x0, &mut ya, &mut wa, w) } else { (x0 + colw + 8.0, &mut yb, &mut wb, w) };
        let h = card(&mut p, c, x, *y, colw);
        *y += h + 8.0;
        *ws += ys;
    }

    for popup in std::mem::take(&mut p.popups) {
        match popup {
            Popup::Options { mi, si, rect } => {
                let a = p.s.anim(anim::POPUP | (mi * 64 + si) as u64, 1.0, 0.0, 26.0);
                gfx::set_alpha(win * a);
                let (Kind::Choice(opts) | Kind::Multi(opts)) = p.s.mods[mi].settings[si].kind else { continue };
                p.round([rect[0] - 1.0, rect[1] - 1.0, rect[2] + 2.0, rect[3] + 2.0], 11.0, POP_RING);
                p.round(rect, 10.0, POP);
                for (oi, name) in opts.iter().enumerate() {
                    let on = match &p.s.vals[mi][si] {
                        Val::Pick(i) => *i == oi,
                        Val::Set(v) => v.get(oi).copied().unwrap_or(false),
                        _ => false,
                    };
                    let r = [rect[0] + 6.0, rect[1] + 6.0 + oi as f32 * 28.0, rect[2] - 12.0, 26.0];
                let hv = p.hov(r);
                if hv > 0.01 {
                    p.round(r, 6.0, gfx::alpha(OPT_HOVER, hv));
                }
                    p.text(name, r[0] + 8.0, r[1] + 13.0, if on { TEXT } else { MUTED }, if on { BOLD(12) } else { REG(12) });
                    p.hit(Hit::Option(mi, si, oi), r);
                }
            }
        }
    }
    p.s.hover_ok = true;
    picker::draw(&mut p, win);
    gfx::set_alpha(1.0);
}

/// The tab icons, orca.html's 24-unit strokes scaled to 14 px.
fn icon(p: &mut Pen<'_>, which: usize, x: f32, y: f32, c: u32) {
    if p.dry {
        return;
    }
    let sc = 14.0 / 24.0 * k();
    let (ox, oy) = (p.o[0] + x * k(), p.o[1] + y * k());
    let pt = |a: f32, b: f32| [ox + a * sc, oy + b * sc];
    let th = 1.8 * sc;
    let d = &mut *p.d;
    let ring = |d: &mut dyn DrawContext, cx: f32, cy: f32, r: f32| {
        let n = 20;
        let pts: Vec<[f32; 2]> = (0..=n).map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            pt(cx + r * a.cos(), cy + r * a.sin())
        }).collect();
        gfx::polyline(d, &pts, th, c);
    };
    match which {
        0 => {
            gfx::polyline(d, &[pt(14.5, 4.5), pt(19.5, 9.5), pt(10.0, 19.0), pt(5.0, 20.0), pt(6.0, 15.0), pt(14.5, 4.5)], th, c);
            gfx::line(d, pt(3.0, 21.0), pt(6.0, 18.0), th, c);
        }
        1 => {
            gfx::polyline(d, &[pt(2.0, 12.0), pt(5.0, 8.5), pt(8.5, 6.0), pt(12.0, 5.0), pt(15.5, 6.0), pt(19.0, 8.5), pt(22.0, 12.0), pt(19.0, 15.5), pt(15.5, 18.0), pt(12.0, 19.0), pt(8.5, 18.0), pt(5.0, 15.5), pt(2.0, 12.0)], th, c);
            ring(d, 12.0, 12.0, 3.0);
        }
        2 => {
            gfx::line(d, pt(5.0, 12.0), pt(19.0, 12.0), th, c);
            gfx::polyline(d, &[pt(13.0, 6.0), pt(19.0, 12.0), pt(13.0, 18.0)], th, c);
        }
        3 => {
            ring(d, 12.0, 12.0, 9.0);
            gfx::line(d, pt(3.0, 12.0), pt(21.0, 12.0), th, c);
            gfx::polyline(d, &[pt(12.0, 3.0), pt(13.8, 7.0), pt(14.3, 12.0), pt(13.8, 17.0), pt(12.0, 21.0)], th, c);
            gfx::polyline(d, &[pt(12.0, 3.0), pt(10.2, 7.0), pt(9.7, 12.0), pt(10.2, 17.0), pt(12.0, 21.0)], th, c);
        }
        4 => {
            ring(d, 12.0, 8.0, 4.0);
            gfx::polyline(d, &[pt(4.0, 21.0), pt(5.0, 18.0), pt(8.0, 15.8), pt(12.0, 15.0), pt(16.0, 15.8), pt(19.0, 18.0), pt(20.0, 21.0)], th, c);
        }
        _ => {
            for (a, b) in [((4.0, 7.0), (14.0, 7.0)), ((18.0, 7.0), (20.0, 7.0)), ((4.0, 17.0), (6.0, 17.0)), ((10.0, 17.0), (20.0, 17.0))] {
                gfx::line(d, pt(a.0, a.1), pt(b.0, b.1), th, c);
            }
            ring(d, 16.0, 7.0, 2.0);
            ring(d, 8.0, 17.0, 2.0);
        }
    }
}
