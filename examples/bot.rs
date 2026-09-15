//! bot — the scripted playtest (headless, deterministic; the Fixture Law:
//! a played game, not a health check). Ports the design reference's bot:
//! hold throttle, project the ballistic landing, lean to match its slope;
//! a second policy backflips over the canyon drops to prove flips are real.

use pelican_game::levels::levels;
use pelican_game::physics::{self, Bike, Input};

fn project_landing(level: &pelican_game::levels::Level, bike: &Bike) -> Option<(f32, f32, f32)> {
    let dt = 1.0f32 / 60.0;
    let mut px = bike.x;
    let mut py = bike.y;
    let mut pvx = bike.vx;
    let mut pvy = bike.vy;
    let mut flight = 0.0f32;
    for _ in 0..300 {
        pvy += physics::G * dt;
        px += pvx * dt;
        py += pvy * dt;
        flight += dt;
        if px < bike.x + 50.0 {
            continue;
        }
        if let Some((h, angle)) = level.ground_at(px) {
            if py > h - 26.0 {
                return Some((angle, flight, px));
            }
        }
    }
    None
}

fn cautious_input(level: &pelican_game::levels::Level, bike: &Bike) -> Input {
    let mut input = Input { throttle: true, brake: false, lean_left: false, lean_right: false };
    if bike.airborne_for > 0.1 {
        if let Some((angle, flight, _)) = project_landing(level, bike) {
            let err = physics::normalize_angle(angle - bike.a);
            let _ = flight;
            input.lean_right = err > 0.02;
            input.lean_left = err < -0.02;
        }
    }
    input
}

fn flip_input(level: &pelican_game::levels::Level, bike: &Bike, mode: &mut u8) -> Input {
    let mut input = Input { throttle: true, brake: false, lean_left: false, lean_right: false };
    if bike.airborne_for > 0.05 {
        let over_chasm = level.ground_at(bike.x + 60.0).is_none();
        if *mode == 0 && over_chasm && bike.airborne_for < 0.15 {
            *mode = 1;
        }
        if *mode == 1 && bike.rot_acc.abs() > std::f32::consts::PI * 1.72 {
            *mode = 2;
        }
        if *mode == 1 {
            input.lean_left = true;
        }
        if *mode == 2 || (!over_chasm && *mode != 1) {
            if let Some((angle, _, _)) = project_landing(level, bike) {
                let err = physics::normalize_angle(angle - bike.a);
                input.lean_right = err > 0.02;
                input.lean_left = err < -0.02;
            }
        }
    } else {
        *mode = 0;
    }
    input
}

fn run_level(level: &pelican_game::levels::Level, flips: bool) -> (bool, f32, usize, Option<&'static str>) {
    let mut bike = Bike::new(level);
    let mut events = Vec::new();
    let mut t = 0.0f32;
    let mut mode = 0u8;
    while t < 120.0 {
        let input = if flips { flip_input(level, &bike, &mut mode) } else { cautious_input(level, &bike) };
        physics::step(&mut bike, level, input, physics::SUBSTEP, t, &mut events);
        events.clear();
        t += physics::SUBSTEP;
        if bike.crashed || bike.finished {
            break;
        }
    }
    (bike.finished, t, bike.flips, bike.crash_reason)
}

fn main() {
    let lv = levels();
    let mut all_ok = true;
    println!("== cautious bot (completability) ==");
    for level in &lv {
        let (fin, t, _, reason) = run_level(level, false);
        println!(
            "L{} {}  time={:.1}s{}",
            level.id,
            if fin { "PASS" } else { "FAIL" },
            t,
            if fin { String::new() } else { format!("  ({:?} at end)", reason) }
        );
        if !fin {
            all_ok = false;
        }
    }
    println!("== flip bot on L4 (risk/reward is real) ==");
    let (fin, t, flips, reason) = run_level(&lv[3], true);
    println!(
        "L4 flipbot finished={} flips={} time={:.1}s {:?}",
        fin, flips, t, reason
    );
    if !(fin && flips > 0) {
        all_ok = false;
    }
    if all_ok {
        println!("ALL PASS");
    } else {
        println!("FAILURES PRESENT");
        std::process::exit(1);
    }
}
