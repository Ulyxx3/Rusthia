// src/main.rs
// ==============================================================================
// Rusthia — Point d'entrée principal
// Traduction idiomatique du singleton Rhythia.cs (Godot) en Bevy ECS
// ==============================================================================

// Les APIs publiques non encore appelées dans cette phase de développement
#![allow(dead_code)]

mod audio;
mod game;
mod map;
mod ui;

use bevy::prelude::*;

/// États du jeu — traduction de SceneManager.cs
/// Godot utilisait des noms de scène implicites (MainMenu, LegacyRunner, SceneResults)
/// Bevy utilise une enum typée avec des transitions explicites
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    /// Chargement initial (= "res://scenes/loading.tscn" dans l'original)
    #[default]
    Loading,
    /// Menu principal (= SceneMenu)
    MainMenu,
    /// Jeu en cours (= SceneGame / LegacyRunner)
    InGame,
    /// Pause pendant le jeu
    Paused,
    /// Écran de résultats (= SceneResults)
    Results,
}

fn main() {
    App::new()
        // --- Plugins Bevy de base ---
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rusthia".into(),
                        resolution: (1280.0, 720.0).into(),
                        // Accepter les fichiers glissés-déposés
                        ..default()
                    }),
                    ..default()
                })
                // Désactiver l'audio Bevy natif — on utilise Kira à la place
                .disable::<bevy::audio::AudioPlugin>(),
        )
        // --- Machine à états ---
        .init_state::<GameState>()
        // --- Plugins du jeu ---
        .add_plugins((
            audio::RusthiaAudioPlugin,
            map::MapPlugin,
            game::GamePlugin,
            ui::UiPlugin,
        ))
        // Gestion du curseur selon l'état
        .add_systems(Update, manage_cursor_visibility)
        .run();
}

/// Cacher et confiner le curseur en jeu, le libérer dans les menus.
/// Équivalent de `Input.mouse_mode = MOUSE_MODE_CAPTURED` dans Godot
fn manage_cursor_visibility(
    state: Res<State<GameState>>,
    mut windows: Query<&mut Window>,
) {
    let Ok(mut window) = windows.get_single_mut() else { return };

    match state.get() {
        GameState::InGame => {
            // Capturer le curseur pour le mouvement relatif
            window.cursor_options.grab_mode = bevy::window::CursorGrabMode::Confined;
            window.cursor_options.visible = false;
        }
        _ => {
            // Libérer le curseur dans les menus
            window.cursor_options.grab_mode = bevy::window::CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}