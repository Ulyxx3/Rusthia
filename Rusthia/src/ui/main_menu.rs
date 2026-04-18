// src/ui/main_menu.rs
// ==============================================================================
// Rusthia — Écran Menu Principal / Song Select
// Menu de sélection de maps avec jaquettes, stats sauvegardées et liste défilante
// ==============================================================================

use bevy::prelude::*;
use bevy::window::FileDragAndDrop;
use crate::{
    GameState,
    map::{LoadMapEvent, ActiveMap},
    game::{attempt::{AttemptState, GameSettings}, save::SaveManager},
    ui::loading::MapDatabase,
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
            .add_systems(
                Update,
                (
                    handle_file_drop,
                    handle_map_selection,
                    handle_play_button,
                    handle_import_button,
                    handle_settings_toggle,
                    handle_sensitivity_buttons,
                    update_sensitivity_display,
                    handle_advanced_settings_buttons,
                    update_advanced_display,
                    update_left_panel_details,
                    handle_left_panel_scroll,
                    handle_ui_scale_buttons,
                    update_ui_scale_display,
                    apply_ui_scale_to_window,
                    // Mods pré-jeu
                    handle_no_fail_toggle,
                    handle_speed_buttons,
                    update_speed_display,
                )
                    .run_if(in_state(GameState::MainMenu)),
            );
    }
}

// ==============================================================================
// COMPOSANTS & RESSOURCES
// ==============================================================================

#[derive(Component)] struct MainMenuRoot;
#[derive(Component)] struct MapListContainer;
#[derive(Component)] struct MapButton(usize);
#[derive(Component)] struct CoverImageDisplay;
#[derive(Component)] struct TitleDisplay;
#[derive(Component)] struct StatsDisplay;
#[derive(Component)] struct PlayButton;
#[derive(Component)] struct ImportBtn;
#[derive(Component)] struct SettingsToggleBtn;
#[derive(Component)] struct SettingsPanel;

// Réglages généraux
#[derive(Component)] struct SensDisplay;
#[derive(Component)] struct SensDecBtn;
#[derive(Component)] struct SensIncBtn;

#[derive(Component)] struct CursorScaleDisplay;
#[derive(Component)] struct CursorScaleDecBtn;
#[derive(Component)] struct CursorScaleIncBtn;

#[derive(Component)] struct ParallaxDisplay;
#[derive(Component)] struct ParallaxDecBtn;
#[derive(Component)] struct ParallaxIncBtn;

#[derive(Component)] struct NoteShapeDisplay;
#[derive(Component)] struct NoteShapeDecBtn;
#[derive(Component)] struct NoteShapeIncBtn;

#[derive(Component)] struct HitboxDisplay;
#[derive(Component)] struct HitboxDecBtn;
#[derive(Component)] struct HitboxIncBtn;

#[derive(Component)] struct ApproachRateDisplay;
#[derive(Component)] struct ApproachRateDecBtn;
#[derive(Component)] struct ApproachRateIncBtn;

#[derive(Component)] struct ApproachDistDisplay;
#[derive(Component)] struct ApproachDistDecBtn;
#[derive(Component)] struct ApproachDistIncBtn;

#[derive(Component)] struct NoteSizeDisplay;
#[derive(Component)] struct NoteSizeDecBtn;
#[derive(Component)] struct NoteSizeIncBtn;

#[derive(Component)] struct FpsShowDisplay;
#[derive(Component)] struct FpsShowDecBtn;
#[derive(Component)] struct FpsShowIncBtn;

// Échelle interface
#[derive(Component)] struct UiScaleDisplay;
#[derive(Component)] struct UiScaleDecBtn;
#[derive(Component)] struct UiScaleIncBtn;

// Mods pré-jeu (panneau gauche)
#[derive(Component)] struct NoFailToggleBtn;
#[derive(Component)] struct NoFailDisplay;
#[derive(Component)] struct SpeedDecBtn;
#[derive(Component)] struct SpeedIncBtn;
#[derive(Component)] struct SpeedDisplay;

// Scroll
#[derive(Component)] struct LeftPanelScrollable;

#[derive(Resource, Default)]
struct SelectedMapIndex(Option<usize>);

// ==============================================================================
// SETUP DE L'INTERFACE
// ==============================================================================

fn setup_main_menu(
    mut commands: Commands,
    db: Res<MapDatabase>,
    settings: Res<GameSettings>,
) {
    let selected_idx = if !db.entries.is_empty() { Some(0) } else { None };
    commands.insert_resource(SelectedMapIndex(selected_idx));
    commands.spawn((Camera2d::default(), MainMenuRoot));

    commands.spawn((
        MainMenuRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.04, 0.04, 0.08)),
    )).with_children(|root| {

        // ==================================================================
        // HEADER (TOP)
        // ==================================================================
        root.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(70.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(30.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BorderColor(Color::srgba(0.2, 0.2, 0.4, 0.5)),
            BackgroundColor(Color::srgba(0.06, 0.06, 0.1, 0.95)),
        )).with_children(|header| {
            header.spawn((
                Text::new("RUSTHIA"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::srgb(0.0, 0.8, 1.0)),
            ));

            header.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                ..default()
            }).with_children(|btns| {
                btns.spawn((
                    SettingsToggleBtn, Button,
                    Node { width: Val::Px(120.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)), BorderColor(Color::srgba(0.4, 0.4, 0.6, 0.8)),
                )).with_children(|b| { b.spawn((Text::new("Réglages"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });

                btns.spawn((
                    ImportBtn, Button,
                    Node { width: Val::Px(120.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)), BorderColor(Color::srgba(0.4, 0.4, 0.6, 0.8)),
                )).with_children(|b| { b.spawn((Text::new("Importer"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
            });
        });

        // ==================================================================
        // CONTENU PRINCIPAL
        // ==================================================================
        root.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        }).with_children(|main_content| {

            // ── PANNEAU GAUCHE : MAP DETAILS + MODS ──────────────────────
            main_content.spawn((
                Node {
                    width: Val::Percent(45.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::all(Val::Px(30.0)),
                    row_gap: Val::Px(20.0),
                    border: UiRect::right(Val::Px(2.0)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                BorderColor(Color::srgba(0.2, 0.2, 0.4, 0.5)),
            )).with_children(|left| {

                // Info block (jaquette + titre)
                left.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    align_items: AlignItems::Center,
                    ..default()
                }).with_children(|ib| {
                    ib.spawn((
                        CoverImageDisplay, ImageNode::default(),
                        Node { width: Val::Px(120.0), height: Val::Px(120.0), border: UiRect::all(Val::Px(2.0)), ..default() },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.5)), BorderColor(Color::srgb(0.2, 0.5, 0.8)),
                    ));
                    ib.spawn((
                        TitleDisplay,
                        Text::new("Sélectionnez une map\n--"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

                // Stats
                left.spawn((
                    StatsDisplay,
                    Text::new("Meilleur Score: --\nMax Combo: --\nPrécision: --"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                ));

                // ── Séparateur mods ────────────────────────────────────────
                left.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(4.0)), ..default() },
                    BackgroundColor(Color::srgba(0.3, 0.3, 0.5, 0.4)),
                ));

                // Titre section mods
                left.spawn((
                    Text::new("⚙ Modificateurs"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgba(0.7, 0.7, 0.9, 0.8)),
                ));

                // No Fail toggle
                left.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                }).with_children(|row| {
                    row.spawn((
                        NoFailToggleBtn, Button,
                        Node {
                            width: Val::Px(120.0), height: Val::Px(36.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)), ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.12, 0.22, 0.9)),
                        BorderColor(Color::srgba(0.3, 0.3, 0.5, 0.7)),
                    )).with_children(|b| {
                        b.spawn((
                            NoFailDisplay,
                            Text::new("No Fail : NON"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::srgba(0.7, 0.7, 0.9, 1.0)),
                        ));
                    });
                });

                // Vitesse de la map (slider − / valeur / +)
                left.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                }).with_children(|row| {
                    row.spawn((
                        Text::new("Vitesse :"),
                        TextFont { font_size: 15.0, ..default() },
                        TextColor(Color::srgba(0.7, 0.7, 0.9, 0.8)),
                        Node { width: Val::Px(78.0), ..default() },
                    ));

                    let btn_style = Node {
                        width: Val::Px(34.0), height: Val::Px(34.0),
                        justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)), ..default()
                    };
                    row.spawn((SpeedDecBtn, Button, btn_style.clone(), BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)), BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5))))
                        .with_children(|b| { b.spawn((Text::new("−"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });

                    let speed_text = format!("{:.2}x", settings.map_speed);
                    row.spawn((
                        SpeedDisplay,
                        Text::new(speed_text),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.0, 0.85, 1.0)),
                        Node { width: Val::Px(56.0), justify_content: JustifyContent::Center, ..default() },
                    ));

                    row.spawn((SpeedIncBtn, Button, btn_style.clone(), BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)), BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5))))
                        .with_children(|b| { b.spawn((Text::new("+"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                });

                // ── Séparateur ─────────────────────────────────────────────
                left.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(4.0)), ..default() },
                    BackgroundColor(Color::srgba(0.3, 0.3, 0.5, 0.4)),
                ));

                // Bouton JOUER
                left.spawn((
                    PlayButton, Button,
                    Node {
                        width: Val::Px(200.0), height: Val::Px(60.0),
                        align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.0)), ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.6, 0.2)), BorderColor(Color::srgb(0.2, 0.8, 0.3)),
                )).with_children(|btn| {
                    btn.spawn((Text::new("JOUER"), TextFont { font_size: 26.0, ..default() }, TextColor(Color::WHITE)));
                });
            });

            // ── PANNEAU DROIT : LISTE DES MAPS (scrollable) ───────────────
            main_content.spawn(Node {
                width: Val::Percent(55.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            }).with_children(|right| {
                right.spawn((
                    Text::new("Collection de Maps"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                    Node { margin: UiRect::bottom(Val::Px(15.0)), ..default() }
                ));

                right.spawn((
                    MapListContainer,
                    LeftPanelScrollable,
                    Interaction::default(),
                    bevy::ui::ScrollPosition::default(),
                    Node {
                        width: Val::Percent(100.0), height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column, row_gap: Val::Px(10.0),
                        overflow: Overflow::clip_y(), ..default()
                    }
                )).with_children(|list| {
                    if db.entries.is_empty() {
                        list.spawn((
                            Text::new("Aucune map trouvée dans le dossier 'maps/'\nCréez le dossier ou utilisez le bouton Importer."),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::srgba(0.8, 0.3, 0.3, 0.8)),
                        ));
                    } else {
                        for (i, entry) in db.entries.iter().enumerate() {
                            list.spawn((
                                MapButton(i), Button,
                                Node {
                                    width: Val::Percent(100.0), height: Val::Px(60.0),
                                    flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center,
                                    padding: UiRect::horizontal(Val::Px(15.0)), border: UiRect::all(Val::Px(1.0)), ..default()
                                },
                                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)), BorderColor(Color::srgba(0.3, 0.3, 0.4, 0.5)),
                            )).with_children(|btn| {
                                btn.spawn((
                                    Text::new(entry.data.pretty_title()),
                                    TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE),
                                ));
                                btn.spawn((
                                    Text::new(format!("Difficulté: {}", &entry.data.difficulty_name)),
                                    TextFont { font_size: 12.0, ..default() }, TextColor(Color::srgba(0.6, 0.6, 0.8, 0.8)),
                                ));
                            });
                        }
                    }
                });
            });
        });

        // ==================================================================
        // PANNEAU OVERLAY SETTINGS (scrollable)
        // ==================================================================
        settings_panel(root, &settings);
    });
}

fn settings_panel(parent: &mut ChildBuilder, settings: &GameSettings) {
    parent
        .spawn((
            SettingsPanel,
            LeftPanelScrollable,
            Interaction::default(),
            bevy::ui::ScrollPosition::default(),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::axes(Val::Px(0.0), Val::Px(60.0)),
                row_gap: Val::Px(12.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.98)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("⚙ PARAMÈTRES RUSTHIA"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::WHITE),
                Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() }
            ));

            create_settings_row(panel, "Sensibilité Pointeur", SensDecBtn, SensDisplay, &format!("{:.2}", settings.sensitivity), SensIncBtn);
            create_settings_row(panel, "Taille du Curseur",    CursorScaleDecBtn, CursorScaleDisplay, &format!("{:.2}", settings.cursor_size), CursorScaleIncBtn);
            create_settings_row(panel, "Force Parallaxe",      ParallaxDecBtn, ParallaxDisplay, &format!("{:.2}", settings.parallax_strength), ParallaxIncBtn);
            let shape_str = match settings.note_shape { 0 => "Carré", 1 => "Squircle", _ => "Cercle" };
            create_settings_row(panel, "Forme des Notes",      NoteShapeDecBtn, NoteShapeDisplay, shape_str, NoteShapeIncBtn);
            create_settings_row(panel, "Taille Hitbox",        HitboxDecBtn, HitboxDisplay, &format!("{:.2}", settings.hitbox_size), HitboxIncBtn);
            create_settings_row(panel, "Vitesse Approche",     ApproachRateDecBtn, ApproachRateDisplay, &format!("{:.1}", settings.approach_rate), ApproachRateIncBtn);
            create_settings_row(panel, "Distance Notes",       ApproachDistDecBtn, ApproachDistDisplay, &format!("{:.1}", settings.approach_distance), ApproachDistIncBtn);
            create_settings_row(panel, "Taille des Notes",     NoteSizeDecBtn, NoteSizeDisplay, &format!("{:.2}", settings.note_size), NoteSizeIncBtn);
            create_settings_row(panel, "Afficher FPS",         FpsShowDecBtn, FpsShowDisplay, if settings.show_fps { "Oui" } else { "Non" }, FpsShowIncBtn);
            create_settings_row(panel, "Échelle Interface",    UiScaleDecBtn, UiScaleDisplay, &format!("{:.1}", settings.ui_scale), UiScaleIncBtn);

            panel.spawn((
                SettingsToggleBtn, Button,
                Node {
                    width: Val::Px(200.0), height: Val::Px(50.0),
                    margin: UiRect::top(Val::Px(40.0)),
                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)), ..default()
                },
                BackgroundColor(Color::srgba(0.6, 0.2, 0.2, 0.8)), BorderColor(Color::srgba(0.8, 0.3, 0.3, 0.8)),
            )).with_children(|btn| {
                btn.spawn((Text::new("FERMER"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
            });
        });
}

fn create_settings_row<D: Bundle, V: Bundle, I: Bundle>(
    p: &mut ChildBuilder,
    label: &str,
    dec_comp: D,
    val_comp: V,
    val_text: &str,
    inc_comp: I,
) {
    let text_style = TextFont { font_size: 18.0, ..default() };
    let btn_size = Node { width: Val::Px(40.0), height: Val::Px(40.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, border: UiRect::all(Val::Px(1.0)), ..default() };

    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(20.0),
        margin: UiRect::bottom(Val::Px(10.0)),
        ..default()
    }).with_children(|row| {
        row.spawn((Text::new(label), text_style.clone(), TextColor(Color::srgba(0.8, 0.8, 0.9, 0.9)), Node { width: Val::Px(220.0), ..default() }));
        row.spawn((dec_comp, Button, btn_size.clone(), BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)), BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5))))
            .with_children(|b| { b.spawn((Text::new("−"), text_style.clone(), TextColor(Color::WHITE))); });
        row.spawn((val_comp, Text::new(val_text), text_style.clone(), TextColor(Color::srgb(0.0, 0.85, 1.0)), Node { width: Val::Px(90.0), justify_content: JustifyContent::Center, ..default() }));
        row.spawn((inc_comp, Button, btn_size.clone(), BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)), BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5))))
            .with_children(|b| { b.spawn((Text::new("+"), text_style.clone(), TextColor(Color::WHITE))); });
    });
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ==============================================================================
// SCROLL (liste maps ET panneau réglages)
// ==============================================================================

fn handle_left_panel_scroll(
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut bevy::ui::ScrollPosition, &Interaction, &Node), With<LeftPanelScrollable>>,
) {
    let mut dy = 0.0;
    for event in mouse_wheel.read() {
        dy += event.y * -30.0;
    }
    if dy == 0.0 { return; }

    for (mut scroll, interaction, node) in query.iter_mut() {
        // Le SettingsPanel est Absolute + Flex quand visible → toujours scroller
        let is_visible_overlay = node.position_type == PositionType::Absolute && node.display == Display::Flex;
        if *interaction == Interaction::Hovered || is_visible_overlay {
            scroll.offset_y = (scroll.offset_y + dy).max(0.0);
        }
    }
}

// ==============================================================================
// SÉLECTION DE MAP
// ==============================================================================

fn handle_map_selection(
    mut interactions: Query<(&Interaction, &MapButton), Changed<Interaction>>,
    mut selected_idx: ResMut<SelectedMapIndex>,
) {
    for (int, btn) in interactions.iter_mut() {
        if *int == Interaction::Pressed {
            selected_idx.0 = Some(btn.0);
        }
    }
}

fn update_left_panel_details(
    selected_idx: Res<SelectedMapIndex>,
    db: Res<MapDatabase>,
    saves: Res<SaveManager>,
    mut title_q: Query<&mut Text, With<TitleDisplay>>,
    mut stats_q: Query<&mut Text, (With<StatsDisplay>, Without<TitleDisplay>)>,
    mut cover_q: Query<&mut ImageNode, With<CoverImageDisplay>>,
    mut images: ResMut<Assets<Image>>,
    mut last_rendered: Local<Option<usize>>,
) {
    if !selected_idx.is_changed() && *last_rendered == selected_idx.0 { return; }
    *last_rendered = selected_idx.0;

    let Some(idx) = selected_idx.0 else { return };
    if let Some(entry) = db.entries.get(idx) {
        let map = &entry.data;

        for mut text in title_q.iter_mut() {
            **text = format!("{}\n{}", map.title, map.artist);
        }

        for mut text in stats_q.iter_mut() {
            if let Some(record) = saves.get_record(&map.id) {
                let mods_str = if record.best_score_mods.is_empty() {
                    String::new()
                } else {
                    format!("\nMods record : {}", record.best_score_mods)
                };
                **text = format!(
                    "Meilleur Score : {}\nMax Combo : {}x\nPrécision : {:.2}%\nDurée : {}s{}",
                    record.best_score, record.best_combo, record.best_accuracy,
                    map.length_ms / 1000, mods_str
                );
            } else {
                **text = format!(
                    "Meilleur Score : --\nMax Combo : --\nPrécision : --\nDurée : {}s",
                    map.length_ms / 1000
                );
            }
        }

        if !map.cover.is_empty() {
            match Image::from_buffer(
                &map.cover,
                bevy::image::ImageType::Extension("png"),
                bevy::image::CompressedImageFormats::all(),
                false,
                bevy::image::ImageSampler::Default,
                bevy::render::render_asset::RenderAssetUsages::default(),
            ) {
                Ok(image) => {
                    let handle = images.add(image);
                    for mut img in cover_q.iter_mut() { img.image = handle.clone(); }
                }
                Err(e) => warn!("Erreur décodage cover : {}", e),
            }
        } else {
            for mut img in cover_q.iter_mut() { img.image = Handle::default(); }
        }
    }
}

// ==============================================================================
// BOUTON JOUER
// ==============================================================================

fn handle_play_button(
    mut commands: Commands,
    mut interactions: Query<&Interaction, (With<PlayButton>, Changed<Interaction>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_idx: Res<SelectedMapIndex>,
    db: Res<MapDatabase>,
    mut attempt: ResMut<AttemptState>,
    settings: Res<GameSettings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let pressed = interactions.iter().any(|i| *i == Interaction::Pressed)
        || keyboard.just_pressed(KeyCode::Enter);

    if pressed {
        if let Some(idx) = selected_idx.0 {
            if let Some(entry) = db.entries.get(idx) {
                commands.insert_resource(ActiveMap(entry.data.clone()));
                *attempt = AttemptState::default();

                // Transférer les mods de GameSettings vers AttemptState
                attempt.speed = settings.map_speed;
                attempt.no_fail_active = settings.no_fail;

                // Construire le mods_label
                let mut mods = Vec::new();
                if settings.no_fail { mods.push("NF".to_string()); }
                if (settings.map_speed - 1.0).abs() > 0.01 {
                    mods.push(format!("{:.2}x", settings.map_speed));
                }
                attempt.mods_label = mods.join(" • ");

                next_state.set(GameState::InGame);
            }
        }
    }
}

// ==============================================================================
// IMPORT / DRAG AND DROP
// ==============================================================================

fn handle_file_drop(
    mut dnd_events: EventReader<FileDragAndDrop>,
    mut load_events: EventWriter<LoadMapEvent>,
) {
    for event in dnd_events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let path_str = path_buf.to_string_lossy().to_string();
            let ext = path_str.rsplit('.').next().unwrap_or("").to_lowercase();
            if matches!(ext.as_str(), "phxm" | "sspm" | "txt") {
                if let Ok(bytes) = std::fs::read(path_buf) {
                    let filename = path_buf.file_name().unwrap_or_default().to_string_lossy().to_string();
                    load_events.send(LoadMapEvent { bytes, filename });
                }
            }
        }
    }
}

fn handle_import_button(
    mut interactions: Query<&Interaction, (With<ImportBtn>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interactions.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Rhythia Maps", &["phxm", "sspm"])
                .pick_file()
            {
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let dest = std::path::PathBuf::from("maps").join(&filename);
                let _ = std::fs::copy(&path, &dest);
                info!("Map importée dans {:?}, rechargement...", dest);
                next_state.set(GameState::Loading);
            }
        }
    }
}

// ==============================================================================
// TOGGLE SETTINGS
// ==============================================================================

fn handle_settings_toggle(
    mut interactions: Query<&Interaction, (With<SettingsToggleBtn>, Changed<Interaction>)>,
    mut panel_q: Query<&mut Node, With<SettingsPanel>>,
) {
    for interaction in interactions.iter() {
        if *interaction == Interaction::Pressed {
            for mut node in panel_q.iter_mut() {
                node.display = match node.display {
                    Display::None => Display::Flex,
                    _ => Display::None,
                };
            }
        }
    }
}

// ==============================================================================
// MODIFICATEURS PRÉ-JEU (No Fail + Vitesse)
// ==============================================================================

fn handle_no_fail_toggle(
    mut settings: ResMut<GameSettings>,
    q: Query<&Interaction, (With<NoFailToggleBtn>, Changed<Interaction>)>,
) {
    for i in q.iter() {
        if *i == Interaction::Pressed { settings.no_fail = !settings.no_fail; }
    }
}

fn handle_speed_buttons(
    mut settings: ResMut<GameSettings>,
    dec_q: Query<&Interaction, (With<SpeedDecBtn>, Changed<Interaction>)>,
    inc_q: Query<&Interaction, (With<SpeedIncBtn>, Changed<Interaction>)>,
) {
    for i in dec_q.iter() {
        if *i == Interaction::Pressed {
            settings.map_speed = ((settings.map_speed * 4.0 - 1.0).round() / 4.0).max(0.25);
        }
    }
    for i in inc_q.iter() {
        if *i == Interaction::Pressed {
            settings.map_speed = ((settings.map_speed * 4.0 + 1.0).round() / 4.0).min(2.0);
        }
    }
}

fn update_speed_display(
    settings: Res<GameSettings>,
    mut q_speed: Query<&mut Text, With<SpeedDisplay>>,
    mut q_nf: Query<&mut Text, (With<NoFailDisplay>, Without<SpeedDisplay>)>,
) {
    if !settings.is_changed() { return; }
    for mut t in q_speed.iter_mut() {
        **t = format!("{:.2}x", settings.map_speed);
    }
    for mut t in q_nf.iter_mut() {
        **t = if settings.no_fail { "No Fail : OUI".to_string() } else { "No Fail : NON".to_string() };
    }
}

// ==============================================================================
// RÉGLAGES GÉNÉRAUX (boutons +/-)
// ==============================================================================

fn handle_sensitivity_buttons(
    mut settings: ResMut<GameSettings>,
    dec_sens_q: Query<&Interaction, (With<SensDecBtn>, Changed<Interaction>)>,
    inc_sens_q: Query<&Interaction, (With<SensIncBtn>, Changed<Interaction>)>,
    dec_cur_q:  Query<&Interaction, (With<CursorScaleDecBtn>, Changed<Interaction>)>,
    inc_cur_q:  Query<&Interaction, (With<CursorScaleIncBtn>, Changed<Interaction>)>,
    dec_par_q:  Query<&Interaction, (With<ParallaxDecBtn>, Changed<Interaction>)>,
    inc_par_q:  Query<&Interaction, (With<ParallaxIncBtn>, Changed<Interaction>)>,
    dec_shape_q: Query<&Interaction, (With<NoteShapeDecBtn>, Changed<Interaction>)>,
    inc_shape_q: Query<&Interaction, (With<NoteShapeIncBtn>, Changed<Interaction>)>,
    dec_hitbox_q: Query<&Interaction, (With<HitboxDecBtn>, Changed<Interaction>)>,
    inc_hitbox_q: Query<&Interaction, (With<HitboxIncBtn>, Changed<Interaction>)>,
) {
    for i in dec_sens_q.iter()   { if *i == Interaction::Pressed { settings.sensitivity = (settings.sensitivity - 0.25).max(0.25); } }
    for i in inc_sens_q.iter()   { if *i == Interaction::Pressed { settings.sensitivity = (settings.sensitivity + 0.25).min(10.0); } }
    for i in dec_cur_q.iter()    { if *i == Interaction::Pressed { settings.cursor_size = (settings.cursor_size - 0.02).max(0.04); } }
    for i in inc_cur_q.iter()    { if *i == Interaction::Pressed { settings.cursor_size = (settings.cursor_size + 0.02).min(0.5); } }
    for i in dec_par_q.iter()    { if *i == Interaction::Pressed { settings.parallax_strength = (settings.parallax_strength - 0.05).max(0.0); } }
    for i in inc_par_q.iter()    { if *i == Interaction::Pressed { settings.parallax_strength = (settings.parallax_strength + 0.05).min(1.0); } }
    for i in dec_shape_q.iter()  { if *i == Interaction::Pressed { settings.note_shape = match settings.note_shape { 0 => 2, 1 => 0, 2 => 1, _ => 0 }; } }
    for i in inc_shape_q.iter()  { if *i == Interaction::Pressed { settings.note_shape = (settings.note_shape + 1) % 3; } }
    for i in dec_hitbox_q.iter() { if *i == Interaction::Pressed { settings.hitbox_size = (settings.hitbox_size - 0.05).max(0.05); } }
    for i in inc_hitbox_q.iter() { if *i == Interaction::Pressed { settings.hitbox_size = (settings.hitbox_size + 0.05).min(1.0); } }
}

fn update_sensitivity_display(
    settings: Res<GameSettings>,
    mut q_sens:    Query<&mut Text, With<SensDisplay>>,
    mut q_cursor:  Query<&mut Text, (With<CursorScaleDisplay>, Without<SensDisplay>)>,
    mut q_par:     Query<&mut Text, (With<ParallaxDisplay>, Without<SensDisplay>, Without<CursorScaleDisplay>)>,
    mut q_shape:   Query<&mut Text, (With<NoteShapeDisplay>, Without<SensDisplay>, Without<CursorScaleDisplay>, Without<ParallaxDisplay>)>,
    mut q_hitbox:  Query<&mut Text, (With<HitboxDisplay>, Without<SensDisplay>, Without<CursorScaleDisplay>, Without<ParallaxDisplay>, Without<NoteShapeDisplay>)>,
) {
    if !settings.is_changed() { return; }
    for mut t in q_sens.iter_mut()   { **t = format!("{:.2}", settings.sensitivity); }
    for mut t in q_cursor.iter_mut() { **t = format!("{:.2}", settings.cursor_size); }
    for mut t in q_par.iter_mut()    { **t = format!("{:.2}", settings.parallax_strength); }
    for mut t in q_shape.iter_mut()  { **t = match settings.note_shape { 0 => "Carré".into(), 1 => "Squircle".into(), _ => "Cercle".into() }; }
    for mut t in q_hitbox.iter_mut() { **t = format!("{:.2}", settings.hitbox_size); }
}

fn handle_advanced_settings_buttons(
    mut settings: ResMut<GameSettings>,
    dec_ar_q: Query<&Interaction, (With<ApproachRateDecBtn>, Changed<Interaction>)>,
    inc_ar_q: Query<&Interaction, (With<ApproachRateIncBtn>, Changed<Interaction>)>,
    dec_ad_q: Query<&Interaction, (With<ApproachDistDecBtn>, Changed<Interaction>)>,
    inc_ad_q: Query<&Interaction, (With<ApproachDistIncBtn>, Changed<Interaction>)>,
    dec_ns_q: Query<&Interaction, (With<NoteSizeDecBtn>, Changed<Interaction>)>,
    inc_ns_q: Query<&Interaction, (With<NoteSizeIncBtn>, Changed<Interaction>)>,
    dec_fps_q: Query<&Interaction, (With<FpsShowDecBtn>, Changed<Interaction>)>,
    inc_fps_q: Query<&Interaction, (With<FpsShowIncBtn>, Changed<Interaction>)>,
) {
    for i in dec_ar_q.iter()  { if *i == Interaction::Pressed { settings.approach_rate = (settings.approach_rate - 0.5).max(1.0); } }
    for i in inc_ar_q.iter()  { if *i == Interaction::Pressed { settings.approach_rate = (settings.approach_rate + 0.5).min(50.0); } }
    for i in dec_ad_q.iter()  { if *i == Interaction::Pressed { settings.approach_distance = (settings.approach_distance - 1.0).max(1.0); } }
    for i in inc_ad_q.iter()  { if *i == Interaction::Pressed { settings.approach_distance = (settings.approach_distance + 1.0).min(50.0); } }
    for i in dec_ns_q.iter()  { if *i == Interaction::Pressed { settings.note_size = (settings.note_size - 0.05).max(0.05); } }
    for i in inc_ns_q.iter()  { if *i == Interaction::Pressed { settings.note_size = (settings.note_size + 0.05).min(2.0); } }
    for i in dec_fps_q.iter() { if *i == Interaction::Pressed { settings.show_fps = !settings.show_fps; } }
    for i in inc_fps_q.iter() { if *i == Interaction::Pressed { settings.show_fps = !settings.show_fps; } }
}

fn update_advanced_display(
    settings: Res<GameSettings>,
    mut q_ar:  Query<&mut Text, With<ApproachRateDisplay>>,
    mut q_ad:  Query<&mut Text, (With<ApproachDistDisplay>, Without<ApproachRateDisplay>)>,
    mut q_ns:  Query<&mut Text, (With<NoteSizeDisplay>, Without<ApproachRateDisplay>, Without<ApproachDistDisplay>)>,
    mut q_fps: Query<&mut Text, (With<FpsShowDisplay>, Without<ApproachRateDisplay>, Without<ApproachDistDisplay>, Without<NoteSizeDisplay>)>,
) {
    if !settings.is_changed() { return; }
    for mut t in q_ar.iter_mut()  { **t = format!("{:.1}", settings.approach_rate); }
    for mut t in q_ad.iter_mut()  { **t = format!("{:.1}", settings.approach_distance); }
    for mut t in q_ns.iter_mut()  { **t = format!("{:.2}", settings.note_size); }
    for mut t in q_fps.iter_mut() { **t = if settings.show_fps { "Oui".to_string() } else { "Non".to_string() }; }
}

// ==============================================================================
// ÉCHELLE DE L'INTERFACE
// ==============================================================================

fn handle_ui_scale_buttons(
    mut settings: ResMut<GameSettings>,
    dec_q: Query<&Interaction, (With<UiScaleDecBtn>, Changed<Interaction>)>,
    inc_q: Query<&Interaction, (With<UiScaleIncBtn>, Changed<Interaction>)>,
) {
    for i in dec_q.iter() {
        if *i == Interaction::Pressed {
            settings.ui_scale = ((settings.ui_scale - 0.1) * 10.0).round() / 10.0;
            settings.ui_scale = settings.ui_scale.max(0.3);
        }
    }
    for i in inc_q.iter() {
        if *i == Interaction::Pressed {
            settings.ui_scale = ((settings.ui_scale + 0.1) * 10.0).round() / 10.0;
            settings.ui_scale = settings.ui_scale.min(1.5);
        }
    }
}

fn update_ui_scale_display(
    settings: Res<GameSettings>,
    mut q: Query<&mut Text, With<UiScaleDisplay>>,
) {
    if !settings.is_changed() { return; }
    for mut t in q.iter_mut() { **t = format!("{:.1}", settings.ui_scale); }
}

/// Applique UiScale à la fenêtre chaque fois que ui_scale change dans GameSettings.
fn apply_ui_scale_to_window(
    settings: Res<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
) {
    if !settings.is_changed() { return; }
    ui_scale.0 = settings.ui_scale;
}
