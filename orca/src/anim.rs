//! Eased values for the click GUI: toggles, hovers, popups. A value chases its target a fixed
//! fraction per second, so it feels the same at any frame rate, and one that nothing asked for
//! in the last two frames is dropped, so it starts fresh when its element comes back.
use super::*;
use std::time::Instant;

pub const HOVER: u64 = 1 << 56;
pub const STATE: u64 = 2 << 56;
pub const POPUP: u64 = 3 << 56;
pub const HUD: u64 = 4 << 56;
pub const WIN: u64 = STATE | 0xFF << 48;

impl State {
    /// Once per frame, whoever draws first.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
        if dt < 0.002 {
            return;
        }
        self.last_tick = now;
        self.dt = dt.min(0.1);
        self.frame += 1;
        let frame = self.frame;
        self.anim.retain(|_, e| e.1.saturating_add(2) >= frame);
    }

    /// The eased value for `key`, moving toward `target`; `init` is where it starts.
    pub fn anim(&mut self, key: u64, target: f32, init: f32, rate: f32) -> f32 {
        let (frame, dt) = (self.frame, self.dt);
        let e = self.anim.entry(key).or_insert((init, frame.wrapping_sub(1)));
        if e.1 != frame {
            e.1 = frame;
            e.0 += (target - e.0) * (1.0 - (-dt * rate).exp());
            if (target - e.0).abs() < 0.003 {
                e.0 = target;
            }
        }
        e.0
    }
}

/// A key from where an element sits, for values that belong to one spot on screen.
pub fn pos_key(r: [f32; 4], salt: u64) -> u64 {
    let q = |v: f32| (v * 4.0) as i64 as u64 & 0x3FFFFF;
    salt << 48 & 0x00FF_0000_0000_0000 | q(r[0]) << 22 | q(r[1])
}
