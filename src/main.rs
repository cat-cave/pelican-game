//! pelican-game — Pelicans Riding Bicycles: A Feasibility Study — The Field
//! Trials. The catcave composite trial: a Bevy game (native + wasm32) whose
//! content — sprite, tiles, tune, deck language, localized strings — comes
//! from the estate's planes.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use pelican_game::game;
use pelican_game::ui;
use pelican_game::world;

/// headless/CI verification mode (see tools/snap.sh): capture a screenshot at
/// T seconds and exit — renders through whatever backend the env provides
/// (llvmpipe under Xvfb = the G-B1 software-render path).
#[derive(Resource)]
struct Snap {
    path: String,
    at: f32,
    t: f32,
    shot: bool,
}

fn parse_snap() -> Option<Snap> {
    let args: Vec<String> = std::env::args().collect();
    let i = args.iter().position(|a| a == "--snap")?;
    Some(Snap { path: args.get(i + 1)?.clone(), at: 6.0, t: 0.0, shot: false })
}
#[derive(Resource)]
struct SimTest {
    start: std::time::Instant,
    logged: usize,
    throttle_held: bool,
}

fn simtest_system(
    mut game: ResMut<game::GameRes>,
    mut input: ResMut<game::InputRes>,
    mut test: ResMut<SimTest>,
    mut exit: MessageWriter<AppExit>,
    state: Res<State<game::AppState>>,
    mut next: ResMut<NextState<game::AppState>>,
) {
    if !test.throttle_held {
        test.throttle_held = true;
        game.start_level(0);
        next.set(game::AppState::Intro);
    }
    input.input.throttle = true; // cautious bot
    let t = test.start.elapsed().as_secs_f32();
    let sec = t as usize;
    if sec > test.logged {
        test.logged = sec;
        println!(
            "[simtest {:2}s] state={:?} x={:.0} vx={:.0} flips={} score={} crash={:?}",
            sec, state.get(), game.bike.x, game.bike.vx, game.flips, game.total, game.bike.crash_reason
        );
    }
    if t > 40.0 {
        println!("[simtest] done: finished={} total={}", game.bike.finished, game.total);
        exit.write(AppExit::Success);
    }
}

/// asset root for AssetPlugin::file_path.
/// wasm: Bevy's HttpWasmAssetReader fetches "<file_path>/<asset>" as a URL
/// relative to the page origin, so it must be the deployed `assets/` dir.
/// native: absolute path — robust under CARGO_TARGET_DIR relocation.
fn asset_root() -> String {
    if cfg!(target_family = "wasm") {
        "assets".to_string()
    } else {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string()
    }
}

fn main() {
    #[cfg(target_family = "wasm")]
    console_error_panic_hook::set_once();
    #[cfg(target_family = "wasm")]
    web_sys::console::log_1(&"pelican-game: main() entered".into());
    let snap = parse_snap();
    let simtest = std::env::args().any(|a| a == "--simtest");
    let (w, h): (u32, u32) = if snap.is_some() { (640, 360) } else { (960, 540) };
    let default_plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pelicans Riding Bicycles: A Feasibility Study".into(),
                resolution: bevy::window::WindowResolution::new(w, h),
                canvas: Some("#gamecanvas".into()),
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest())
        .set(AssetPlugin {
            mode: bevy::asset::AssetMode::Unprocessed,
            // wasm: HttpWasmAssetReader fetches "<file_path>/<asset>" relative
            // to the page origin — it MUST be a relative URL, not a build-host
            // absolute path (the old CARGO_MANIFEST_DIR path 404'd every asset
            // and left the game with no sprites and no fonts).
            file_path: asset_root(),
            ..default()
        });

    let mut app = App::new();
    if simtest {
        // render-free app-loop trial: MinimalPlugins at 60Hz, game systems only
        app.add_plugins((
            bevy::MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            )),
            bevy::asset::AssetPlugin {
                file_path: asset_root(),
                mode: bevy::asset::AssetMode::Unprocessed,
                ..default()
            },
            bevy::state::app::StatesPlugin,
        ));
        app.insert_resource(SimTest { start: std::time::Instant::now(), logged: 0, throttle_held: false })
            .init_state::<game::AppState>()
            .init_resource::<game::GameRes>()
            .init_resource::<game::InputRes>()
            .init_resource::<game::CameraRes>()
            .init_resource::<game::Particles>()
            .insert_resource(game::Rng(0xC0FFEE))
            .insert_resource(game::Lang::En)
            .insert_resource(ui::make_texts())
            .add_systems(
                Update,
                (game::sim_system, simtest_system).chain(),
            );
        app.run();
        return;
    }
    app.add_plugins(default_plugins);
    // software-render friendly (headless CI / llvmpipe boxes)
    app.insert_resource(ClearColor(bevy::color::Color::srgb(0.588, 0.784, 0.882)));

    app.init_state::<game::AppState>()
        .init_resource::<game::GameRes>()
        .init_resource::<game::InputRes>()
        .init_resource::<game::CameraRes>()
        .init_resource::<game::Particles>()
        .insert_resource(game::Rng(0xC0FFEE))
        .insert_resource(game::Lang::En)
        .insert_resource(ui::make_texts())
        .add_systems(Startup, (world::load_assets, ui::build_ui, setup_camera, signal_ready).chain());

    // level lifecycle
    app.add_systems(OnEnter(game::AppState::Intro), (world::despawn_level, world::spawn_level).chain());

    // simulation
    app.add_systems(
        Update,
        (
            game::keyboard_system,
            game::sim_system,
            world::camera_system,
            world::rider_sync,
            ui::hud_update,
            ui::screen_visibility,
            ui::ui_interact,
        )
            .chain(),
    );

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera, Transform::from_xyz(140.0, -340.0, 1000.0)));
}

fn snap_system(
    mut commands: Commands,
    mut snap: ResMut<Snap>,
    time: Res<Time>,
    mut exit: MessageWriter<AppExit>,
    mut game: ResMut<game::GameRes>,
    mut next: ResMut<NextState<game::AppState>>,
) {
    snap.t += time.delta_secs();
    if snap.t < 0.1 {
        // drop straight into trial 1 for an in-game shot
        game.start_level(0);
        next.set(game::AppState::Intro);
    }
    if !snap.shot && snap.t >= snap.at {
        snap.shot = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(snap.path.clone()));
    }
    if snap.t >= snap.at + 1.2 {
        exit.write(AppExit::Success);
    }
}

/// tell the shell the game booted (loader hides; headless tests can poll it)
fn signal_ready() {
    #[cfg(target_family = "wasm")]
    {
        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                doc.set_title("pelican-ready");
            }
        }
    }
}
