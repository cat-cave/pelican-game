//! levels — the five field trials (ported 1:1 from the bot-validated design
//! reference; same segment DSL, same numbers, deterministic compile).
//!
//! Terrain is a heightmap sampled every STEP px. y grows DOWN (physics
//! space); h(x) is the surface y. Gaps are spans with no ground; water_y
//! makes falling below it a splash crash.

pub const STEP: f32 = 6.0;

#[derive(Clone, Debug)]
pub enum Seg {
    Flat(f32),
    Hill { len: f32, h: f32 },
    Valley { len: f32, h: f32 },
    Slope { len: f32, dy: f32 },
    Gap(f32),
    Ramp { len: f32, h: f32 },
    Kicker { len: f32, h: f32 },
    Cliff(f32),
}

pub struct Level {
    pub id: usize,
    pub name_key: &'static str,
    pub cols: Vec<Option<f32>>,
    pub gaps: Vec<(f32, f32)>,
    pub step: f32,
    pub start_x: f32,
    pub start_y: f32,
    pub finish_x: f32,
    pub water_y: Option<f32>,
    pub par: f32,
}

impl Level {
    /// {h, angle} of the surface at world x, or None inside a gap.
    pub fn ground_at(&self, x: f32) -> Option<(f32, f32)> {
        let i = (x / self.step).floor() as i64;
        if i < 0 || i as usize + 1 >= self.cols.len() {
            return None;
        }
        let c0 = self.cols[i as usize]?;
        let c1 = self.cols[i as usize + 1]?;
        let p = (x / self.step) - i as f32;
        let h = c0 + (c1 - c0) * p;
        let angle = (c1 - c0).atan2(self.step);
        Some((h, angle))
    }

    pub fn height_near(&self, i: usize) -> f32 {
        if let Some(v) = self.cols.get(i).copied().flatten() {
            return v;
        }
        for d in 1..self.cols.len() {
            if i > d {
                if let Some(v) = self.cols[i - d] {
                    return v;
                }
            }
            if i + d < self.cols.len() {
                if let Some(v) = self.cols[i + d] {
                    return v;
                }
            }
        }
        400.0
    }
}

pub fn compile(id: usize, name_key: &'static str, segments: &[Seg], start_h: f32, par: f32, water_y: Option<f32>) -> Level {
    let mut cols: Vec<Option<f32>> = Vec::new();
    let mut gaps: Vec<(f32, f32)> = Vec::new();
    let mut h = start_h;

    let mut push = |cols: &mut Vec<Option<f32>>, v: f32| cols.push(Some(v));

    // implicit start runway
    for _ in 0..(420.0 / STEP).round() as usize {
        push(&mut cols, h);
    }
    for seg in segments {
        let n = match seg {
            Seg::Flat(len)
            | Seg::Hill { len, .. }
            | Seg::Valley { len, .. }
            | Seg::Slope { len, .. }
            | Seg::Gap(len)
            | Seg::Ramp { len, .. }
            | Seg::Kicker { len, .. } => (*len / STEP).round() as usize,
            Seg::Cliff(_) => 1,
        };
        match *seg {
            Seg::Flat(_) => {
                for _ in 0..n {
                    push(&mut cols, h);
                }
            }
            Seg::Hill { h: hh, .. } => {
                for i in 0..n {
                    let p = i as f32 / n as f32;
                    push(&mut cols, h - hh * (0.5 - 0.5 * (std::f32::consts::TAU * p).cos()));
                }
            }
            Seg::Valley { h: hh, .. } => {
                for i in 0..n {
                    let p = i as f32 / n as f32;
                    push(&mut cols, h + hh * (0.5 - 0.5 * (std::f32::consts::TAU * p).cos()));
                }
            }
            Seg::Slope { dy, .. } => {
                for i in 0..n {
                    let p = i as f32 / n as f32;
                    push(&mut cols, h + dy * p);
                }
                h += dy;
            }
            Seg::Gap(_) => {
                let start_col = cols.len();
                for _ in 0..n {
                    cols.push(None);
                }
                gaps.push((start_col as f32 * STEP, (start_col + n) as f32 * STEP));
            }
            Seg::Ramp { h: hh, .. } => {
                for i in 0..n {
                    let p = i as f32 / n as f32;
                    push(&mut cols, h - hh * (0.5 - 0.5 * (std::f32::consts::PI * p).cos()));
                }
                h -= hh;
            }
            Seg::Kicker { h: hh, .. } => {
                for i in 0..n {
                    let p = i as f32 / n as f32;
                    push(&mut cols, h - hh * p * p);
                }
                h -= hh;
            }
            Seg::Cliff(dy) => {
                h += dy;
                push(&mut cols, h);
            }
        }
    }
    for _ in 0..(420.0 / STEP).round() as usize {
        push(&mut cols, h);
    }

    let start_x = 140.0;
    let start_i = (start_x / STEP).round() as usize;
    let start_y = Level::height_near(&Level {
        id,
        name_key,
        cols: cols.clone(),
        gaps: gaps.clone(),
        step: STEP,
        start_x,
        start_y: 0.0,
        finish_x: 0.0,
        water_y: None,
        par,
    }, start_i) - 54.0;
    let finish_x = cols.len() as f32 * STEP - 360.0;

    Level { id, name_key, cols, gaps, step: STEP, start_x, start_y, finish_x, water_y, par }
}

pub fn levels() -> Vec<Level> {
    vec![
        // 1 — The Seaside Promenade
        compile(1, "level.1", &[
            Seg::Flat(400.0),
            Seg::Hill { len: 320.0, h: 34.0 },
            Seg::Flat(200.0),
            Seg::Hill { len: 340.0, h: 40.0 },
            Seg::Valley { len: 300.0, h: 36.0 },
            Seg::Flat(220.0),
            Seg::Hill { len: 380.0, h: 44.0 },
            Seg::Flat(240.0),
            Seg::Valley { len: 260.0, h: 28.0 },
            Seg::Slope { len: 280.0, dy: -36.0 },
            Seg::Flat(260.0),
            Seg::Hill { len: 360.0, h: 42.0 },
            Seg::Flat(260.0),
        ], 420.0, 17.0, None),
        // 2 — Downhill, With Wind
        compile(2, "level.2", &[
            Seg::Slope { len: 220.0, dy: -80.0 },
            Seg::Hill { len: 320.0, h: 34.0 },
            Seg::Slope { len: 300.0, dy: -110.0 },
            Seg::Flat(160.0),
            Seg::Slope { len: 300.0, dy: -120.0 },
            Seg::Hill { len: 340.0, h: 36.0 },
            Seg::Slope { len: 260.0, dy: -90.0 },
            Seg::Valley { len: 300.0, h: 38.0 },
            Seg::Slope { len: 300.0, dy: -100.0 },
            Seg::Flat(200.0),
            Seg::Hill { len: 320.0, h: 38.0 },
            Seg::Slope { len: 240.0, dy: -60.0 },
            Seg::Flat(280.0),
        ], 560.0, 16.0, None),
        // 3 — The Jetty Gaps
        compile(3, "level.3", &[
            Seg::Flat(420.0),
            Seg::Gap(88.0),
            Seg::Flat(260.0),
            Seg::Hill { len: 260.0, h: 24.0 },
            Seg::Flat(120.0),
            Seg::Gap(96.0),
            Seg::Flat(240.0),
            Seg::Slope { len: 160.0, dy: -20.0 },
            Seg::Flat(100.0),
            Seg::Gap(104.0),
            Seg::Flat(260.0),
            Seg::Hill { len: 280.0, h: 30.0 },
            Seg::Flat(120.0),
            Seg::Gap(100.0),
            Seg::Flat(240.0),
            Seg::Gap(92.0),
            Seg::Flat(300.0),
        ], 400.0, 19.0, Some(470.0)),
        // 4 — Klezmer Canyon
        compile(4, "level.4", &[
            Seg::Flat(420.0),
            Seg::Slope { len: 200.0, dy: -60.0 },
            Seg::Flat(220.0),
            Seg::Kicker { len: 130.0, h: 30.0 },
            Seg::Gap(110.0),
            Seg::Cliff(200.0),
            Seg::Flat(300.0),
            Seg::Kicker { len: 140.0, h: 34.0 },
            Seg::Gap(125.0),
            Seg::Cliff(230.0),
            Seg::Flat(300.0),
            Seg::Kicker { len: 150.0, h: 38.0 },
            Seg::Gap(135.0),
            Seg::Cliff(260.0),
            Seg::Flat(340.0),
        ], 560.0, 21.0, Some(1290.0)),
        // 5 — The Feasibility Summit
        compile(5, "level.5", &[
            Seg::Flat(340.0),
            Seg::Slope { len: 210.0, dy: -80.0 },
            Seg::Flat(140.0),
            Seg::Slope { len: 190.0, dy: -74.0 },
            Seg::Valley { len: 260.0, h: 34.0 },
            Seg::Slope { len: 230.0, dy: -88.0 },
            Seg::Flat(200.0),
            Seg::Gap(96.0),
            Seg::Cliff(10.0),
            Seg::Slope { len: 120.0, dy: -26.0 },
            Seg::Flat(220.0),
            Seg::Slope { len: 260.0, dy: -102.0 },
            Seg::Hill { len: 280.0, h: 36.0 },
            Seg::Slope { len: 200.0, dy: -78.0 },
            Seg::Flat(200.0),
            Seg::Kicker { len: 150.0, h: 32.0 },
            Seg::Gap(130.0),
            Seg::Cliff(210.0),
            Seg::Flat(340.0),
        ], 700.0, 24.0, Some(1150.0)),
    ]
}
