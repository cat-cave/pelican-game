//! physics — trials-style bike physics (ported 1:1 from the bot-validated
//! design reference: same constants, same model, fixed timestep, no
//! randomness). y grows DOWN; angle is clockwise-on-screen in that frame.
//!
//! Wheel contacts cancel approach velocity along the terrain normal at the
//! deepest wheel; drive/brake act along the tangent; a damped servo holds
//! the chassis level with the terrain when grounded ("wing-assisted
//! balance"); lean in air spins the bike (flips). A head point (the beak)
//! is the crash sensor.

use crate::levels::Level;
use bevy::math::Vec2;

pub const SUBSTEP: f32 = 1.0 / 120.0;

pub const G: f32 = 1400.0;
pub const WHEEL_R: f32 = 14.0;
pub const REAR: Vec2 = Vec2::new(-25.0, 12.0);
pub const FRONT: Vec2 = Vec2::new(25.0, 12.0);
pub const HEAD: Vec2 = Vec2::new(10.0, -30.0);
pub const DRIVE_F: f32 = 950.0;
pub const MAX_SPEED: f32 = 380.0;
pub const BRAKE_K: f32 = 9.0;
pub const ROLL_DRAG: f32 = 1.2;
pub const AIR_DRAG: f32 = 0.05;
pub const RESTITUTION: f32 = 0.12;
pub const GROUND_SPRING: f32 = 120.0;
pub const SERVO_KP: f32 = 90.0;
pub const SERVO_KD: f32 = 18.0;
pub const LEAN_ASSIST: f32 = 0.32;
pub const AIR_TORQUE: f32 = 38.0;
pub const ANG_DRAG_AIR: f32 = 0.5;
pub const CRASH_ANGLE: f32 = 1.75;

#[derive(Clone, Debug)]
pub struct Bike {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub a: f32,
    pub w: f32,
    pub rear_spin: f32,
    pub front_spin: f32,
    pub rear_omega: f32,
    pub front_omega: f32,
    pub grounded_rear: bool,
    pub grounded_front: bool,
    pub airborne_for: f32,
    pub rot_acc: f32,
    pub flips: usize,
    pub finished: bool,
    pub crashed: bool,
    pub crash_reason: Option<&'static str>,
}

impl Bike {
    pub fn new(level: &Level) -> Self {
        Self {
            x: level.start_x,
            y: level.start_y,
            vx: 0.0,
            vy: 0.0,
            a: 0.0,
            w: 0.0,
            rear_spin: 0.0,
            front_spin: 0.0,
            rear_omega: 0.0,
            front_omega: 0.0,
            grounded_rear: false,
            grounded_front: false,
            airborne_for: 99.0,
            rot_acc: 0.0,
            flips: 0,
            finished: false,
            crashed: false,
            crash_reason: None,
        }
    }

    pub fn world_point(&self, local: Vec2) -> Vec2 {
        let (s, c) = self.a.sin_cos();
        Vec2::new(
            self.x + local.x * c - local.y * s,
            self.y + local.x * s + local.y * c,
        )
    }
}

pub fn normalize_angle(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub throttle: bool,
    pub brake: bool,
    pub lean_left: bool,
    pub lean_right: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ev {
    Land { impact: f32 },
    Flip { dir: f32, count: usize },
    Crash { reason: &'static str },
    Splash,
    Finish,
}

pub fn step(bike: &mut Bike, level: &Level, input: Input, dt: f32, now: f32, events: &mut Vec<Ev>) {
    if bike.crashed || bike.finished {
        return;
    }
    let lean = match (input.lean_left, input.lean_right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };

    bike.vy += G * dt;

    let rear_w = bike.world_point(REAR);
    let front_w = bike.world_point(FRONT);
    let g_r = level.ground_at(rear_w.x);
    let g_f = level.ground_at(front_w.x);
    let pen_r = g_r.map(|(h, _)| rear_w.y + WHEEL_R - h);
    let pen_f = g_f.map(|(h, _)| front_w.y + WHEEL_R - h);
    bike.grounded_rear = pen_r.map(|p| p > 0.0).unwrap_or(false);
    bike.grounded_front = pen_f.map(|p| p > 0.0).unwrap_or(false);
    let grounded = bike.grounded_rear || bike.grounded_front;

    if grounded {
        let (gh, gangle) = if pen_f.unwrap_or(-1e9) > pen_r.unwrap_or(-1e9) {
            g_f.or(g_r).unwrap()
        } else {
            g_r.or(g_f).unwrap()
        };
        let _ = gh;
        // slope at the chassis center governs the contact frame (stable on
        // convex faces); fall back to the deepest wheel's slope
        let g_center = level.ground_at(bike.x).unwrap_or((gh, gangle));
        let angle = g_center.1;
        let (s, c) = angle.sin_cos();
        let n = Vec2::new(s, -c); // up-normal
        let t = Vec2::new(c, s);  // +x tangent
        let pen = pen_r.unwrap_or(-1e9).max(pen_f.unwrap_or(-1e9));

        if pen > 1.8 {
            let corr = (pen - 1.5).min(8.0) * 0.18;
            bike.x += n.x * corr;
            bike.y += n.y * corr;
        }

        let vn = bike.vx * n.x + bike.vy * n.y;
        if vn < 0.0 {
            let bounce = if vn < -260.0 && bike.airborne_for > 0.18 { RESTITUTION } else { 0.0 };
            if vn < -300.0 && bike.airborne_for > 0.25 {
                events.push(Ev::Land { impact: -vn });
            }
            bike.vx -= n.x * vn * (1.0 + bounce);
            bike.vy -= n.y * vn * (1.0 + bounce);
        }
        if pen > 0.2 {
            let f = GROUND_SPRING * pen.min(8.0) * dt;
            bike.vx += n.x * f;
            bike.vy += n.y * f;
        }

        let vt = bike.vx * t.x + bike.vy * t.y;
        if (bike.grounded_rear || bike.grounded_front) && input.throttle && !input.brake && vt < MAX_SPEED {
            bike.vx += t.x * DRIVE_F * dt;
            bike.vy += t.y * DRIVE_F * dt;
        }
        let drag = if input.brake {
            (BRAKE_K * dt).min(1.0)
        } else {
            (ROLL_DRAG * 2.0 * dt).min(1.0)
        };
        bike.vx -= t.x * vt * drag;
        bike.vy -= t.y * vt * drag;

        if bike.airborne_for > 0.25 {
            if bike.rot_acc.abs() >= std::f32::consts::PI * 1.8 {
                let n = (bike.rot_acc.abs() / (std::f32::consts::PI * 1.8)).floor() as usize;
                bike.flips += n;
                events.push(Ev::Flip { dir: bike.rot_acc.signum(), count: bike.flips });
            }
            bike.rot_acc = 0.0;
        }
        bike.airborne_for = 0.0;

        let target = angle + lean * LEAN_ASSIST;
        let err = normalize_angle(target - bike.a);
        bike.w += (err * SERVO_KP - bike.w * SERVO_KD) * dt;

        if normalize_angle(bike.a - angle).abs() > CRASH_ANGLE {
            crash(bike, "overrotation", now, events);
            return;
        }
    } else {
        bike.airborne_for += dt;
        bike.w += lean * AIR_TORQUE * dt;
        bike.w -= bike.w * ANG_DRAG_AIR * dt;
        bike.rot_acc += bike.w * dt;
    }

    bike.vx -= bike.vx * AIR_DRAG * dt;
    bike.vy -= bike.vy * AIR_DRAG * dt;
    bike.x += bike.vx * dt;
    bike.y += bike.vy * dt;
    bike.a += bike.w * dt;

    let grounded_any = bike.grounded_rear || bike.grounded_front;
    if grounded_any {
        bike.rear_omega = bike.vx / WHEEL_R;
        bike.front_omega = bike.vx / WHEEL_R;
    } else {
        if input.throttle && !input.brake {
            bike.rear_omega = (bike.rear_omega + 40.0 * dt).min(26.0);
        } else {
            bike.rear_omega = (bike.rear_omega - 8.0 * dt).max(0.0);
        }
        bike.front_omega *= 1.0 - 0.6 * dt;
    }
    bike.rear_spin += bike.rear_omega * dt;
    bike.front_spin += bike.front_omega * dt;

    let head = bike.world_point(HEAD);
    if let Some((hh, _)) = level.ground_at(head.x) {
        if head.y > hh - 2.0 {
            crash(bike, "beak", now, events);
            return;
        }
    }
    if let Some(wy) = level.water_y {
        if bike.y > wy {
            events.push(Ev::Splash);
            crash(bike, "water", now, events);
            return;
        }
    }
    if bike.y > 2600.0 {
        crash(bike, "fell", now, events);
        return;
    }
    if bike.x >= level.finish_x {
        bike.finished = true;
        events.push(Ev::Finish);
    }
}

fn crash(bike: &mut Bike, reason: &'static str, _now: f32, events: &mut Vec<Ev>) {
    bike.crashed = true;
    bike.crash_reason = Some(reason);
    events.push(Ev::Crash { reason });
}

/// ragdoll — verlet particles for the crash aftermath (the pelican, unseated)
#[derive(Clone, Debug)]
pub struct RagPart {
    pub x: f32,
    pub y: f32,
    pub px: f32,
    pub py: f32,
    pub r: f32,
    pub kind: u8, // 0 body 1 tail 2 head 3 beak 4 wing 5 foot
}

pub fn make_ragdoll(bike: &Bike) -> Vec<RagPart> {
    const LOCALS: [(Vec2, f32, u8); 6] = [
        (Vec2::new(2.0, -12.0), 9.0, 0),
        (Vec2::new(-9.0, -8.0), 6.0, 1),
        (Vec2::new(12.0, -22.0), 6.0, 2),
        (Vec2::new(22.0, -19.0), 4.0, 3),
        (Vec2::new(-2.0, -21.0), 5.0, 4),
        (Vec2::new(8.0, 0.0), 4.0, 5),
    ];
    LOCALS
        .iter()
        .map(|(l, r, k)| {
            let wp = bike.world_point(*l);
            RagPart {
                x: wp.x,
                y: wp.y,
                px: wp.x - bike.vx * SUBSTEP,
                py: wp.y - bike.vy * SUBSTEP,
                r: *r,
                kind: *k,
            }
        })
        .collect()
}

pub fn step_ragdoll(parts: &mut [RagPart], level: &Level, dt: f32) {
    const LINKS: [(usize, usize, f32); 6] = [
        (0, 1, 11.0),
        (0, 2, 14.0),
        (2, 3, 11.0),
        (0, 4, 10.0),
        (0, 5, 12.0),
        (2, 4, 12.0),
    ];
    for p in parts.iter_mut() {
        let vx = (p.x - p.px) * 0.995;
        let vy = (p.y - p.py) * 0.995;
        p.px = p.x;
        p.py = p.y;
        p.x += vx;
        p.y += vy + G * dt * dt;
    }
    for _ in 0..3 {
        for &(i, j, len) in LINKS.iter() {
            let (ax, ay) = (parts[i].x, parts[i].y);
            let (bx, by) = (parts[j].x, parts[j].y);
            let d = Vec2::new(bx - ax, by - ay);
            let dist = d.length().max(1e-6);
            let diff = ((dist - len) / dist) * 0.5;
            parts[i].x += d.x * diff;
            parts[i].y += d.y * diff;
            parts[j].x -= d.x * diff;
            parts[j].y -= d.y * diff;
        }
        for p in parts.iter_mut() {
            if let Some((gh, _)) = level.ground_at(p.x) {
                if p.y > gh - p.r {
                    p.y = gh - p.r;
                    p.px += (p.x - p.px) * 0.42;
                }
            }
            if let Some(wy) = level.water_y {
                if p.y > wy {
                    p.y = wy;
                }
            }
        }
    }
}
