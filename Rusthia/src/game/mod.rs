// src/game/mod.rs
// ==============================================================================
// Rusthia — GamePlugin
// Orchestre les systèmes de jeu en séquence garantie
// ==============================================================================

pub mod attempt;
pub mod judgments;
pub mod spawner;
pub mod save;

use bevy::prelude::*;
use attempt::{AttemptState, GameSettings};
use judgments::GameCursor;
use judgments::{update_cursor, check_auto_hit, update_health, handle_pause_input};
use spawner::{NoteHitEvent, NoteMissEvent};
use spawner::{
    setup_game_scene, cleanup_game_scene,
    spawn_notes, move_notes, despawn_processed_notes,
    update_cursor_visual, update_camera_parallax, update_health_bar_3d,
};
use crate::{
    GameState,
    audio::KiraManager,
    map::ActiveMap,
};

// ==============================================================================
// RESSOURCES INTERNES
// ==============================================================================

/// Audio en attente de démarrage (pendant le countdown pré-jeu)
/// Évite de jouer l'audio immédiatement à l'entrée dans InGame
#[derive(Resource)]
pub struct PendingAudio(pub Vec<u8>);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(save::SavePlugin)
            // --- Ressources ---
            .insert_resource(AttemptState::default())
            .insert_resource(GameSettings::default())
            .insert_resource(GameCursor::default())

            // --- Événements ---
            .add_event::<NoteHitEvent>()
            .add_event::<NoteMissEvent>()

            // --- Systèmes d'init / nettoyage ---
            .add_systems(
                OnEnter(GameState::InGame),
                (setup_game_scene, prepare_audio).chain(),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (cleanup_game_scene, stop_game_audio).chain(),
            )

            .add_systems(
                OnEnter(GameState::Results),
                save_score_on_end,
            )

            // --- Boucle principale : ordre strict par .chain() ---
            .add_systems(
                Update,
                (
                    update_cursor,           // 1. Déplacer le curseur
                    update_cursor_visual,    // 2. Afficher le curseur 3D
                    update_camera_parallax,  // Parallaxe perspective caméra
                    tick_pregame,            // 3. Countdown → démarrage audio
                    spawn_notes,             // 4. Spawner les notes
                    move_notes,              // 5. Animer les notes
                    despawn_processed_notes, // 6. Nettoyer les notes passées
                    check_auto_hit,          // 7. Détecter les hits auto
                    update_health,           // 8. Mettre à jour santé/score
                    update_health_bar_3d,    // 8.5 Mettre à jour le Mesh de la barre
                    handle_pause_input,      // 9. Pause
                    check_game_end,          // 10. Détecter la fin de la map
                )
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

// ==============================================================================
// SYSTÈMES SUPPLÉMENTAIRES
// ==============================================================================

/// Stocker l'audio dans PendingAudio — l'audio démarrera après le countdown.
fn prepare_audio(
    map_res: Option<Res<ActiveMap>>,
    mut commands: Commands,
) {
    let Some(active_map) = map_res else {
        warn!("InGame sans map chargée !");
        return;
    };

    let audio = active_map.0.audio.clone();
    if audio.is_empty() {
        warn!("La map n'a pas d'audio.");
        return;
    }

    commands.insert_resource(PendingAudio(audio));
    info!(
        "Map prête : {} — countdown 3 secondes",
        active_map.0.pretty_title()
    );
}

/// Arrêter l'audio Kira et nettoyer les ressources temporaires.
/// Exécuté sur OnExit(InGame) — garantit que la musique ne continue PAS dans les menus.
fn stop_game_audio(
    mut kira: ResMut<KiraManager>,
    mut commands: Commands,
) {
    // Arrêter la lecture audio en cours
    if let Some(mut handle) = kira.song_handle.take() {
        let _ = handle.stop(kira::Tween::default());
        info!("Audio arrêté.");
    }

    // Supprimer PendingAudio si le countdown n'était pas encore terminé
    commands.remove_resource::<PendingAudio>();
}

/// Décompter le délai pré-jeu et démarrer l'audio quand il atteint 0.
///
/// Pendant le countdown :
///   - attempt.progress_ms est NÉGATIF (géré dans spawn_notes)
///   - Les notes commencent à apparaître (elles s'approchent depuis loin)
///   - Le joueur peut se préparer
///
/// À la fin du countdown :
///   - Kira démarre la lecture
///   - sync_audio_clock() prend le relai
fn tick_pregame(
    pending: Option<Res<PendingAudio>>,
    mut kira: ResMut<KiraManager>,
    mut attempt: ResMut<AttemptState>,
    mut commands: Commands,
) {
    // Déjà démarré = rien à faire
    if attempt.audio_started { return; }

    // Le countdown est géré dans spawn_notes() via time.delta_secs
    // Ici on surveille juste quand il atteint 0
    if attempt.pregame_remaining_ms <= 0.0 {
        if let Some(audio) = pending {
            kira.play_from_bytes(&audio.0);
            commands.remove_resource::<PendingAudio>();
            info!("Audio démarré — synchronisation Kira activée");
        }
        attempt.audio_started = true;
    }
}

/// Détecter la fin naturelle de la map (toutes notes traitées + audio terminé)
fn check_game_end(
    attempt: Res<AttemptState>,
    map_res: Option<Res<ActiveMap>>,
    kira: Res<KiraManager>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if attempt.failed { return; } // Déjà géré par update_health
    let Some(map) = map_res else { return };

    let all_notes_done = attempt.next_spawn_index >= map.0.notes.len();
    let audio_done = !kira.is_playing()
        || attempt.progress_ms >= map.0.length_ms as f64 + 1000.0;

    if all_notes_done && audio_done && attempt.audio_started {
        info!("Map terminée ! Score:{} Acc:{:.2}%", attempt.score, attempt.accuracy());
        next_state.set(GameState::Results);
    }
}

/// Sauvegarde le score que la partie soit gagnée ou perdue
fn save_score_on_end(
    mut save_manager: ResMut<save::SaveManager>,
    attempt: Res<AttemptState>,
    map_res: Option<Res<ActiveMap>>,
) {
    if let Some(map) = map_res {
        save_manager.submit_result(&map.0.id, attempt.score, attempt.best_combo, attempt.accuracy());
        info!("Score enregistré : {}", attempt.score);
    }
}
