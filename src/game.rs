//! game — resources, state machine, and simulation systems.
//!
//! The game's user-facing strings all come through the embedded locale
//! bundles (assets/locales; the itotori leg) — no literals in systems.

use crate::levels::{levels, Level};
use crate::physics::{self, Bike, Ev, Input};
use bevy::prelude::*;

pub const SCORE_BASE: i32 = 250;
pub const FLIP_BONUS: i32 = 100;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    Intro,
    Playing,
    Crashed,
    Cleared,
    AllClear,
}

#[derive(Resource)]
pub struct GameRes {
    pub levels: Vec<Level>,
    pub current: usize,
    pub bike: Bike,
    pub time: f32,
    pub flips: usize,
    pub level_score: i32,
    pub total: i32,
    pub breakdown: (i32, i32, i32), // base, flips, time
    pub ragdoll: Option<Vec<physics::RagPart>>,
    pub crash_timer: f32,
    pub intro_timer: f32,
    pub hint_fade: f32,
    pub flip_toast: (String, f32),
    pub events: Vec<Ev>,
    pub best: i32,
}

impl Default for GameRes {
    fn default() -> Self {
        Self {
            levels: levels(),
            current: 0,
            bike: Bike::new(&levels()[0]),
            time: 0.0,
            flips: 0,
            level_score: 0,
            total: 0,
            breakdown: (0, 0, 0),
            ragdoll: None,
            crash_timer: 0.0,
            intro_timer: 0.0,
            hint_fade: 0.0,
            flip_toast: (String::new(), 0.0),
            events: Vec::new(),
            best: 0,
        }
    }
}

impl GameRes {
    pub fn level(&self) -> &Level {
        &self.levels[self.current]
    }

    pub fn start_level(&mut self, i: usize) {
        self.current = i;
        self.bike = Bike::new(&self.levels[i]);
        self.time = 0.0;
        self.flips = 0;
        self.level_score = 0;
        self.ragdoll = None;
        self.crash_timer = 0.0;
        self.intro_timer = 0.0;
        self.hint_fade = 4.5;
        self.flip_toast = (String::new(), 0.0);
        self.events.clear();
    }
}

#[derive(Resource, Default)]
pub struct InputRes {
    pub input: Input,
}

#[derive(Resource, Default)]
pub struct CameraRes {
    pub x: f32,
    pub y: f32,
    pub init: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Resource)]
pub enum Lang {
    En,
    Ja,
}

/// button actions (ui wiring)
#[derive(Component, Clone, Copy)]
pub enum BtnAction {
    Play,
    About,
    AboutBack,
    Retry,
    Next,
    Menu,
    LangEn,
    LangJa,
}

/// particle for dust/splash/feathers
#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub r: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
}

#[derive(Resource, Default)]
pub struct Particles {
    pub items: Vec<Particle>,
}

/// deterministic rng (seeded lcg) for particles — the world itself has none
#[derive(Resource)]
pub struct Rng(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    /// The control check's gameplay lane asserts
    /// `document.documentElement.dataset.pelicanState === 'Playing'` (wasm
    /// shell bridge in main.rs). This pins the exact published strings —
    /// renaming a variant must not silently republish a different word.
    #[test]
    fn shell_state_names_are_the_ci_contract() {
        assert_eq!(shell_state_name(&AppState::Title), "Title");
        assert_eq!(shell_state_name(&AppState::Intro), "Intro");
        assert_eq!(shell_state_name(&AppState::Playing), "Playing");
        assert_eq!(shell_state_name(&AppState::Crashed), "Crashed");
        assert_eq!(shell_state_name(&AppState::Cleared), "Cleared");
        assert_eq!(shell_state_name(&AppState::AllClear), "AllClear");
    }
}

impl Rng {
    pub fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32)
    }
}

/// run the fixed-timestep simulation; emits events consumed by fx systems
pub fn sim_system(
    state: Res<State<AppState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game: ResMut<GameRes>,
    input: Res<InputRes>,
    texts: Res<crate::ui::Texts>,
    lang: Res<Lang>,
    mut fx: ResMut<Particles>,
    mut rng: ResMut<Rng>,
    mut cam: ResMut<CameraRes>,
) {
    let dt = physics::SUBSTEP;
    match state.get() {
        AppState::Intro => {
            game.intro_timer += dt;
            if game.intro_timer > 1.6 {
                app_state.set(AppState::Playing);
            }
        }
        AppState::Playing => {
            let g: &mut GameRes = &mut game;
            g.time += dt;
            let lvl = g.current;
            physics::step(&mut g.bike, &g.levels[lvl], input.input, dt, g.time, &mut g.events);
            if g.hint_fade > 0.0 {
                g.hint_fade -= dt;
            }
            if g.flip_toast.1 > 0.0 {
                g.flip_toast.1 -= dt;
            }
            let mut next: Option<AppState> = None;
            let par = g.levels[g.current].par;
            let taken = std::mem::take(&mut g.events);
            for ev in taken {
                match ev {
                    Ev::Land { impact: _ } => {
                        emit(&mut fx, &mut rng, "dust", g.bike.x, g.bike.y + 24.0, 8);
                    }
                    Ev::Flip { count, .. } => {
                        g.flips = count;
                        g.flip_toast = (texts.t(*lang, "msg.flip"), 1.4);
                    }
                    Ev::Splash => {
                        emit(&mut fx, &mut rng, "splash", g.bike.x, g.bike.y, 22);
                    }
                    Ev::Crash { reason } => {
                        if reason == "beak" || reason == "overrotation" {
                            emit(&mut fx, &mut rng, "feathers", g.bike.x, g.bike.y - 20.0, 14);
                        }
                        g.ragdoll = Some(physics::make_ragdoll(&g.bike));
                        g.crash_timer = 0.0;
                        next = Some(AppState::Crashed);
                    }
                    Ev::Finish => {
                        let time_bonus = (((par - g.time) * 12.0).round() as i32).max(0);
                        let flip_b = g.flips as i32 * FLIP_BONUS;
                        g.breakdown = (SCORE_BASE, flip_b, time_bonus);
                        g.level_score = SCORE_BASE + flip_b + time_bonus;
                        g.total += g.level_score;
                        emit(&mut fx, &mut rng, "feathers", g.bike.x, g.bike.y - 30.0, 18);
                        next = Some(AppState::Cleared);
                    }
                }
            }
            if let Some(s) = next {
                app_state.set(s);
            }
        }
        AppState::Crashed => {
            let g: &mut GameRes = &mut game;
            g.crash_timer += dt;
            let lvl = g.current;
            if let Some(rag) = g.ragdoll.as_mut() {
                physics::step_ragdoll(rag, &g.levels[lvl], dt);
            }
            // the bike tumbles on, riderless
            let coast = Input { throttle: false, brake: true, lean_left: false, lean_right: false };
            let lvl = g.current;
            physics::step(&mut g.bike, &g.levels[lvl], coast, dt, g.time, &mut Vec::new());
        }
        _ => {}
    }

    // particles
    for p in fx.items.iter_mut() {
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.vy += 260.0 * dt;
        p.life -= dt;
    }
    fx.items.retain(|p| p.life > 0.0);

    // camera target (smoothed in world.rs's camera system)
    let bike = &game.bike;
    let target_x = bike.x + (bike.vx * 0.42).clamp(-40.0, 150.0);
    let target_y = bike.y - 40.0;
    cam.x = target_x;
    cam.y = target_y;
    cam.init = true;
}

fn emit(fx: &mut Particles, rng: &mut Rng, kind: &str, x: f32, y: f32, n: usize) {
    let palette: &[Color] = match kind {
        "dust" => &[Color::srgb(0.847, 0.855, 0.831), Color::srgb(0.769, 0.784, 0.769)],
        "splash" => &[Color::srgb(0.498, 0.714, 0.831), Color::srgb(0.949, 0.973, 0.988)],
        _ => &[Color::srgb(0.973, 0.965, 0.933), Color::srgb(0.949, 0.973, 0.988)],
    };
    for _ in 0..n {
        let a = rng.next_f32() * std::f32::consts::TAU;
        let sp = 40.0 + rng.next_f32() * 130.0;
        let life = 0.5 + rng.next_f32() * 0.6;
        fx.items.push(Particle {
            x,
            y,
            vx: a.cos() * sp,
            vy: a.sin() * sp - if kind == "splash" { 120.0 } else { 0.0 },
            r: 2.0 + rng.next_f32() * 3.4,
            life,
            max_life: life,
            color: palette[(rng.next_f32() * palette.len() as f32) as usize % palette.len()],
        });
    }
}

/// keyboard input
pub fn keyboard_system(mut input: ResMut<InputRes>, keys: Res<ButtonInput<KeyCode>>, state: Res<State<AppState>>, mut next: ResMut<NextState<AppState>>, mut game: ResMut<GameRes>) {
    input.input.throttle = keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD, KeyCode::ArrowUp, KeyCode::KeyW]);
    input.input.brake = keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]);
    input.input.lean_left = keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]);
    input.input.lean_right = keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]);

    if keys.just_pressed(KeyCode::KeyR) {
        match state.get() {
            AppState::Playing | AppState::Crashed | AppState::Cleared => {
                let cur = game.current;
                game.start_level(cur);
                next.set(AppState::Intro);
            }
            _ => {}
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::Playing | AppState::Crashed | AppState::Cleared | AppState::AllClear => {
                game.total = 0;
                next.set(AppState::Title);
            }
            _ => {}
        }
    }
    if keys.just_pressed(KeyCode::Enter) && *state.get() == AppState::Cleared {
        if let Some(s) = game.advance_level() {
            next.set(s);
        }
    }
}

/// DOM contract for the wasm shell bridge (main.rs `shell_bridge`): the exact
/// string published as `<html data-pelican-state="…">` and consumed by the
/// control check's gameplay lane. The Debug impl is NOT the contract — this
/// mapping is (pinned by test; CI predicates depend on the exact strings).
pub fn shell_state_name(s: &AppState) -> &'static str {
    match s {
        AppState::Title => "Title",
        AppState::Intro => "Intro",
        AppState::Playing => "Playing",
        AppState::Crashed => "Crashed",
        AppState::Cleared => "Cleared",
        AppState::AllClear => "AllClear",
    }
}

impl GameRes {
    /// advance to the next trial (or the final report); returns the next state
    pub fn advance_level(&mut self) -> Option<AppState> {
        if self.current + 1 >= self.levels.len() {
            if self.total > self.best {
                self.best = self.total;
            }
            Some(AppState::AllClear)
        } else {
            self.start_level(self.current + 1);
            Some(AppState::Intro)
        }
    }
}
