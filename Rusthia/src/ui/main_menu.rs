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
                    update_left_panel_details,
                    handle_left_panel_scroll,
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
#[derive(Component)] struct MapButton(usize); // Index in MapDatabase
#[derive(Component)] struct SelectedMapDisplay; // To update the left panel
#[derive(Component)] struct CoverImageDisplay;
#[derive(Component)] struct TitleDisplay;
#[derive(Component)] struct StatsDisplay;
#[derive(Component)] struct PlayButton;
#[derive(Component)] struct ImportBtn;
#[derive(Component)] struct SettingsToggleBtn;
#[derive(Component)] struct SettingsPanel;

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

#[derive(Component)] struct LeftPanelScrollable;

#[derive(Resource, Default)]
struct SelectedMapIndex(Option<usize>);

// ==============================================================================
// SETUP DE L'INTERFACE
// ==============================================================================

fn setup_main_menu(
    mut commands: Commands,
    db: Res<MapDatabase>,
    mut images: ResMut<Assets<Image>>,
) {
    // Par défaut, première map sélectionnée
    let selected_idx = if !db.entries.is_empty() { Some(0) } else { None };
    commands.insert_resource(SelectedMapIndex(selected_idx));

    commands.spawn((Camera2d::default(), MainMenuRoot));

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::srgb(0.04, 0.04, 0.08)), // Fond général sombre
        ))
        .with_children(|root| {
            // ==================================================================
            // PANNEAU GAUCHE : DÉTAILS DE LA MAP
            // ==================================================================
            root.spawn((
                LeftPanelScrollable,
                Interaction::default(), // pour savoir s'il est hoveré, Optionnel si on scroll globalement
                bevy::ui::ScrollPosition::default(),
                Node {
                    width: Val::Percent(45.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(15.0),
                    border: UiRect::right(Val::Px(2.0)),
                    overflow: Overflow::clip_y(), // Permet au contenu de dépasser invisiblement
                    ..default()
                },
                BorderColor(Color::srgba(0.2, 0.2, 0.4, 0.5)),
            )).with_children(|left_panel| {
                // Titre RUSTHIA discret en haut à gauche
                left_panel.spawn((
                    Text::new("RUSTHIA"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::srgb(0.0, 0.8, 1.0)),
                    Node {
                        align_self: AlignSelf::FlexStart,
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    }
                ));

                // Image de la jaquette (Cover)
                left_panel.spawn((
                    CoverImageDisplay,
                    ImageNode::default(),
                    Node {
                        width: Val::Px(240.0), // Réduit de 300 à 240
                        height: Val::Px(240.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.5)),
                    BorderColor(Color::srgb(0.2, 0.5, 0.8)),
                ));

                // Titre et Artiste
                left_panel.spawn((
                    TitleDisplay,
                    Text::new("Sélectionnez une map"),
                    TextFont { font_size: 26.0, ..default() }, // Réduction de 32 à 26
                    TextColor(Color::WHITE),
                ));

                // Zone Stats (Score, Combo)
                left_panel.spawn((
                    StatsDisplay,
                    Text::new("Meilleur Score: --\nMax Combo: --\nPrécision: --"),
                    TextFont { font_size: 16.0, ..default() }, // Réduction de 18 à 16
                    TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                ));

                // Bouton JOUER
                left_panel.spawn((
                    PlayButton,
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.6, 0.2)),
                    BorderColor(Color::srgb(0.2, 0.8, 0.3)),
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new("JOUER"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

                // Boutons Utilitaires : Paramètres & Importer
                left_panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                }).with_children(|buttons_row| {
                    // Settings
                    buttons_row.spawn((
                        SettingsToggleBtn,
                        Button,
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(36.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)),
                        BorderColor(Color::srgba(0.4, 0.4, 0.6, 0.8)),
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new("⚙ Réglages"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Importer
                    buttons_row.spawn((
                        ImportBtn,
                        Button,
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(36.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)),
                        BorderColor(Color::srgba(0.4, 0.4, 0.6, 0.8)),
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new("📂 Importer"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                });

                // Panneau Settings Caché (Overlay)
                settings_panel(left_panel);
            });

            // ==================================================================
            // PANNEAU DROIT : LISTE DES MAPS
            // ==================================================================
            root.spawn(Node {
                width: Val::Percent(55.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            }).with_children(|right_panel| {
                right_panel.spawn((
                    Text::new("Collection de Maps"),
                    TextFont { font_size: 28.0, ..default() },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    }
                ));

                // Conteneur de liste ("ScrollView" rudimentaire, sans clip pour l'instant)
                right_panel.spawn((
                    MapListContainer,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        overflow: Overflow::clip_y(),
                        ..default()
                    }
                )).with_children(|list| {
                    if db.entries.is_empty() {
                        list.spawn((
                            Text::new("Aucune map trouvée dans le dossier 'maps/'\nCréez le dossier ou utilisez le bouton Importer."),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::srgba(0.8, 0.3, 0.3, 0.8)),
                        ));
                    } else {
                        // Génération des boutons pour chaque map trouvée
                        for (i, entry) in db.entries.iter().enumerate() {
                            list.spawn((
                                MapButton(i),
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(60.0),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::horizontal(Val::Px(15.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
                                BorderColor(Color::srgba(0.3, 0.3, 0.4, 0.5)),
                            )).with_children(|btn| {
                                btn.spawn((
                                    Text::new(entry.data.pretty_title()),
                                    TextFont { font_size: 18.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                                btn.spawn((
                                    Text::new(format!("Difficulté: {}", &entry.data.difficulty_name)),
                                    TextFont { font_size: 12.0, ..default() },
                                    TextColor(Color::srgba(0.6, 0.6, 0.8, 0.8)),
                                ));
                            });
                        }
                    }
                });
            });
        });
}

fn settings_panel(parent: &mut ChildBuilder) {
    parent
        .spawn((
            SettingsPanel,
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0), // Gap réduit
                padding: UiRect::all(Val::Px(12.0)), // Padding réduit
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::top(Val::Px(10.0)), // Margin réduit
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.12, 0.95)),
            BorderColor(Color::srgba(0.3, 0.3, 0.6, 0.8)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("⚙ Paramètres de Jeu"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgba(0.8, 0.8, 0.9, 0.9)),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() }
            ));

            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Sensibilité Pointeur"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgba(0.75, 0.75, 0.9, 0.9)),
                    ));
                    row.spawn((
                        SensDecBtn,
                        Button,
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)),
                        BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("−"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    });

                    row.spawn((
                        SensDisplay,
                        Text::new("2.0"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.0, 0.85, 1.0)),
                        Node { width: Val::Px(36.0), justify_content: JustifyContent::Center, ..default() },
                    ));

                    row.spawn((
                        SensIncBtn,
                        Button,
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)),
                        BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("+"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    });
                });

            // Ligne pour la taille du curseur
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Taille du Curseur"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgba(0.75, 0.75, 0.9, 0.9)),
                    ));
                    row.spawn((
                        CursorScaleDecBtn,
                        Button,
                        Node {
                            width: Val::Px(28.0), height: Val::Px(28.0),
                            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)), ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)),
                        BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("−"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    });

                    row.spawn((
                        CursorScaleDisplay,
                        Text::new("0.12"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.0, 0.85, 1.0)),
                        Node { width: Val::Px(36.0), justify_content: JustifyContent::Center, ..default() },
                    ));

                    row.spawn((
                        CursorScaleIncBtn,
                        Button,
                        Node {
                            width: Val::Px(28.0), height: Val::Px(28.0),
                            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)), ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)),
                        BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("+"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    });
                });

             // Ligne pour la parallaxe
             panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Force Parallaxe"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgba(0.75, 0.75, 0.9, 0.9)),
                    ));
                    row.spawn((
                        ParallaxDecBtn,
                        Button,
                        Node {
                            width: Val::Px(28.0), height: Val::Px(28.0),
                            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)), ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)),
                        BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("−"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    });

                    row.spawn((
                        ParallaxDisplay,
                        Text::new("0.15"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.0, 0.85, 1.0)),
                        Node { width: Val::Px(36.0), justify_content: JustifyContent::Center, ..default() },
                    ));

                    row.spawn((
                        ParallaxIncBtn,
                        Button,
                        Node {
                            width: Val::Px(28.0), height: Val::Px(28.0),
                            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(1.0)), ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)),
                        BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("+"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    });
                });

             // Ligne pour le style de Note (Forme)
             panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Forme des Notes"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgba(0.75, 0.75, 0.9, 0.9)),
                    ));
                    row.spawn((
                        NoteShapeDecBtn, Button,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)), BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    )).with_children(|b| { b.spawn((Text::new("−"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });

                    row.spawn((
                        NoteShapeDisplay,
                        Text::new("Squircle"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.0, 0.85, 1.0)),
                        Node { width: Val::Px(80.0), justify_content: JustifyContent::Center, ..default() },
                    ));

                    row.spawn((
                        NoteShapeIncBtn, Button,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)), BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    )).with_children(|b| { b.spawn((Text::new("+"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                });

             // Ligne pour la taille de la Hitbox
             panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Taille Hitbox"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgba(0.75, 0.75, 0.9, 0.9)),
                    ));
                    row.spawn((
                        HitboxDecBtn, Button,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)), BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    )).with_children(|b| { b.spawn((Text::new("−"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });

                    row.spawn((
                        HitboxDisplay,
                        Text::new("0.35"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.0, 0.85, 1.0)),
                        Node { width: Val::Px(36.0), justify_content: JustifyContent::Center, ..default() },
                    ));

                    row.spawn((
                        HitboxIncBtn, Button,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.25, 0.8)), BorderColor(Color::srgba(0.2, 0.2, 0.5, 0.5)),
                    )).with_children(|b| { b.spawn((Text::new("+"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                });
        });
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ==============================================================================
// SYSTÈMES ET INTERACTIONS
// ==============================================================================

fn handle_left_panel_scroll(
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut bevy::ui::ScrollPosition, &Interaction), With<LeftPanelScrollable>>,
) {
    let mut dy = 0.0;
    for event in mouse_wheel.read() {
        // En pixels : inverser pour un scroll descendant naturel
        dy += event.y * -30.0;
    }
    if dy != 0.0 {
        // Si on hover le panneau gauche, on scroll
        for (mut scroll, interaction) in query.iter_mut() {
            if *interaction == Interaction::Hovered {
                scroll.offset_y += dy;
                // Empêcher d'aller en négatif (haut du scroll)
                if scroll.offset_y < 0.0 {
                    scroll.offset_y = 0.0;
                }
            }
        }
    }
}

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
    mut commands: Commands,
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
                **text = format!(
                    "Meilleur Score : {}\nMax Combo : {}x\nPrécision : {:.2}%\nDurée : {}s",
                    record.best_score, record.best_combo, record.best_accuracy, map.length_ms / 1000
                );
            } else {
                **text = format!(
                    "Meilleur Score : --\nMax Combo : --\nPrécision : --\nDurée : {}s",
                    map.length_ms / 1000
                );
            }
        }

        // Gestion de la CoverImage
        if !map.cover.is_empty() {
             // Pour transformer le buffer de l'image (PNG/JPG) en Handle<Image>
             // Note: Cela demande feature png et jpeg sur image ou bevy
             match Image::from_buffer(
                 &map.cover,
                 bevy::image::ImageType::Extension("png"), // Supposition standard Rhythia
                 bevy::image::CompressedImageFormats::all(),
                 false,
                 bevy::image::ImageSampler::Default,
                 bevy::render::render_asset::RenderAssetUsages::default(),
             ) {
                 Ok(image) => {
                     let handle = images.add(image);
                     for mut img in cover_q.iter_mut() {
                         img.image = handle.clone();
                     }
                 }
                 Err(e) => warn!("Erreur décodage cover : {}", e),
             }
        } else {
             // Retirer l'image si la map n'en a pas
             for mut img in cover_q.iter_mut() {
                 img.image = Handle::default();
             }
        }
    }
}

fn handle_play_button(
    mut commands: Commands,
    mut interactions: Query<&Interaction, (With<PlayButton>, Changed<Interaction>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_idx: Res<SelectedMapIndex>,
    db: Res<MapDatabase>,
    mut attempt: ResMut<AttemptState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let pressed = interactions.iter().any(|i| *i == Interaction::Pressed)
        || keyboard.just_pressed(KeyCode::Enter);

    if pressed {
        if let Some(idx) = selected_idx.0 {
            if let Some(entry) = db.entries.get(idx) {
                // Créer ActiveMap à partir de db
                commands.insert_resource(ActiveMap(entry.data.clone()));
                *attempt = AttemptState::default();
                next_state.set(GameState::InGame);
            }
        }
    }
}

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

fn handle_sensitivity_buttons(
    mut settings: ResMut<GameSettings>,
    dec_sens_q: Query<&Interaction, (With<SensDecBtn>, Changed<Interaction>)>,
    inc_sens_q: Query<&Interaction, (With<SensIncBtn>, Changed<Interaction>)>,
    dec_cur_q: Query<&Interaction, (With<CursorScaleDecBtn>, Changed<Interaction>)>,
    inc_cur_q: Query<&Interaction, (With<CursorScaleIncBtn>, Changed<Interaction>)>,
    dec_par_q: Query<&Interaction, (With<ParallaxDecBtn>, Changed<Interaction>)>,
    inc_par_q: Query<&Interaction, (With<ParallaxIncBtn>, Changed<Interaction>)>,
    dec_shape_q: Query<&Interaction, (With<NoteShapeDecBtn>, Changed<Interaction>)>,
    inc_shape_q: Query<&Interaction, (With<NoteShapeIncBtn>, Changed<Interaction>)>,
    dec_hitbox_q: Query<&Interaction, (With<HitboxDecBtn>, Changed<Interaction>)>,
    inc_hitbox_q: Query<&Interaction, (With<HitboxIncBtn>, Changed<Interaction>)>,
) {
    for interaction in dec_sens_q.iter() {
        if *interaction == Interaction::Pressed { settings.sensitivity = (settings.sensitivity - 0.25).max(0.25); }
    }
    for interaction in inc_sens_q.iter() {
        if *interaction == Interaction::Pressed { settings.sensitivity = (settings.sensitivity + 0.25).min(10.0); }
    }
    for interaction in dec_cur_q.iter() {
        if *interaction == Interaction::Pressed { settings.cursor_size = (settings.cursor_size - 0.02).max(0.04); }
    }
    for interaction in inc_cur_q.iter() {
        if *interaction == Interaction::Pressed { settings.cursor_size = (settings.cursor_size + 0.02).min(0.5); }
    }
    for interaction in dec_par_q.iter() {
        if *interaction == Interaction::Pressed { settings.parallax_strength = (settings.parallax_strength - 0.05).max(0.0); }
    }
    for interaction in inc_par_q.iter() {
        if *interaction == Interaction::Pressed { settings.parallax_strength = (settings.parallax_strength + 0.05).min(1.0); }
    }
    for interaction in dec_shape_q.iter() {
        if *interaction == Interaction::Pressed {
            settings.note_shape = match settings.note_shape { 0 => 2, 1 => 0, 2 => 1, _ => 0 };
        }
    }
    for interaction in inc_shape_q.iter() {
        if *interaction == Interaction::Pressed {
            settings.note_shape = (settings.note_shape + 1) % 3;
        }
    }
    for interaction in dec_hitbox_q.iter() {
        if *interaction == Interaction::Pressed { settings.hitbox_size = (settings.hitbox_size - 0.05).max(0.05); }
    }
    for interaction in inc_hitbox_q.iter() {
        if *interaction == Interaction::Pressed { settings.hitbox_size = (settings.hitbox_size + 0.05).min(1.0); }
    }
}

fn update_sensitivity_display(
    settings: Res<GameSettings>,
    mut q_sens: Query<&mut Text, With<SensDisplay>>,
    mut q_cursor: Query<&mut Text, (With<CursorScaleDisplay>, Without<SensDisplay>)>,
    mut q_parallax: Query<&mut Text, (With<ParallaxDisplay>, Without<SensDisplay>, Without<CursorScaleDisplay>)>,
    mut q_shape: Query<&mut Text, (With<NoteShapeDisplay>, Without<SensDisplay>, Without<CursorScaleDisplay>, Without<ParallaxDisplay>)>,
    mut q_hitbox: Query<&mut Text, (With<HitboxDisplay>, Without<SensDisplay>, Without<CursorScaleDisplay>, Without<ParallaxDisplay>, Without<NoteShapeDisplay>)>,
) {
    if !settings.is_changed() { return; }
    for mut text in q_sens.iter_mut() { **text = format!("{:.2}", settings.sensitivity); }
    for mut text in q_cursor.iter_mut() { **text = format!("{:.2}", settings.cursor_size); }
    for mut text in q_parallax.iter_mut() { **text = format!("{:.2}", settings.parallax_strength); }
    for mut text in q_shape.iter_mut() {
        **text = match settings.note_shape {
            0 => "Carré".into(),
            1 => "Squircle".into(),
            _ => "Cercle".into(),
        };
    }
    for mut text in q_hitbox.iter_mut() { **text = format!("{:.2}", settings.hitbox_size); }
}
