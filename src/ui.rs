//! ui — title/end screens in the design plane's deck language, the HUD, and
//! the embedded locale bundles (the itotori leg). Paper/ink/accent palette,
//! serif headings, mono kickers.

use crate::game::{AppState, BtnAction, GameRes, Lang};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use crate::world::WorldAssets;
use bevy::prelude::*;

// deck palette (design-pelican-deck.html)
pub const INK: Color = Color::srgb(0.102, 0.122, 0.18);
pub const PAPER: Color = Color::srgb(0.965, 0.953, 0.925);
pub const ACCENT: Color = Color::srgb(0.91, 0.384, 0.173);
pub const MUTED: Color = Color::srgb(0.42, 0.439, 0.502);

pub const EN: &str = include_str!("../assets/locales/en.json");
/// ja.json is produced by the machine lane (tools/localize-machine.py — live
/// model calls through kumiki). If the lane is down at build time we ship
/// EN-only honestly: HAS_JA=false hides the toggle instead of faking a bundle.
pub const HAS_JA: bool = cfg!(feature = "locale-ja");

pub type StringMap = std::collections::HashMap<String, String>;

/// marker: a Text node whose content is locale-driven
#[derive(Component)]
pub struct L10N_MARKER(pub String);

#[derive(Resource)]
pub struct Texts {
    pub en: StringMap,
    pub ja: Option<StringMap>,
}

impl Texts {
    fn parse(json: &str) -> StringMap {
        serde_json::from_str(json).unwrap_or_default()
    }
    pub fn t(&self, lang: Lang, key: &str) -> String {
        if lang == Lang::Ja {
            if let Some(ja) = self.ja.as_ref() {
                if let Some(v) = ja.get(key) {
                    return v.clone();
                }
            }
        }
        self.en.get(key).cloned().unwrap_or_else(|| key.to_string())
    }
}

pub fn make_texts() -> Texts {
    #[cfg(feature = "locale-ja")]
    {
        Texts {
            en: Texts::parse(EN),
            ja: Some(Texts::parse(include_str!("../assets/locales/ja.json"))),
        }
    }
    #[cfg(not(feature = "locale-ja"))]
    {
        Texts { en: Texts::parse(EN), ja: None }
    }
}

// ---- markers ----
#[derive(Component)]
pub struct HudPlate;

#[derive(Component)]
pub struct HudTime;

#[derive(Component)]
pub struct HudScore;

#[derive(Component)]
pub struct HudLevelName;

#[derive(Component)]
pub struct ClearedScores;

#[derive(Component)]
pub struct AllClearTotal;

#[derive(Component)]
pub struct BestLine;

macro_rules! screens {
    ($($m:ident),* $(,)?) => { $(
        #[derive(Component)]
        pub struct $m;
    )* };
}
screens!(ScreenTitle, ScreenAbout, ScreenCrashed, ScreenCleared, ScreenAllClear);

/// which screens are visible per state
pub fn screen_visibility(
    state: Res<State<AppState>>,
    mut title: Query<&mut Node, With<ScreenTitle>>,
    mut crashed: Query<&mut Node, (With<ScreenCrashed>, Without<ScreenTitle>)>,
    mut cleared: Query<&mut Node, (With<ScreenCleared>, Without<ScreenTitle>, Without<ScreenCrashed>)>,
    mut allclear: Query<&mut Node, (With<ScreenAllClear>, Without<ScreenTitle>, Without<ScreenCrashed>, Without<ScreenCleared>)>,
) {
    let s = state.get();
    for mut n in title.iter_mut() {
        n.display = if *s == AppState::Title { Display::Flex } else { Display::None };
    }
    for mut n in crashed.iter_mut() {
        n.display = if *s == AppState::Crashed { Display::Flex } else { Display::None };
    }
    for mut n in cleared.iter_mut() {
        n.display = if *s == AppState::Cleared { Display::Flex } else { Display::None };
    }
    for mut n in allclear.iter_mut() {
        n.display = if *s == AppState::AllClear { Display::Flex } else { Display::None };
    }
}

// ---- construction helpers ----
fn card_with(sr: &mut ChildSpawnerCommands, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    sr.spawn((
        Node {
            width: Val::Px(720.0),
            max_height: Val::Percent(88.0),
            overflow: Overflow::scroll_y(),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(34.0)),
            ..default()
        },
        BackgroundColor(PAPER),
    ))
    .with_children(|card| f(card));
}


fn text_node(
    parent: &mut ChildSpawnerCommands,
    wa: &WorldAssets,
    key: &str,
    font: Handle<Font>,
    size: f32,
    color: Color,
    texts: &Texts,
    lang: Lang,
) {
    parent.spawn((
        Text::new(texts.t(lang, key)),
        TextFont { font: font.into(), font_size: size.into(), ..default() },
        TextColor(color),
        L10N_MARKER(key.to_string()),
    ));
}

fn button(parent: &mut ChildSpawnerCommands, wa: &WorldAssets, key: &str, action: BtnAction, texts: &Texts, lang: Lang, accent: bool) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(22.0), Val::Px(10.0)),
                margin: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(if accent { ACCENT } else { PAPER }),
            BorderColor::all(if accent { ACCENT } else { INK }),
            action,
        ))
        .with_children(|b| {
            text_node(b, wa, key, wa.fonts.serif.clone(), 18.0, if accent { PAPER } else { INK }, texts, lang);
        });
}

pub fn build_ui(mut commands: Commands, wa: Res<WorldAssets>, texts: Res<Texts>, lang: Res<Lang>) {
    let f = &wa.fonts;
    let t = &*texts;
    let lg = *lang;
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|root| {
            // ===== HUD =====
            root.spawn((
                HudPlate,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(14.0),
                    top: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(3.0),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.11, 0.11, 0.13, 0.72)),
            ))
            .with_children(|plate| {
                text_node(plate, &wa, "hud.level", f.mono.clone(), 11.0, ACCENT, t, lg);
                text_node(plate, &wa, "level.1", f.serif.clone(), 16.0, PAPER, t, lg);
                plate.spawn((HudTime, Text::new(""), TextFont { font: f.mono.clone().into(), font_size: 11.0.into(), ..default() }, TextColor(ACCENT)));
                plate.spawn((HudScore, Text::new(""), TextFont { font: f.mono.clone().into(), font_size: 14.0.into(), ..default() }, TextColor(PAPER)));
            });

            // ===== TITLE =====
            root.spawn((
                ScreenTitle,
                BackgroundColor(Color::srgba(0.102, 0.122, 0.18, 0.55)),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|sr| {
                card_with(sr, |card| {
                    text_node(card, &wa, "game.kicker", f.mono.clone(), 13.0, ACCENT, t, lg);
                    text_node(card, &wa, "game.title", f.serif_bold.clone(), 36.0, INK, t, lg);
                    text_node(card, &wa, "game.thesis", f.serif_italic.clone(), 16.0, MUTED, t, lg);
                    card.spawn((Node { width: Val::Px(64.0), height: Val::Px(3.0), ..default() }, BackgroundColor(ACCENT)));
                    card.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(24.0),
                        ..default()
                    }).with_children(|row| {
                        row.spawn((ImageNode::new(wa.fixture.clone()), Node {
                            width: Val::Px(170.0),
                            ..default()
                        }));
                        row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), ..default() })
                            .with_children(|col| {
                                text_node(col, &wa, "game.subtitle", f.serif_bold.clone(), 26.0, INK, t, lg);
                                text_node(col, &wa, "intro.objective", f.serif_italic.clone(), 15.0, MUTED, t, lg);
                                col.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
                                    .with_children(|btns| {
                                        button(btns, &wa, "menu.play", BtnAction::Play, t, lg, true);
                                        button(btns, &wa, "menu.about", BtnAction::About, t, lg, false);
                                    });
                                col.spawn((BestLine, Text::new(""), TextFont { font: f.mono.clone().into(), font_size: 11.0.into(), ..default() }, TextColor(MUTED)));
                                col.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(8.0), ..default() })
                                    .with_children(|langs| {
                                        button(langs, &wa, "lang.en", BtnAction::LangEn, t, lg, false);
                                        if HAS_JA {
                                            button(langs, &wa, "lang.ja", BtnAction::LangJa, t, lg, false);
                                        }
                                    });
                            });
                    });
                    text_node(card, &wa, "game.footer", f.mono.clone(), 11.0, MUTED, t, lg);
                });
            });

            // ===== ABOUT =====
            root.spawn((
                ScreenAbout,
                BackgroundColor(Color::srgba(0.102, 0.122, 0.18, 0.55)),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|sr| {
                card_with(sr, |card| {
                    text_node(card, &wa, "about.heading", f.mono.clone(), 13.0, ACCENT, t, lg);
                    text_node(card, &wa, "about.verdict", f.serif_bold.clone(), 30.0, INK, t, lg);
                    card.spawn((ImageNode::new(wa.concept3d.clone()), Node { width: Val::Px(420.0), ..default() }));
                    text_node(card, &wa, "about.body", f.serif.clone(), 15.0, INK, t, lg);
                    text_node(card, &wa, "about.provenance", f.mono.clone(), 11.0, MUTED, t, lg);
                    button(card, &wa, "about.close", BtnAction::AboutBack, t, lg, false);
                });
            });

            // ===== CRASHED =====
            simple_screen(root, ScreenCrashed, &wa, t, lg, &["msg.crashed", "msg.crashed.body"], 30.0, &[
                ("msg.retry", BtnAction::Retry, true),
                ("msg.restart", BtnAction::Menu, false),
            ]);

            // ===== CLEARED =====
            root.spawn((
                ScreenCleared,
                BackgroundColor(Color::srgba(0.102, 0.122, 0.18, 0.55)),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|sr| {
                card_with(sr, |card| {
                    text_node(card, &wa, "msg.cleared", f.mono.clone(), 13.0, ACCENT, t, lg);
                    text_node(card, &wa, "msg.cleared.body", f.serif_bold.clone(), 28.0, INK, t, lg);
                    card.spawn((ClearedScores, Text::new(""), TextFont { font: f.mono.clone().into(), font_size: 17.0.into(), ..default() }, TextColor(INK)));
                    card.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
                        .with_children(|btns| {
                            button(btns, &wa, "msg.next", BtnAction::Next, t, lg, true);
                            button(btns, &wa, "msg.retry", BtnAction::Retry, t, lg, false);
                        });
                });
            });

            // ===== ALL CLEAR =====
            root.spawn((
                ScreenAllClear,
                BackgroundColor(Color::srgba(0.102, 0.122, 0.18, 0.55)),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|sr| {
                card_with(sr, |card| {
                    text_node(card, &wa, "game.kicker", f.mono.clone(), 13.0, ACCENT, t, lg);
                    text_node(card, &wa, "msg.allclear", f.serif_bold.clone(), 40.0, ACCENT, t, lg);
                    text_node(card, &wa, "msg.allclear.body", f.serif_italic.clone(), 16.0, MUTED, t, lg);
                    card.spawn((AllClearTotal, Text::new(""), TextFont { font: f.serif_bold.clone().into(), font_size: 24.0.into(), ..default() }, TextColor(INK)));
                    card.spawn((BestLine, Text::new(""), TextFont { font: f.mono.clone().into(), font_size: 12.0.into(), ..default() }, TextColor(MUTED)));
                    card.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
                        .with_children(|btns| {
                            button(btns, &wa, "menu.play", BtnAction::Play, t, lg, true);
                            button(btns, &wa, "msg.restart", BtnAction::Menu, t, lg, false);
                        });
                    text_node(card, &wa, "game.footer", f.mono.clone(), 11.0, MUTED, t, lg);
                });
            });
        });
}

fn simple_screen(
    root: &mut ChildSpawnerCommands,
    marker: impl Component + Send + Sync + 'static,
    wa: &WorldAssets,
    t: &Texts,
    lg: Lang,
    lines: &[&str],
    h_size: f32,
    buttons: &[(&str, BtnAction, bool)],
) {
    let f = &wa.fonts;
    root.spawn((
        marker,
        BackgroundColor(Color::srgba(0.102, 0.122, 0.18, 0.55)),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
    ))
    .with_children(|sr| {
        sr.spawn(Node {
            width: Val::Px(620.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(38.0)),
            ..default()
        })
        .with_children(|card| {
            let mut first = true;
            for key in lines {
                if first {
                    text_node(card, wa, key, f.mono.clone(), 15.0, ACCENT, t, lg);
                    first = false;
                } else {
                    text_node(card, wa, key, f.serif_bold.clone(), h_size, INK, t, lg);
                }
            }
            card.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
                .with_children(|btns| {
                    for (key, action, accent) in buttons {
                        button(btns, wa, key, *action, t, lg, *accent);
                    }
                });
        });
    });
}

/// per-frame HUD + dynamic text
pub fn hud_update(
    game: Res<GameRes>,
    state: Res<State<AppState>>,
    lang: Res<Lang>,
    texts: Res<Texts>,
    mut q_time: Query<&mut Text, With<HudTime>>,
    mut q_score: Query<&mut Text, (With<HudScore>, Without<HudTime>)>,
    mut q_cleared: Query<&mut Text, (With<ClearedScores>, Without<HudTime>, Without<HudScore>)>,
    mut q_all: Query<&mut Text, (With<AllClearTotal>, Without<HudTime>, Without<HudScore>, Without<ClearedScores>)>,
    mut q_best: Query<&mut Text, (With<BestLine>, Without<HudTime>, Without<HudScore>, Without<ClearedScores>, Without<AllClearTotal>)>,
    mut q_hud_plate: Query<&mut Node, (With<HudPlate>, Without<ScreenTitle>)>,
    mut q_level_name: Query<&mut Text, (With<HudLevelName>, Without<HudTime>, Without<HudScore>, Without<ClearedScores>, Without<AllClearTotal>, Without<BestLine>)>,
) {
    let playing = matches!(state.get(), AppState::Playing | AppState::Crashed | AppState::Intro);
    for mut n in q_hud_plate.iter_mut() {
        n.display = if playing { Display::Flex } else { Display::None };
    }
    if let Ok(mut t) = q_time.single_mut() {
        **t = format!("{:.1}s · {}: {}", game.time, texts.t(*lang, "hud.flips"), game.flips);
    }
    if let Ok(mut t) = q_score.single_mut() {
        **t = format!("{}: {}", texts.t(*lang, "hud.score"), game.total);
    }
    if let Ok(mut t) = q_level_name.single_mut() {
        let key = format!("level.{}", game.current + 1);
        let v = texts.t(*lang, &key);
        if **t != v {
            **t = v;
        }
    }
    if *state.get() == AppState::Cleared {
        if let Ok(mut t) = q_cleared.single_mut() {
            **t = format!(
                "{} 250 · {} +{} · {} +{} = {}",
                texts.t(*lang, "msg.finish"),
                texts.t(*lang, "msg.flipbonus"),
                game.breakdown.1,
                texts.t(*lang, "msg.timebonus"),
                game.breakdown.2,
                game.level_score
            );
        }
    }
    if *state.get() == AppState::AllClear {
        if let Ok(mut t) = q_all.single_mut() {
            **t = format!("{}: {}", texts.t(*lang, "msg.total"), game.total);
        }
    }
    for mut t in q_best.iter_mut() {
        let v = if game.best > 0 {
            format!("{}: {}", texts.t(*lang, "menu.best"), game.best)
        } else {
            String::new()
        };
        if **t != v {
            **t = v;
        }
    }
}

/// re-render every L10N node on language change + button interactions
pub fn ui_interact(
    interaction_q: Query<(&Interaction, &BtnAction), Changed<Interaction>>,
    mut game: ResMut<GameRes>,
    mut next: ResMut<NextState<AppState>>,
    mut lang: ResMut<Lang>,
    mut texts_changed: Local<bool>,
    mut about_open: Local<bool>,
    mut q_l10n: Query<(&L10N_MARKER, &mut Text)>,
    texts: Res<Texts>,
    mut about_q: Query<&mut Node, With<ScreenAbout>>,
    state: Res<State<AppState>>,
) {
    for (i, action) in interaction_q.iter() {
        if *i != Interaction::Pressed {
            continue;
        }
        match action {
            BtnAction::Play => {
                game.total = 0;
                game.start_level(0);
                next.set(AppState::Intro);
            }
            BtnAction::About => *about_open = true,
            BtnAction::AboutBack => *about_open = false,
            BtnAction::Retry => {
                let cur = game.current;
                game.start_level(cur);
                next.set(AppState::Intro);
            }
            BtnAction::Next => {
                if let Some(s) = game.advance_level() {
                    next.set(s);
                }
            }
            BtnAction::Menu => {
                game.total = 0;
                next.set(AppState::Title);
            }
            BtnAction::LangEn => {
                if *lang != Lang::En {
                    *lang = Lang::En;
                    *texts_changed = true;
                }
            }
            BtnAction::LangJa => {
                if *lang != Lang::Ja {
                    *lang = Lang::Ja;
                    *texts_changed = true;
                }
            }
        }
    }
    // about screen visibility (independent of game state)
    for mut n in about_q.iter_mut() {
        n.display = if *about_open { Display::Flex } else { Display::None };
    }
    if *state.get() == AppState::Title && *about_open {
        // keep title under about
    }
    if *texts_changed {
        for (marker, mut t) in q_l10n.iter_mut() {
            **t = texts.t(*lang, &marker.0);
        }
        *texts_changed = false;
    }
}
