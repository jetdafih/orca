//! Orca click GUI: Delete opens the window, Escape closes it. Settings and keybinds live here;
//! the modules only hold state, none of them acts on the game yet.
use mod_api::{
    self, ClientHost, ClientTickPreEvent, DrawContext, Flow, GuiRenderPostEvent, Hooks,
    InputKeyEvent, KeyAction, ScreenKeyPressedPreEvent, ScreenMouseButtonPressedPreEvent,
    ScreenMouseButtonReleasedPreEvent, ScreenCharTypedPreEvent, ScreenMouseDraggedPreEvent, ScreenMouseScrolledPreEvent, ScreenRenderPostEvent, ScreenRenderPreEvent, WindowResizeEvent,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

mod anim;
mod data;
mod gfx;
mod hud;
mod items;
mod ui;
use ui::picker::Pick;

const ID: &str = "orca";
const CATS: [(&str, &str); 6] = [
    ("combat", "Combat"),
    ("render", "Render"),
    ("movement", "Movement"),
    ("world", "World"),
    ("player", "Player"),
    ("config", "Config"),
];
const PROFILES: [&str; 3] = ["default", "legit", "pvp"];

const CARD: u32 = 0xFF15_1518;
const FIELD: u32 = 0xFF1E_1E23;
const TRACK: u32 = 0xFF2A_2A31;
const SEL: u32 = 0xFF1C_1C21;
const TEXT: u32 = 0xFFF4_F4F5;
const LABEL: u32 = 0xFFC9_C9D1;
const MUTED: u32 = 0xFF7E_7E88;
const ACCENT: u32 = 0xFF80_B4F5;


#[derive(Clone, Copy)]
pub enum Kind {
    Slider { min: f32, max: f32, step: f32 },
    Range { min: f32, max: f32, step: f32 },
    Switch,
    Choice(&'static [&'static str]),
    Multi(&'static [&'static str]),
    Color,
    Hotbar,
    Items,
}

#[derive(Clone)]
pub enum Val {
    Num(f32),
    Range(f32, f32),
    On(bool),
    Pick(usize),
    Set(Vec<bool>),
    Argb(u32),
    Text(String),
}

pub struct Setting {
    pub group: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub kind: Kind,
    pub def: Val,
}

pub struct Mod {
    pub cat: &'static str,
    pub name: &'static str,
    pub key: Option<&'static str>,
    pub on: bool,
    pub settings: Vec<Setting>,
}

#[derive(Clone, Copy)]
enum Hit {
    Tab(usize),
    Module(usize),
    Toggle(usize),
    Bind,
    Setting(usize, usize),
    Profile(usize),
    Arraylist,
    Keybinds,
    Auto,
    Start,
    Slot(usize, usize, usize),
    Chip(usize, usize, usize),
    Option(usize, usize, usize),
    ProfileBtn(usize),
    PkScrim,
    PkNone,
    PkClose,
    PkTag(usize),
    PkItem(usize),
    PkClear,
    PkAll,
    PkSwatch(usize),
}

struct State {
    mods: Vec<Mod>,
    vals: Vec<Vec<Val>>,
    on: Vec<bool>,
    key: Vec<Option<String>>,
    cat: usize,
    sel: [usize; 5],
    profile: usize,
    arraylist: bool,
    keybinds: bool,
    capturing: bool,
    /// The slider being dragged: module, setting, and which end of a range.
    drag: Option<(usize, usize, bool)>,
    rects: Vec<(Hit, [f32; 4])>,
    /// The dropdown that is open: module and setting.
    open: Option<(usize, usize)>,
    /// The profile loaded, as against the one selected.
    active: usize,
    auto: bool,
    start: bool,
    started: std::time::Instant,
    last_click: Option<(usize, std::time::Instant)>,
    pick: Option<Pick>,
    toasts: Vec<hud::Toast>,
    anim: std::collections::HashMap<u64, (f32, u64)>,
    frame: u64,
    dt: f32,
    last_tick: std::time::Instant,
    mouse: [f32; 2],
    hover_ok: bool,
}

impl State {
    fn new() -> Self {
        let mods = data::mods();
        let vals = mods.iter().map(|m| m.settings.iter().map(|s| s.def.clone()).collect()).collect();
        let on = mods.iter().map(|m| m.on).collect();
        let key = mods.iter().map(|m| m.key.map(str::to_owned)).collect();
        let mut sel = [0; 5];
        for (c, s) in sel.iter_mut().enumerate() {
            *s = mods.iter().position(|m| m.cat == CATS[c].0).unwrap_or(0);
        }
        Self {
            mods,
            vals,
            on,
            key,
            cat: 0,
            sel,
            profile: 0,
            arraylist: true,
            keybinds: true,
            capturing: false,
            drag: None,
            rects: Vec::new(),
            open: None,
            active: 0,
            auto: true,
            start: false,
            started: std::time::Instant::now(),
            last_click: None,
            pick: None,
            toasts: Vec::new(),
            anim: Default::default(),
            frame: 0,
            dt: 0.016,
            last_tick: std::time::Instant::now(),
            mouse: [-1.0, -1.0],
            hover_ok: true,
        }
    }

    fn selected(&self) -> Option<usize> {
        self.sel.get(self.cat).copied()
    }

    fn hit(&self, at: [f32; 2]) -> Option<Hit> {
        self.rects
            .iter()
            .rev()
            .find(|(_, r)| at[0] >= r[0] && at[0] < r[0] + r[2] && at[1] >= r[1] && at[1] < r[1] + r[3])
            .map(|(h, _)| *h)
    }

    fn track(&self, mi: usize, si: usize) -> Option<[f32; 4]> {
        self.rects
            .iter()
            .find(|(h, _)| matches!(h, Hit::Setting(m, s) if *m == mi && *s == si))
            .map(|(_, r)| *r)
    }

    /// Sets a slider or range value from the mouse x over its track.
    fn drag_to(&mut self, mi: usize, si: usize, high: bool, x: f32) {
        let Some(track) = self.track(mi, si) else { return };
        let (min, max, step) = match self.mods[mi].settings[si].kind {
            Kind::Slider { min, max, step } | Kind::Range { min, max, step } => (min, max, step),
            _ => return,
        };
        let t = ((x - track[0]) / track[2]).clamp(0.0, 1.0);
        let v = (((min + (max - min) * t) / step).round() * step).clamp(min, max);
        match &mut self.vals[mi][si] {
            Val::Num(n) => *n = v,
            Val::Range(lo, hi) if high => *hi = v.max(*lo),
            Val::Range(lo, hi) => *lo = v.min(*hi),
            _ => {}
        }
    }

    fn press(&mut self, at: [f32; 2]) {
        let hit = self.hit(at);
        if self.pick.is_some() {
            self.pick_press(hit);
            return;
        }
        let keep = match (hit, self.open) {
            (Some(Hit::Option(..)), _) => true,
            (Some(Hit::Setting(m, s)), Some((a, b))) => m == a && s == b,
            _ => false,
        };
        if !keep {
            self.open = None;
        }
        // Any click but the bind button gives up a pending key capture.
        if !matches!(hit, Some(Hit::Bind)) {
            self.capturing = false;
        }
        let Some(hit) = hit else { return };
        match hit {
            Hit::Tab(c) => self.cat = c,
            Hit::Module(m) => {
                // A second click on the same row inside 350 ms switches the module.
                let now = std::time::Instant::now();
                if self.last_click.is_some_and(|(l, t)| l == m && now.duration_since(t).as_millis() < 350) {
                    self.on[m] = !self.on[m];
                    self.last_click = None;
                } else {
                    self.last_click = Some((m, now));
                }
                self.sel[self.cat] = m;
            }
            Hit::Toggle(m) => self.on[m] = !self.on[m],
            Hit::Bind => self.capturing = true,
            Hit::Profile(p) => self.profile = p,
            Hit::ProfileBtn(b) => {
                if b == 1 {
                    self.active = self.profile;
                }
            }
            Hit::Arraylist => self.arraylist = !self.arraylist,
            Hit::Keybinds => self.keybinds = !self.keybinds,
            Hit::Auto => self.auto = !self.auto,
            Hit::Start => self.start = !self.start,
            Hit::Slot(mi, si, i) => {
                if let Val::Set(v) = &mut self.vals[mi][si] {
                    v[i] = !v[i];
                }
            }
            Hit::Chip(mi, si, i) => {
                if let Val::Text(t) = &mut self.vals[mi][si] {
                    if i >= t.split(',').filter(|x| !x.trim().is_empty()).count() {
                        self.pick = Some(Pick::Items { mi, si, query: String::new(), tag: 0, scroll: 0.0 });
                        return;
                    }
                    let mut items: Vec<&str> = t.split(',').map(str::trim).filter(|x| !x.is_empty()).collect();
                    if i < items.len() {
                        items.remove(i);
                    }
                    *t = items.join(", ");
                }
            }
            Hit::Option(mi, si, oi) => match &mut self.vals[mi][si] {
                Val::Pick(p) => {
                    *p = oi;
                    self.open = None;
                }
                Val::Set(v) => v[oi] = !v[oi],
                _ => {}
            },
            Hit::PkScrim | Hit::PkNone | Hit::PkClose | Hit::PkTag(_) | Hit::PkItem(_) | Hit::PkClear | Hit::PkAll | Hit::PkSwatch(_) => {}
            Hit::Setting(mi, si) => {
                let Some(track) = self.track(mi, si) else { return };
                let (kind, val) = (self.mods[mi].settings[si].kind, self.vals[mi][si].clone());
                match (kind, val) {
                    (_, Val::On(b)) => self.vals[mi][si] = Val::On(!b),
                    (Kind::Choice(_) | Kind::Multi(_), _) => self.open = if self.open == Some((mi, si)) { None } else { Some((mi, si)) },
                    (_, Val::Num(_)) => {
                        self.drag = Some((mi, si, false));
                        self.drag_to(mi, si, false, at[0]);
                    }
                    (Kind::Range { min, max, .. }, Val::Range(lo, hi)) => {
                        let px = |v: f32| track[0] + (v - min) / (max - min) * track[2];
                        let high = (at[0] - px(hi)).abs() < (at[0] - px(lo)).abs();
                        self.drag = Some((mi, si, high));
                        self.drag_to(mi, si, high, at[0]);
                    }
                    (Kind::Color, _) => self.pick = Some(Pick::Color { mi, si }),
                    _ => {}
                }
            }
        }
    }
}

fn value_text(kind: Kind, unit: &str, v: &Val) -> String {
    let dec = |step: f32| {
        let t = step.to_string();
        t.find('.').map_or(0, |i| t.len() - i - 1)
    };
    match (kind, v) {
        (Kind::Slider { step, .. }, Val::Num(n)) => format!("{:.*}{}", dec(step), n, unit),
        (Kind::Range { step, .. }, Val::Range(a, b)) => format!("{:.*} - {:.*}{}", dec(step), a, dec(step), b, unit),
        (Kind::Choice(o), Val::Pick(i)) => o.get(*i).copied().unwrap_or("").to_owned(),
        (Kind::Color, Val::Argb(c)) => format!("#{:06X}", c & 0xFF_FFFF),
        (Kind::Items, Val::Text(t)) => t.replace('_', " "),
        _ => String::new(),
    }
}

fn key_label(full: &str) -> String {
    full.strip_prefix("key.keyboard.").unwrap_or(full).replace(['.', '_'], " ").to_uppercase()
}

#[derive(Default)]
pub struct Orca {
    state: Mutex<Option<State>>,
    close: AtomicBool,
}

impl Orca {
    fn with<R>(&self, f: impl FnOnce(&mut State) -> R) -> R {
        let mut g = self.state.lock().unwrap();
        f(g.get_or_insert_with(State::new))
    }
}

#[mod_api::horizon_mod]
impl Hooks for Orca {
    fn input_key(&self, host: &mut dyn ClientHost, event: &mut InputKeyEvent<'_>) {
        if event.action != KeyAction::Press || !host.playing() || host.mod_screen().is_some() {
            return;
        }
        if event.key == "key.keyboard.delete" {
            host.mod_screen_open(ID);
            return;
        }
        let k = key_label(event.key);
        self.with(|s| {
            for i in 0..s.key.len() {
                if s.key[i].as_deref().is_some_and(|b| key_label(b) == k) {
                    s.on[i] = !s.on[i];
                    let (name, off) = (s.mods[i].name, !s.on[i]);
                    hud::push_toast(s, name, off);
                }
            }
        });
    }

    fn client_tick_pre(&self, host: &mut dyn ClientHost, _: &mut ClientTickPreEvent) {
        if self.close.swap(false, Ordering::Relaxed) {
            host.mod_screen_close(ID);
        }
    }

    /// The overlays sit under the window: drawn before it while it is open.
    fn screen_render_pre(&self, host: &dyn ClientHost, event: &mut ScreenRenderPreEvent<'_>) -> Flow {
        if host.mod_screen() == Some(ID) && host.local_player().is_some() {
            let size = event.draw.size();
            self.with(|s| hud::draw(s, &mut *event.draw, host, size));
        }
        Flow::Continue
    }

    fn screen_render_post(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        if host.mod_screen() != Some(ID) {
            return;
        }
        let size = event.draw.size();
        let mouse = event.screen.mouse;
        self.with(|s| {
            s.mouse = mouse;
            ui::draw_window(s, &mut *event.draw, host, size)
        });
    }

    fn screen_mouse_button_pressed_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseButtonPressedPreEvent<'_>) -> Flow {
        if host.mod_screen() != Some(ID) {
            return Flow::Continue;
        }
        if event.button == 0 {
            let at = event.screen.mouse;
            self.with(|s| s.press(at));
        }
        Flow::Cancel
    }

    fn screen_mouse_button_released_pre(&self, host: &dyn ClientHost, _: &mut ScreenMouseButtonReleasedPreEvent<'_>) -> Flow {
        if host.mod_screen() != Some(ID) {
            return Flow::Continue;
        }
        self.with(|s| s.drag = None);
        Flow::Cancel
    }

    fn screen_mouse_dragged_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseDraggedPreEvent<'_>) -> Flow {
        if host.mod_screen() != Some(ID) {
            return Flow::Continue;
        }
        let x = event.screen.mouse[0];
        self.with(|s| {
            if let Some((m, i, high)) = s.drag {
                s.drag_to(m, i, high, x);
            }
        });
        Flow::Cancel
    }

    fn screen_mouse_scrolled_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseScrolledPreEvent<'_>) -> Flow {
        if host.mod_screen() != Some(ID) {
            return Flow::Continue;
        }
        let dy = event.scroll[1] as f32;
        self.with(|s| {
            if let Some(Pick::Items { scroll, .. }) = &mut s.pick {
                *scroll = (*scroll - dy * 40.0).max(0.0);
            }
        });
        Flow::Cancel
    }

    fn screen_char_typed_pre(&self, host: &dyn ClientHost, event: &mut ScreenCharTypedPreEvent<'_>) -> Flow {
        if host.mod_screen() != Some(ID) {
            return Flow::Continue;
        }
        let c = event.c;
        self.with(|s| {
            if let Some(Pick::Items { query, scroll, .. }) = &mut s.pick {
                if !c.is_control() {
                    query.push(c);
                    *scroll = 0.0;
                }
            }
        });
        Flow::Cancel
    }

    fn screen_key_pressed_pre(&self, host: &dyn ClientHost, event: &mut ScreenKeyPressedPreEvent<'_>) -> Flow {
        if host.mod_screen() != Some(ID) {
            return Flow::Continue;
        }
        let key = event.key.to_owned();
        let in_pick = self.with(|s| {
            let Some(pick) = &mut s.pick else { return false };
            match (key.as_str(), pick) {
                ("key.keyboard.escape", _) => s.pick = None,
                ("key.keyboard.backspace", Pick::Items { query, scroll, .. }) => {
                    query.pop();
                    *scroll = 0.0;
                }
                _ => {}
            }
            true
        });
        if in_pick {
            return Flow::Cancel;
        }
        let captured = self.with(|s| {
            if !s.capturing {
                return false;
            }
            s.capturing = false;
            if let Some(m) = s.selected() {
                let clear = matches!(key.as_str(), "key.keyboard.escape" | "key.keyboard.backspace" | "key.keyboard.delete");
                s.key[m] = if clear { None } else { Some(key.clone()) };
            }
            true
        });
        if captured {
            return Flow::Cancel;
        }
        if key == "key.keyboard.delete" {
            self.close.store(true, Ordering::Relaxed);
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn window_resize(&self, _host: &mut dyn ClientHost, event: &mut WindowResizeEvent) {
        gfx::set_gui_scale(event.width, event.gui_width);
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        // The overlays belong to the game, not the menus.
        if event.gui_hidden || host.mod_screen() == Some(ID) || host.local_player().is_none() {
            return;
        }
        let size = event.batch.size();
        self.with(|s| hud::draw(s, &mut *event.batch, host, size));
    }
}
