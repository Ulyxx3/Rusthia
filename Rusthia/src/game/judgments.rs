// src/game/judgments.rs
// ==============================================================================
// Rusthia — Hit Detection AUTO (hover-to-hit, pas click-to-hit)
// Traduction de HitJudgment.cs + HealthJudgment.cs
//
// SoundSpace / Rhythia = jeu de type "cursor hover"
//   → Aucun clic nécessaire
//   → Si cursor.position est dans la hitbox ET delta <= HIT_WINDOW → hit auto
//
// HealthJudgment :
//   hit  → healthStep = max(step/1.45, 15) ; health = min(100, health + step/1.75)
//   miss → health = max(0, health - step)  ; healthStep = min(step*1.2, 100)
// ==============================================================================

use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use crate::{
    game::{
        attempt::{AttemptState, GameSettings},
        spawner::{NoteHitEvent, NoteMissEvent, NoteComponent},
    },
    map::types::{HIT_WINDOW_MS, GRID_SIZE, CURSOR_SIZE},
};

// ==============================================================================
// RESSOURCE CURSEUR
// ==============================================================================

/// Position du curseur dans l'espace de jeu normalisé [-1, 1]
/// Traduction de CurrentAttempt.CursorPosition dans GameComponent.cs
#[derive(Resource, Default, Debug)]
pub struct GameCursor {
    pub x: f32,
    pub y: f32,
}

impl GameCursor {
    /// Test d'intersection curseur/note (hitbox carrée)
    /// HitJudgment.cs: CursorPosition.DistanceTo(Vector2(note.X, note.Y)) <= HIT_BOX_SIZE
    #[inline]
    pub fn in_hitbox(&self, note_x: f32, note_y: f32, hitbox_size: f32) -> bool {
        (self.x - note_x).abs() <= hitbox_size
            && (self.y - note_y).abs() <= hitbox_size
    }
}

// ==============================================================================
// SYSTÈMES
// ==============================================================================

/// Mettre à jour la position du curseur depuis les mouvements souris.
/// Traduction de GameComponent.cs _Input() :
///   `CurrentAttempt.CursorPosition += delta / sensitivity / 57.5`
pub fn update_cursor(
    mut mouse_motion: EventReader<MouseMotion>,
    mut cursor: ResMut<GameCursor>,
    settings: Res<GameSettings>,
) {
    for event in mouse_motion.read() {
        // Nouvelle formule : cursor_delta = mouse_delta * sensitivity / BASE
        // BASE = 300 → à sensitivity=2.0, il faut 150px pour traverser 1 unité de grille
        // Éch. : sensitivity=1 → 300px/unité (lent), sensitivity=6 → 50px/unité (rapide)
        let base: f32 = 300.0;
        cursor.x += event.delta.x * settings.sensitivity / base;
        cursor.y -= event.delta.y * settings.sensitivity / base;

        // Bornes de la grille
        let half = GRID_SIZE / 2.0;
        let cursor_half = CURSOR_SIZE / 2.0;
        let bound = (half - cursor_half) / half;
        cursor.x = cursor.x.clamp(-bound, bound);
        cursor.y = cursor.y.clamp(-bound, bound);
    }
}

/// Auto-hit — vérifie CHAQUE FRAME si le curseur est sur une note hittable.
///
/// Règles :
///   1. La note doit être dans la fenêtre temporelle : |progress - note.ms| ≤ HIT_WINDOW
///   2. Le curseur doit être dans la hitbox : |cursor.x - note.x| ≤ HIT_BOX_SIZE
///   3. La note n'a pas encore été frappée
///
/// Contrairement au clic, il suffit de "passer dessus" au bon moment.
/// C'est le mécanisme fondamental de SoundSpace+.
pub fn check_auto_hit(
    cursor: Res<GameCursor>,
    mut note_query: Query<&mut NoteComponent>,
    attempt: Res<AttemptState>,
    settings: Res<GameSettings>,
    mut hit_events: EventWriter<NoteHitEvent>,
) {
    if attempt.paused || attempt.failed || !attempt.audio_started { return; }

    for mut note in note_query.iter_mut() {
        if note.is_hit || !note.is_hittable { continue; }

        // Vérification temporelle
        let delta_ms = (attempt.progress_ms - note.millisecond as f64).abs();
        if delta_ms > HIT_WINDOW_MS { continue; }

        // Vérification spatiale — curseur dans la hitbox de la note
        if cursor.in_hitbox(note.grid_x, note.grid_y, settings.hitbox_size) {
            note.is_hit = true;
            note.is_hittable = false;
            hit_events.send(NoteHitEvent {
                note_index: note.note_index,
                delta_ms,
            });
            // Pas de break — une note par frame max, les autres seront vérifiées au prochain frame
        }
    }
}

/// Mettre à jour santé/score suite aux hits et misses.
/// Traduction exacte de HealthJudgment.cs defaultHealthResult()
pub fn update_health(
    mut attempt: ResMut<AttemptState>,
    mut hit_events: EventReader<NoteHitEvent>,
    mut miss_events: EventReader<NoteMissEvent>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    for _ in hit_events.read() {
        // HIT: step descend (récompense), health monte
        attempt.health_step = (attempt.health_step / 1.45).max(15.0);
        attempt.health = (attempt.health + attempt.health_step / 1.75).min(100.0);
        attempt.combo += 1;
        attempt.best_combo = attempt.best_combo.max(attempt.combo);
        attempt.hits += 1;
        attempt.score += 100 * attempt.combo as i64;
    }

    for _ in miss_events.read() {
        // MISS: step monte (punition exponentielle), health descend
        attempt.health = (attempt.health - attempt.health_step).max(0.0);
        attempt.health_step = (attempt.health_step * 1.2).min(100.0);
        attempt.combo = 0;
        attempt.misses += 1;
    }

    // Fail si santé épuisée
    if attempt.health <= 0.0 && !attempt.failed {
        attempt.failed = true;
        next_state.set(crate::GameState::Results);
    }
}

/// Pause avec Échap
pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut attempt: ResMut<AttemptState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        attempt.paused = !attempt.paused;
    }
}
