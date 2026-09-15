//! world — terrain, sky, water, flag, scenery, rider, ragdoll, particles.
//!
//! Coordinate map: physics is y-DOWN; rendering is y-UP, so render_y = -phys_y.
//! Terrain meshes are built once per level (spans between gaps), textured
//! with the pixd tiles; the rider is the authored sprite pair (rider + 2
//! wheels as children, wheels spin, whole rotates with the chassis).

use crate::game::{AppState, CameraRes, GameRes, Lang, Particles, Rng};
use crate::physics;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::mesh::Mesh2d;

#[derive(Component)]
pub struct OnLevel; // despawned on every level (re)start

#[derive(Component)]
pub struct CloudMark {
    pub base_x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct SunMark;

#[derive(Component)]
pub struct RiderMark;

#[derive(Component)]
pub struct RiderBody;

#[derive(Component)]
pub struct RiderWheel {
    pub rear: bool,
}

#[derive(Component)]
pub struct RagdollDots {
    pub count: usize,
}

#[derive(Component)]
pub struct ParticleDots {
    pub count: usize,
}

#[derive(Resource)]
pub struct WorldAssets {
    pub rider: Handle<Image>,
    pub wheel: Handle<Image>,
    pub cloud: Handle<Image>,
    pub fixture: Handle<Image>,
    pub concept3d: Handle<Image>,
    pub weave: Handle<Image>,
    pub strata: Handle<Image>,
    pub finish: Handle<Image>,
    pub dot: Handle<Image>, // white circle, tinted per-use
    pub fonts: FontPack,
}

#[derive(Clone)]
pub struct FontPack {
    pub serif: Handle<Font>,
    pub serif_bold: Handle<Font>,
    pub serif_italic: Handle<Font>,
    pub mono: Handle<Font>,
}

pub fn load_assets(mut commands: Commands, assets: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    let wa = WorldAssets {
        rider: assets.load("sprites/rider.png"),
        wheel: assets.load("sprites/wheel.png"),
        cloud: assets.load("sprites/cloud.png"),
        fixture: assets.load("sprites/fixture-sprite.png"),
        concept3d: assets.load("sprites/concept-3d.png"),
        weave: assets.load("tiles/weave.png"),
        strata: assets.load("tiles/strata.png"),
        finish: assets.load("tiles/finish.png"),
        dot: circle_image(&mut images),
        fonts: FontPack {
            serif: assets.load("fonts/LiberationSerif-Regular.ttf"),
            serif_bold: assets.load("fonts/LiberationSerif-Bold.ttf"),
            serif_italic: assets.load("fonts/LiberationSerif-Italic.ttf"),
            mono: assets.load("fonts/LiberationMono-Regular.ttf"),
        },
    };
    commands.insert_resource(wa);
}

fn circle_image(images: &mut Assets<Image>) -> Handle<Image> {
    let size = 64usize;
    let mut data = Vec::with_capacity(size * size * 4);
    let c = 32.0;
    for y in 0..size {
        for x in 0..size {
            let d = ((x as f32 - c + 0.5).powi(2) + (y as f32 - c + 0.5).powi(2)).sqrt();
            let a = if d <= 26.0 {
                255
            } else if d <= 31.0 {
                ((31.0 - d) / 5.0 * 255.0) as u8
            } else {
                0
            };
            data.extend_from_slice(&[255, 255, 255, a]);
        }
    }
    let image = Image::new(
        bevy::render::render_resource::Extent3d { width: size as u32, height: size as u32, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    images.add(image)
}

/// build the level world (terrain, water, clouds, flag, start sign, rider)
pub fn spawn_level(mut commands: Commands, game: Res<GameRes>, wa: Res<WorldAssets>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    let level = game.level();

    // ---- terrain spans ----
    let mut span_start: Option<usize> = None;
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for i in 0..=level.cols.len() {
        let solid = i < level.cols.len() && level.cols[i].is_some();
        if solid && span_start.is_none() {
            span_start = Some(i);
        }
        if !solid {
            if let Some(a) = span_start.take() {
                spans.push((a, i.saturating_sub(1)));
            }
        }
    }
    let bottom = -(-level.cols.iter().flatten().fold(f32::MIN, |m, h| m.max(*h)) - 620.0);
    let ground_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.361, 0.369, 0.4)));
    let crust_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.11, 0.11, 0.13)));
    let weave_mat = materials.add(ColorMaterial::from(wa.weave.clone()));
    let strata_mat = materials.add(ColorMaterial::from(wa.strata.clone()));

    for (a, b) in spans {
        if b <= a {
            continue;
        }
        // ground fill
        let mut verts: Vec<[f32; 3]> = Vec::new();
        let mut idx: Vec<u32> = Vec::new();
        for i in a..=b {
            let h = level.cols[i].unwrap();
            verts.push([i as f32 * level.step, -h, 0.0]);
            verts.push([i as f32 * level.step, bottom, 0.0]);
        }
        for row in 0..(b - a) {
            let v = (row * 2) as u32;
            idx.extend_from_slice(&[v, v + 1, v + 2, v + 1, v + 3, v + 2]);
        }
        let mesh = mesh_from(verts, None, idx);
        commands.spawn((Mesh2d(meshes.add(mesh)), MeshMaterial2d(ground_mat.clone()), OnLevel));

        // strata texture inside the body (a second fill, textured, slightly inset)
        let mut verts: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut idx: Vec<u32> = Vec::new();
        let inset = 34.0;
        for i in a..=b {
            let h = level.cols[i].unwrap();
            let x = i as f32 * level.step;
            verts.push([x, -h - inset, 0.0]);
            verts.push([x, bottom, 0.0]);
            uvs.push([x / 128.0, 0.0]);
            uvs.push([x / 128.0, 3.0]);
        }
        for row in 0..(b - a) {
            let v = (row * 2) as u32;
            idx.extend_from_slice(&[v, v + 1, v + 2, v + 1, v + 3, v + 2]);
        }
        let mesh = mesh_from(verts, Some(uvs), idx);
        commands.spawn((Mesh2d(meshes.add(mesh)), MeshMaterial2d(strata_mat.clone()), OnLevel));

        // weave band (the fixture tile) along the surface + ink crust line
        let mut verts: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut idx: Vec<u32> = Vec::new();
        for i in a..=b {
            let h = level.cols[i].unwrap();
            let x = i as f32 * level.step;
            verts.push([x, -h, 0.0]);
            verts.push([x, -h - 26.0, 0.0]);
            uvs.push([x / 128.0, 0.0]);
            uvs.push([x / 128.0, 26.0 / 128.0]);
        }
        for row in 0..(b - a) {
            let v = (row * 2) as u32;
            idx.extend_from_slice(&[v, v + 1, v + 2, v + 1, v + 3, v + 2]);
        }
        let mesh = mesh_from(verts, Some(uvs), idx);
        commands.spawn((Mesh2d(meshes.add(mesh)), MeshMaterial2d(weave_mat.clone()), OnLevel, Transform::from_xyz(0.0, 0.0, 1.0)));

        let mut verts: Vec<[f32; 3]> = Vec::new();
        let mut idx: Vec<u32> = Vec::new();
        for i in a..=b {
            let h = level.cols[i].unwrap();
            let x = i as f32 * level.step;
            verts.push([x, -h + 1.5, 0.0]);
            verts.push([x, -h - 1.5, 0.0]);
        }
        for row in 0..(b - a) {
            let v = (row * 2) as u32;
            idx.extend_from_slice(&[v, v + 1, v + 2, v + 1, v + 3, v + 2]);
        }
        let mesh = mesh_from(verts, None, idx);
        commands.spawn((Mesh2d(meshes.add(mesh)), MeshMaterial2d(crust_mat.clone()), OnLevel, Transform::from_xyz(0.0, 0.0, 2.0)));
    }

    // ---- water ----
    if let Some(wy) = level.water_y {
        let width = level.cols.len() as f32 * level.step;
        let mut verts = vec![
            [0.0, -wy + 2.0, 0.0],
            [width, -wy + 2.0, 0.0],
            [0.0, bottom, 0.0],
            [width, bottom, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 1, 3, 2];
        let mesh = mesh_from(verts.drain(..).collect(), None, idx);
        let mat = materials.add(ColorMaterial::from_color(Color::srgba(0.35, 0.6, 0.76, 0.85)));
        commands.spawn((Mesh2d(meshes.add(mesh)), MeshMaterial2d(mat), OnLevel, Transform::from_xyz(0.0, 0.0, -5.0)));
    }

    // ---- clouds (parallax) ----
    let mut seed = 7u64 + level.id as u64 * 131;
    let mut rnd = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as f32) / (u32::MAX as f32)
    };
    for _ in 0..14 {
        let base_x = rnd() * 5200.0;
        let y = -(30.0 + rnd() * 130.0);
        let s = 0.6 + rnd() * 1.1;
        commands.spawn((
            Sprite { image: wa.cloud.clone(), custom_size: Some(Vec2::new(196.0 * s, 75.0 * s)), ..default() },
            Transform::from_xyz(base_x, y, -40.0),
            CloudMark { base_x, y },
            OnLevel,
        ));
    }

    // ---- sun ----
    commands.spawn((
        Sprite { image: wa.dot.clone(), color: Color::srgb(0.965, 0.675, 0.18), custom_size: Some(Vec2::splat(84.0)), ..default() },
        Transform::from_xyz(0.0, 0.0, -45.0),
        SunMark,
        OnLevel,
    ));

    // ---- finish flag ----
    let fy = {
        let idx = (level.finish_x / level.step) as usize;
        level.height_near(idx)
    };
    let pole = meshes.add(Rectangle::new(6.0, 150.0));
    let pole_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.11, 0.11, 0.13)));
    commands.spawn((
        Mesh2d(pole),
        MeshMaterial2d(pole_mat),
        Transform::from_xyz(level.finish_x, -fy + 75.0, 3.0),
        OnLevel,
    ));
    commands.spawn((
        Sprite { image: wa.dot.clone(), color: Color::srgb(0.965, 0.675, 0.18), custom_size: Some(Vec2::splat(14.0)), ..default() },
        Transform::from_xyz(level.finish_x, -fy + 154.0, 3.0),
        OnLevel,
    ));
    commands.spawn((
        Sprite { image: wa.finish.clone(), custom_size: Some(Vec2::new(84.0, 30.0)), ..default() },
        Transform::from_xyz(level.finish_x + 45.0, -fy + 133.0, 3.0),
        OnLevel,
    ));

    // ---- start sign ----
    let sy = {
        let idx = (level.start_x / level.step) as usize;
        level.height_near(idx)
    };
    let post = meshes.add(Rectangle::new(4.0, 40.0));
    let post_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.361, 0.369, 0.4)));
    commands.spawn((Mesh2d(post), MeshMaterial2d(post_mat), Transform::from_xyz(level.start_x - 40.0, -sy + 20.0, 3.0), OnLevel));
    let plate = meshes.add(Rectangle::new(30.0, 22.0));
    let plate_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.824, 0.227, 0.196)));
    commands.spawn((Mesh2d(plate), MeshMaterial2d(plate_mat), Transform::from_xyz(level.start_x - 40.0, -sy + 44.0, 3.0), OnLevel));

    // ---- rider (sprite + 2 wheels) ----
    let rider_size = Vec2::new(248.0 / 4.0, 231.0 / 4.0);
    let wheel_size = Vec2::splat(128.0 / 4.0);
    // sprites.json: rider origin (chassis point) at (109,174) of 248x231 →
    // center-relative offset (image y-down → render y-up)
    let off = Vec2::new(248.0 / 8.0 - 109.0 / 4.0, 174.0 / 4.0 - 231.0 / 8.0);
    commands
        .spawn((
            RiderMark,
            OnLevel,
            Transform::from_xyz(level.start_x, -level.start_y, 5.0),
        ))
        .with_children(|p| {
            p.spawn((RiderBody, Sprite { image: wa.rider.clone(), custom_size: Some(rider_size), ..default() }, Transform::from_xyz(off.x, off.y, 0.0)));
            p.spawn((
                RiderWheel { rear: true },
                Sprite { image: wa.wheel.clone(), custom_size: Some(wheel_size), ..default() },
                Transform::from_xyz(physics::REAR.x, -physics::REAR.y, 0.5),
            ));
            p.spawn((
                RiderWheel { rear: false },
                Sprite { image: wa.wheel.clone(), custom_size: Some(wheel_size), ..default() },
                Transform::from_xyz(physics::FRONT.x, -physics::FRONT.y, 0.5),
            ));
        });

    // ragdoll + particle dot pools
    commands.spawn((RagdollDots { count: 0 }, OnLevel, Visibility::Hidden, Transform::default()));
    commands.spawn((ParticleDots { count: 0 }, OnLevel, Visibility::Hidden, Transform::default()));
}

fn mesh_from(verts: Vec<[f32; 3]>, uvs: Option<Vec<[f32; 2]>>, idx: Vec<u32>) -> Mesh {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    if let Some(uv) = uvs {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    }
    mesh.insert_indices(Indices::U32(idx));
    mesh
}

pub fn despawn_level(mut commands: Commands, q: Query<Entity, With<OnLevel>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

/// smooth camera + parallax + sun
pub fn camera_system(
    cam_res: Res<CameraRes>,
    mut cam_q: Query<&mut Transform, With<Camera2d>>,
    mut clouds: Query<(&CloudMark, &mut Transform), (Without<SunMark>, Without<Camera2d>)>,
    mut sun: Query<(&SunMark, &mut Transform), (Without<CloudMark>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut ct) = cam_q.single_mut() else { return };
    if !cam_res.init {
        ct.translation.x = cam_res.x;
        ct.translation.y = -cam_res.y;
        return;
    }
    let k = (time.delta_secs() * 6.0).min(1.0);
    ct.translation.x += (cam_res.x - ct.translation.x) * k;
    ct.translation.y += (-cam_res.y - ct.translation.y) * (time.delta_secs() * 3.4).min(1.0);
    let cx = ct.translation.x;
    let cy = ct.translation.y;
    for (mark, mut t) in clouds.iter_mut() {
        let x = mark.base_x + cx * 0.75; // move at 25% camera speed
        let wrapped = ((x - cx + 780.0).rem_euclid(5200.0)) - 2600.0 + cx;
        t.translation.x = wrapped;
        t.translation.y = mark.y + cy * 0.08;
    }
    for (_, mut t) in sun.iter_mut() {
        t.translation.x = cx + 380.0;
        t.translation.y = cy + 180.0;
    }
}

/// rider transform sync (+ ragdoll / particle dots)
pub fn rider_sync(
    game: Res<GameRes>,
    state: Res<State<AppState>>,
    mut rider: Query<&mut Transform, (With<RiderMark>, Without<RiderWheel>)>,
    mut wheels: Query<(&RiderWheel, &mut Transform), Without<RiderMark>>,
    mut body_vis: Query<&mut Visibility, (With<RiderBody>, Without<RiderWheel>)>,
    mut dots: Query<(&RagdollDots, &mut Visibility, &Children, Entity), (Without<ParticleDots>, Without<RiderBody>)>,
    mut dot_children: Query<&mut Transform, (Without<RiderMark>, Without<RiderWheel>)>,
    wa: Res<WorldAssets>,
    mut commands: Commands,
    particles: Res<Particles>,
    pdots: Query<(&ParticleDots, Entity), Without<RagdollDots>>,
) {
    let Ok(mut rt) = rider.single_mut() else { return };
    let bike = &game.bike;
    rt.translation = Vec3::new(bike.x, -bike.y, 5.0);
    rt.rotation = Quat::from_rotation_z(-bike.a);
    for (wheel, mut t) in wheels.iter_mut() {
        let spin = if wheel.rear { bike.rear_spin } else { bike.front_spin };
        t.rotation = Quat::from_rotation_z(-spin);
    }
    let crashed = *state.get() == AppState::Crashed;
    if let Ok(mut v) = body_vis.single_mut() {
        if crashed {
            *v = Visibility::Hidden;
        }
    }

    // ragdoll dots: rebuild on count change, else sync transforms
    if let Some(rag) = game.ragdoll.as_ref() {
        for (_dots, mut vis, children, e) in dots.iter_mut() {
            *vis = Visibility::Visible;
            if children.len() != rag.len() {
                commands.entity(e).despawn(); // cascades to children
                let mut spawned = commands.spawn((
                    RagdollDots { count: rag.len() },
                    OnLevel,
                    Transform::default(),
                ));
                for p in rag {
                    spawned.with_child((
                        Sprite { image: wa.dot.clone(), custom_size: Some(Vec2::splat(p.r * 2.2)), ..default() },
                        Transform::from_xyz(p.x, -p.y, 6.0),
                    ));
                }
            } else {
                for (c, p) in children.iter().zip(rag.iter()) {
                    if let Ok(mut t) = dot_children.get_mut(c) {
                        t.translation = Vec3::new(p.x, -p.y, 6.0);
                    }
                }
            }
        }
    } else {
        for (_dots, mut vis, _children, _e) in dots.iter_mut() {
            *vis = Visibility::Hidden;
        }
    }

    // particle dots: rebuild on count change (despawn cascades in 0.19)
    for (pm, e) in pdots.iter() {
        let n = particles.items.len();
        if pm.count != n {
            commands.entity(e).despawn();
            let mut spawned = commands.spawn((ParticleDots { count: n }, OnLevel, Transform::default()));
            for p in &particles.items {
                spawned.with_child((
                    Sprite { image: wa.dot.clone(), color: p.color, custom_size: Some(Vec2::splat(p.r * 2.0)), ..default() },
                    Transform::from_xyz(p.x, -p.y, 6.0),
                ));
            }
        }
    }
}

/// ignore: keep Lang import used (language flips re-render UI only)
pub fn _lang_marker(_: Res<Lang>, _: ResMut<Rng>) {}
