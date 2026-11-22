// Generated from orca.html MODS; the settings each module shows.
use crate::{Kind, Mod, Setting, Val};

pub fn mods() -> Vec<Mod> {
    vec![
        Mod { cat: "combat", name: "Aura", key: None, on: true, settings: vec![
            Setting { group: "Targeting", label: "Mode", unit: "", kind: Kind::Choice(&["Single", "Switch", "Multi"]), def: Val::Pick(0) },
            Setting { group: "Targeting", label: "Priority", unit: "", kind: Kind::Choice(&["Distance", "Health", "Angle"]), def: Val::Pick(0) },
            Setting { group: "Targeting", label: "Target type", unit: "", kind: Kind::Multi(&["Players", "Mobs", "Animals", "Villagers", "Invisible", "Teammates"]), def: Val::Set(vec![true, true, false, false, false, false]) },
            Setting { group: "Tuning", label: "Speed", unit: " cps", kind: Kind::Slider { min: 1.0, max: 20.0, step: 1.0 }, def: Val::Num(11.0) },
            Setting { group: "Tuning", label: "Range", unit: "", kind: Kind::Range { min: 0.0, max: 5.0, step: 0.1 }, def: Val::Range(0.9, 3.6) },
            Setting { group: "Tuning", label: "Delay", unit: " ms", kind: Kind::Slider { min: 0.0, max: 300.0, step: 5.0 }, def: Val::Num(120.0) },
            Setting { group: "Tuning", label: "Color", unit: "", kind: Kind::Color, def: Val::Argb(0xFF80B4F5) },
            Setting { group: "Whitelist", label: "Hotbar slots", unit: "", kind: Kind::Hotbar, def: Val::Set(vec![true, true, true, false, false, false, false, false, false]) },
            Setting { group: "Whitelist", label: "Items", unit: "", kind: Kind::Items, def: Val::Text("diamond_sword, netherite_sword, diamond_axe".into()) },
            Setting { group: "Options", label: "Players only", unit: "", kind: Kind::Switch, def: Val::On(false) },
            Setting { group: "Options", label: "Require click", unit: "", kind: Kind::Switch, def: Val::On(true) },
            Setting { group: "Options", label: "Weapons only", unit: "", kind: Kind::Switch, def: Val::On(true) },
            Setting { group: "Options", label: "Through walls", unit: "", kind: Kind::Switch, def: Val::On(false) },
            Setting { group: "Options", label: "Keep target", unit: "", kind: Kind::Switch, def: Val::On(false) },
        ] },
        Mod { cat: "combat", name: "Velocity", key: Some("V"), on: true, settings: vec![
            Setting { group: "Tuning", label: "Horizontal", unit: "%", kind: Kind::Slider { min: 0.0, max: 100.0, step: 1.0 }, def: Val::Num(100.0) },
            Setting { group: "Tuning", label: "Vertical", unit: "%", kind: Kind::Slider { min: 0.0, max: 100.0, step: 1.0 }, def: Val::Num(100.0) },
            Setting { group: "Options", label: "Only while targeting", unit: "", kind: Kind::Switch, def: Val::On(false) },
        ] },
        Mod { cat: "combat", name: "Reach", key: None, on: false, settings: vec![
            Setting { group: "Tuning", label: "Distance", unit: " blocks", kind: Kind::Slider { min: 3.0, max: 6.0, step: 0.1 }, def: Val::Num(3.5) },
            Setting { group: "Whitelist", label: "Items", unit: "", kind: Kind::Items, def: Val::Text("diamond_sword".into()) },
            Setting { group: "Options", label: "Sprint only", unit: "", kind: Kind::Switch, def: Val::On(false) },
            Setting { group: "Options", label: "Weapons only", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "combat", name: "Criticals", key: Some("C"), on: false, settings: vec![
            Setting { group: "Targeting", label: "Mode", unit: "", kind: Kind::Choice(&["Packet", "Jump", "Edge"]), def: Val::Pick(0) },
            Setting { group: "Tuning", label: "Delay", unit: " ms", kind: Kind::Slider { min: 0.0, max: 500.0, step: 10.0 }, def: Val::Num(150.0) },
        ] },
        Mod { cat: "combat", name: "AutoClicker", key: Some("G"), on: true, settings: vec![
            Setting { group: "Tuning", label: "CPS", unit: "", kind: Kind::Range { min: 1.0, max: 20.0, step: 1.0 }, def: Val::Range(8.0, 12.0) },
            Setting { group: "Tuning", label: "Jitter", unit: "", kind: Kind::Slider { min: 0.0, max: 10.0, step: 1.0 }, def: Val::Num(2.0) },
            Setting { group: "Whitelist", label: "Hotbar slots", unit: "", kind: Kind::Hotbar, def: Val::Set(vec![true, false, false, false, false, false, false, false, false]) },
            Setting { group: "Options", label: "Left click", unit: "", kind: Kind::Switch, def: Val::On(true) },
            Setting { group: "Options", label: "Right click", unit: "", kind: Kind::Switch, def: Val::On(false) },
            Setting { group: "Options", label: "Block hit", unit: "", kind: Kind::Switch, def: Val::On(false) },
        ] },
        Mod { cat: "combat", name: "HitBoxes", key: Some("H"), on: false, settings: vec![
            Setting { group: "Tuning", label: "Expand", unit: " blocks", kind: Kind::Slider { min: 0.0, max: 0.5, step: 0.01 }, def: Val::Num(0.1) },
            Setting { group: "Options", label: "Players only", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "combat", name: "TriggerBot", key: Some("T"), on: false, settings: vec![
            Setting { group: "Tuning", label: "Delay", unit: " ms", kind: Kind::Slider { min: 0.0, max: 300.0, step: 5.0 }, def: Val::Num(60.0) },
            Setting { group: "Whitelist", label: "Hotbar slots", unit: "", kind: Kind::Hotbar, def: Val::Set(vec![true, true, false, false, false, false, false, false, false]) },
            Setting { group: "Whitelist", label: "Items", unit: "", kind: Kind::Items, def: Val::Text("diamond_sword, diamond_axe".into()) },
            Setting { group: "Options", label: "Require weapon", unit: "", kind: Kind::Switch, def: Val::On(true) },
            Setting { group: "Options", label: "Players only", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "render", name: "ESP", key: None, on: true, settings: vec![
            Setting { group: "Style", label: "Mode", unit: "", kind: Kind::Choice(&["Box", "Outline", "Glow"]), def: Val::Pick(0) },
            Setting { group: "Style", label: "Color", unit: "", kind: Kind::Color, def: Val::Argb(0xFF80B4F5) },
            Setting { group: "Tuning", label: "Width", unit: " px", kind: Kind::Slider { min: 1.0, max: 5.0, step: 0.5 }, def: Val::Num(2.0) },
            Setting { group: "Options", label: "Players", unit: "", kind: Kind::Switch, def: Val::On(true) },
            Setting { group: "Options", label: "Mobs", unit: "", kind: Kind::Switch, def: Val::On(false) },
        ] },
        Mod { cat: "render", name: "Tracers", key: None, on: false, settings: vec![
            Setting { group: "Style", label: "Color", unit: "", kind: Kind::Color, def: Val::Argb(0xFF80B4F5) },
            Setting { group: "Tuning", label: "Width", unit: " px", kind: Kind::Slider { min: 1.0, max: 5.0, step: 0.5 }, def: Val::Num(1.5) },
            Setting { group: "Options", label: "Players", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "render", name: "FullBright", key: None, on: true, settings: vec![
            Setting { group: "Style", label: "Mode", unit: "", kind: Kind::Choice(&["Gamma", "Potion"]), def: Val::Pick(0) },
        ] },
        Mod { cat: "render", name: "Zoom", key: Some("Z"), on: false, settings: vec![
            Setting { group: "Tuning", label: "Zoom", unit: "×", kind: Kind::Slider { min: 1.0, max: 10.0, step: 0.5 }, def: Val::Num(4.0) },
            Setting { group: "Options", label: "Smooth", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "render", name: "Watermark", key: None, on: true, settings: vec![
            Setting { group: "Style", label: "Style", unit: "", kind: Kind::Choice(&["Logo + text", "Logo only", "Text only"]), def: Val::Pick(0) },
            Setting { group: "Style", label: "Position", unit: "", kind: Kind::Choice(&["Top left", "Top center", "Bottom left", "Bottom right"]), def: Val::Pick(0) },
            Setting { group: "Tuning", label: "Scale", unit: "×", kind: Kind::Slider { min: 0.5, max: 2.5, step: 0.1 }, def: Val::Num(1.0) },
            Setting { group: "Options", label: "Background", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "render", name: "Nametags", key: None, on: false, settings: vec![
            Setting { group: "Tuning", label: "Scale", unit: "×", kind: Kind::Slider { min: 0.5, max: 2.0, step: 0.1 }, def: Val::Num(1.0) },
            Setting { group: "Options", label: "Show health", unit: "", kind: Kind::Switch, def: Val::On(true) },
            Setting { group: "Options", label: "Show armor", unit: "", kind: Kind::Switch, def: Val::On(false) },
        ] },
        Mod { cat: "movement", name: "Sprint", key: None, on: true, settings: vec![
            Setting { group: "Style", label: "Mode", unit: "", kind: Kind::Choice(&["Legit", "Omni"]), def: Val::Pick(0) },
        ] },
        Mod { cat: "movement", name: "Fly", key: Some("F"), on: false, settings: vec![
            Setting { group: "Style", label: "Mode", unit: "", kind: Kind::Choice(&["Vanilla", "Glide"]), def: Val::Pick(0) },
            Setting { group: "Tuning", label: "Speed", unit: " b/s", kind: Kind::Slider { min: 0.1, max: 5.0, step: 0.1 }, def: Val::Num(1.5) },
        ] },
        Mod { cat: "movement", name: "Speed", key: None, on: false, settings: vec![
            Setting { group: "Tuning", label: "Speed", unit: "×", kind: Kind::Slider { min: 1.0, max: 3.0, step: 0.05 }, def: Val::Num(1.2) },
        ] },
        Mod { cat: "movement", name: "Step", key: None, on: false, settings: vec![
            Setting { group: "Tuning", label: "Height", unit: " blocks", kind: Kind::Slider { min: 1.0, max: 3.0, step: 0.5 }, def: Val::Num(1.0) },
        ] },
        Mod { cat: "movement", name: "NoSlow", key: None, on: true, settings: vec![
            Setting { group: "Whitelist", label: "Items", unit: "", kind: Kind::Items, def: Val::Text("shield, golden_apple, bow".into()) },
            Setting { group: "Options", label: "Items", unit: "", kind: Kind::Switch, def: Val::On(true) },
            Setting { group: "Options", label: "Sneak", unit: "", kind: Kind::Switch, def: Val::On(false) },
        ] },
        Mod { cat: "world", name: "Scaffold", key: None, on: false, settings: vec![
            Setting { group: "Style", label: "Mode", unit: "", kind: Kind::Choice(&["Normal", "Telly"]), def: Val::Pick(0) },
            Setting { group: "Tuning", label: "Delay", unit: " ms", kind: Kind::Slider { min: 0.0, max: 200.0, step: 10.0 }, def: Val::Num(50.0) },
            Setting { group: "Whitelist", label: "Hotbar slots", unit: "", kind: Kind::Hotbar, def: Val::Set(vec![false, false, true, true, true, true, true, true, true]) },
            Setting { group: "Whitelist", label: "Items", unit: "", kind: Kind::Items, def: Val::Text("cobblestone, oak_planks, white_wool".into()) },
            Setting { group: "Options", label: "Tower", unit: "", kind: Kind::Switch, def: Val::On(true) },
            Setting { group: "Options", label: "Safe walk", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "world", name: "AutoTool", key: None, on: true, settings: vec![
            Setting { group: "Whitelist", label: "Hotbar slots", unit: "", kind: Kind::Hotbar, def: Val::Set(vec![true, true, true, true, true, false, false, false, false]) },
            Setting { group: "Options", label: "Swap back", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "world", name: "Nuker", key: None, on: false, settings: vec![
            Setting { group: "Tuning", label: "Range", unit: " blocks", kind: Kind::Slider { min: 1.0, max: 6.0, step: 0.5 }, def: Val::Num(4.0) },
            Setting { group: "Whitelist", label: "Items", unit: "", kind: Kind::Items, def: Val::Text("diamond_pickaxe, netherite_pickaxe".into()) },
            Setting { group: "Options", label: "Instant", unit: "", kind: Kind::Switch, def: Val::On(false) },
        ] },
        Mod { cat: "world", name: "Timer", key: None, on: false, settings: vec![
            Setting { group: "Tuning", label: "Speed", unit: "×", kind: Kind::Slider { min: 0.5, max: 3.0, step: 0.1 }, def: Val::Num(1.0) },
        ] },
        Mod { cat: "player", name: "AutoRespawn", key: None, on: false, settings: vec![
            Setting { group: "Options", label: "Instant", unit: "", kind: Kind::Switch, def: Val::On(true) },
        ] },
        Mod { cat: "player", name: "FastPlace", key: None, on: false, settings: vec![
            Setting { group: "Tuning", label: "Delay", unit: " ticks", kind: Kind::Slider { min: 0.0, max: 4.0, step: 1.0 }, def: Val::Num(0.0) },
            Setting { group: "Whitelist", label: "Items", unit: "", kind: Kind::Items, def: Val::Text("obsidian".into()) },
        ] },
        Mod { cat: "player", name: "NoFall", key: None, on: false, settings: vec![
            Setting { group: "Style", label: "Mode", unit: "", kind: Kind::Choice(&["Packet", "Spoof"]), def: Val::Pick(0) },
        ] },
    ]
}
