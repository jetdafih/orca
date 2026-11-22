//! Shapes and text on top of `DrawContext`: rounded rectangles, rings and the Outfit font
//! that orca.html is set in. Everything is in GUI pixels.
use mod_api::text::TextStyle;
use mod_api::{ClientHost, DrawContext, GuiDraw, Text};


static ALPHA: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3F80_0000);

/// Scales the alpha of everything drawn from here on; 1.0 is untouched.
pub fn set_alpha(a: f32) {
    ALPHA.store(a.clamp(0.0, 1.0).to_bits(), std::sync::atomic::Ordering::Relaxed);
}

fn m(argb: u32) -> u32 {
    let a = f32::from_bits(ALPHA.load(std::sync::atomic::Ordering::Relaxed));
    if a >= 0.999 { argb } else { alpha(argb, a) }
}

pub fn fill(draw: &mut dyn DrawContext, rect: [f32; 4], argb: u32) {
    draw.fill(rect, m(argb));
}

pub fn quad(draw: &mut dyn DrawContext, pts: [[f32; 2]; 4], argb: u32) {
    draw.quad(pts, m(argb));
}

/// Blends `a` into `b`, all four channels.
pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ch = |s: u32| ((a >> s & 0xFF) as f32 * (1.0 - t) + (b >> s & 0xFF) as f32 * t).round() as u32;
    ch(24) << 24 | ch(16) << 16 | ch(8) << 8 | ch(0)
}

pub fn alpha(argb: u32, f: f32) -> u32 {
    let a = ((argb >> 24) as f32 * f).round() as u32;
    (a << 24) | (argb & 0x00FF_FFFF)
}

/// Segments per quarter circle: enough that a corner of radius `r` looks smooth at any GUI scale.
fn segments(r: f32) -> usize {
    ((r.sqrt() * 2.5).ceil() as usize).clamp(3, 14)
}

fn arc_point(c: [f32; 2], r: f32, a: f32) -> [f32; 2] {
    [c[0] + r * a.cos(), c[1] + r * a.sin()]
}

/// The four corner centres with the quarter-turn each arc starts at: TL, TR, BR, BL.
fn corners([x, y, w, h]: [f32; 4], r: f32) -> [([f32; 2], f32); 4] {
    [([x + r, y + r], 2.0), ([x + w - r, y + r], 3.0), ([x + w - r, y + h - r], 0.0), ([x + r, y + h - r], 1.0)]
}

/// Fan from `c` out to radius `r` over one quarter turn.
fn fan(draw: &mut dyn DrawContext, c: [f32; 2], r: f32, turn: f32, argb: u32) {
    let n = segments(r);
    let q = std::f32::consts::FRAC_PI_2;
    for i in 0..n {
        let a0 = (turn + i as f32 / n as f32) * q;
        let a1 = (turn + (i + 1) as f32 / n as f32) * q;
        let (p0, p1) = (arc_point(c, r, a0), arc_point(c, r, a1));
        quad(draw, [c, p0, p1, p1], argb);
    }
}

/// Band between radii `r0` and `r1` over one quarter turn.
fn band(draw: &mut dyn DrawContext, c: [f32; 2], r0: f32, r1: f32, turn: f32, argb: u32) {
    let n = segments(r1);
    let q = std::f32::consts::FRAC_PI_2;
    for i in 0..n {
        let a0 = (turn + i as f32 / n as f32) * q;
        let a1 = (turn + (i + 1) as f32 / n as f32) * q;
        quad(draw, [arc_point(c, r0, a0), arc_point(c, r1, a0), arc_point(c, r1, a1), arc_point(c, r0, a1)], argb);
    }
}

/// A filled rectangle with every corner rounded to `radius`. The pieces never overlap, so a
/// translucent colour comes out even. The edge gets a half-strength rim a half pixel wide.
pub fn round(draw: &mut dyn DrawContext, rect: [f32; 4], radius: f32, argb: u32) {
    let [x, y, w, h] = rect;
    let r = radius.min(w / 2.0).min(h / 2.0);
    if r < 0.5 {
        fill(draw, rect, argb);
        return;
    }
    fill(draw, [x + r, y, w - 2.0 * r, h], argb);
    if h > 2.0 * r {
        fill(draw, [x, y + r, r, h - 2.0 * r], argb);
        fill(draw, [x + w - r, y + r, r, h - 2.0 * r], argb);
    }
    let rim = alpha(argb, 0.5);
    for (c, turn) in corners(rect, r) {
        fan(draw, c, r - 0.25, turn, argb);
        band(draw, c, r - 0.25, r + 0.25, turn, rim);
    }
}

pub fn circle(draw: &mut dyn DrawContext, cx: f32, cy: f32, radius: f32, argb: u32) {
    round(draw, [cx - radius, cy - radius, radius * 2.0, radius * 2.0], radius, argb);
}

/// A rounded outline `th` thick, drawn inside `rect`.
pub fn ring(draw: &mut dyn DrawContext, rect: [f32; 4], radius: f32, th: f32, argb: u32) {
    let [x, y, w, h] = rect;
    let r = radius.min(w / 2.0).min(h / 2.0);
    if r < 0.5 {
        fill(draw, [x, y, w, th], argb);
        fill(draw, [x, y + h - th, w, th], argb);
        fill(draw, [x, y + th, th, h - 2.0 * th], argb);
        fill(draw, [x + w - th, y + th, th, h - 2.0 * th], argb);
        return;
    }
    fill(draw, [x + r, y, w - 2.0 * r, th], argb);
    fill(draw, [x + r, y + h - th, w - 2.0 * r, th], argb);
    fill(draw, [x, y + r, th, h - 2.0 * r], argb);
    fill(draw, [x + w - th, y + r, th, h - 2.0 * r], argb);
    for (c, turn) in corners(rect, r) {
        band(draw, c, (r - th).max(0.0), r, turn, argb);
    }
}

/// A face: the CSS pixel size orca.html sets a line in (10, 11, 12, 14 or 16) and the weight.
#[derive(Clone, Copy)]
pub struct Face(pub u8, pub bool);

/// The window scales the stylesheet may use, largest first; each has its own font files.
pub const TIERS: [f32; 4] = [0.6, 0.5, 0.42, 0.35];
static TIER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
/// The game's GUI scale. Each font file is rasterised at that many texels per GUI pixel, so one texel is one
/// screen pixel; 3 until the window reports its size.
static GUI_SCALE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(3);

pub fn set_tier(t: usize) {
    TIER.store(t, std::sync::atomic::Ordering::Relaxed);
}

pub fn set_gui_scale(width: u32, gui_width: u32) {
    if gui_width > 0 {
        let s = (width as f32 / gui_width as f32).round() as usize;
        GUI_SCALE.store(s.clamp(1, 6), std::sync::atomic::Ordering::Relaxed);
    }
}

/// Glyphs hang from a baseline 7 GUI px below the text's y whatever the size, so a line is centred on `cy`
/// from its face's size, not from a box that scales with the window.
pub fn text_y(cy: f32, face: Face) -> f32 {
    cy - 7.0 + 0.30 * face.0 as f32 * scale() * 1.12
}

/// Where the middle of a line placed at `y` lands once the game snaps it to a whole GUI pixel.
pub fn text_mid(y: f32, face: Face) -> f32 {
    y.round() + 7.0 - 0.30 * face.0 as f32 * scale() * 1.12
}

pub fn scale() -> f32 {
    TIERS[TIER.load(std::sync::atomic::Ordering::Relaxed)]
}

fn styled(s: &str, face: Face) -> Text {
    let mut style = TextStyle::default();
    style.font = Some(format!("orca:{}{}t{}s{}", if face.1 { 'b' } else { 'o' }, face.0, TIER.load(std::sync::atomic::Ordering::Relaxed), GUI_SCALE.load(std::sync::atomic::Ordering::Relaxed)));
    Text::literal(s).with_style(style)
}

/// Text with its top-left at `at`. The game places glyphs on whole GUI pixels.
pub fn text(draw: &mut dyn DrawContext, s: &str, at: [f32; 2], argb: u32, face: Face) {
    let _ = draw.gui_paint(&[GuiDraw::Text {
        text: styled(s, face),
        x: at[0].round() as i32,
        y: at[1].round() as i32,
        color: m(argb),
        shadow: false,
    }]);
}

pub fn text_width(host: &dyn ClientHost, s: &str, face: Face) -> f32 {
    host.text_width(&styled(s, face)).map(|w| w as f32).unwrap_or(0.0)
}

/// A straight line `th` thick.
pub fn line(draw: &mut dyn DrawContext, a: [f32; 2], b: [f32; 2], th: f32, argb: u32) {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let (nx, ny) = (-dy / len * th / 2.0, dx / len * th / 2.0);
    quad(draw, [[a[0] + nx, a[1] + ny], [b[0] + nx, b[1] + ny], [b[0] - nx, b[1] - ny], [a[0] - nx, a[1] - ny]], argb);
}

pub fn polyline(draw: &mut dyn DrawContext, pts: &[[f32; 2]], th: f32, argb: u32) {
    for w in pts.windows(2) {
        line(draw, w[0], w[1], th, argb);
    }
}
