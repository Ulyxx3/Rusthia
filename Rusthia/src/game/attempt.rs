// src/game/attempt.rs
// ==============================================================================
// Rusthia — État d'une session de jeu
// Traduction de Attempt.cs + SettingsProfile
// ==============================================================================

use bevy::prelude::*;

/// Paramètres d'affichage et de gameplay — équivalent de SettingsProfile
/// Valeurs par défaut calquées sur les settings originaux de Rhythia
#[derive(Resource, Clone)]
pub struct GameSettings {
    /// Vitesse d'approche (approach rate) — default: 12.0
    /// Contrôle la vitesse à laquelle les notes arrivent
    pub approach_rate: f32,

    /// Distance d'approche (approach distance) en unités monde — default: 10.0
    /// Distance maximale depuis laquelle les notes sont visibles
    pub approach_distance: f32,

    /// Fade-in en % de la distance d'approach — default: 50.0 (%)
    /// Les notes apparaissent graduellement sur les premiers 50% du trajet
    pub fade_in: f32,

    /// Taille des notes dans la grille — default: 0.25
    pub note_size: f32,

    /// Opacité globale des notes [0.0, 1.0] — default: 1.0
    pub note_opacity: f32,

    /// Mode Pushback: les notes reculent quand elles dépassent le plan — default: false
    pub pushback: bool,

    /// Ghost mode: les notes disparaissent à l'approche de la zone de frappe — default: false
    pub ghost_mode: bool,

    /// Fade-out factor [0.0, 10.0] — 0 = désactivé — default: 0
    pub fade_out: f32,

    /// Sensibilité de la souris — échelle 0.5–10 (1=lent, 3=normal, 10=très rapide)
    pub sensitivity: f32,
    
    /// Taille du curseur
    pub cursor_size: f32,
    
    /// Force de l'effet de parallaxe de la caméra
    pub parallax_strength: f32,
    
    /// Forme de la note : 0=Carré plein (Défaut), 1=Squircle (Contour Arrondi), 2=Cercle
    pub note_shape: u8,
    
    /// Taille de la Hitbox
    pub hitbox_size: f32,

    /// Volume maître [0.0, 1.0]
    pub volume_master: f32,

    /// Volume musique [0.0, 1.0]
    pub volume_music: f32,

    /// Volume SFX [0.0, 1.0]
    pub volume_sfx: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            approach_rate: 12.0,
            approach_distance: 10.0,
            fade_in: 50.0,
            note_size: 0.25,
            note_opacity: 1.0,
            pushback: false,
            ghost_mode: false,
            fade_out: 0.0,
            sensitivity: 2.0,
            cursor_size: 0.12,
            parallax_strength: 0.15,
            note_shape: 1, // par défaut on met squircle comme demandé
            hitbox_size: 0.35, // hitbox plus grande (défaut avant: 0.18)
            volume_master: 1.0,
            volume_music: 1.0,
            volume_sfx: 1.0,
        }
    }
}

impl GameSettings {
    /// Calculer l'approachTime en secondes: at = ad / ar
    /// Utilisé dans la formule de depth du LegacyRenderer
    #[inline]
    pub fn approach_time_secs(&self) -> f32 {
        self.approach_distance / self.approach_rate
    }

    /// Calculer la profondeur de la hitwindow pour le mode Pushback
    /// LegacyRenderer.cs L28: hitWindowDepth = pushback ? (float)HIT_WINDOW * ar / 1000 : 0
    #[inline]
    pub fn hit_window_depth(&self) -> f32 {
        use crate::map::types::HIT_WINDOW_MS;
        if self.pushback {
            HIT_WINDOW_MS as f32 * self.approach_rate / 1000.0
        } else {
            0.0
        }
    }
}

/// État global d'une session de jeu — traduction de Attempt.cs
/// Ressource Bevy insérée dans OnEnter(InGame) et retirée dans OnExit(InGame)
#[derive(Resource)]
pub struct AttemptState {
    /// Position audio en millisecondes — synchronisée depuis AudioClock chaque frame
    /// = LegacyRunner.CurrentAttempt.Progress dans le code original
    pub progress_ms: f64,

    /// Santé du joueur [0.0, 100.0] — default: 100
    /// = Attempt.Health
    pub health: f64,

    /// Pas de santé dynamique — escalade sur les misses
    /// = HealthJudgment.HealthStep (default: 15, max: 100)
    pub health_step: f64,

    /// Score cumulé
    pub score: i64,

    /// Combo courant (reset sur miss)
    pub combo: u32,

    /// Meilleur combo de cette session
    pub best_combo: u32,

    /// Multiplicateur de vitesse [0.5, 2.0]
    /// = Attempt.Speed
    pub speed: f32,

    /// Index de la prochaine note à spawner (optimization: on avance toujours)
    pub next_spawn_index: usize,

    /// Nombre de hits réussis
    pub hits: u32,

    /// Nombre de misses
    pub misses: u32,

    /// La partie est-elle terminée (santé = 0) ?
    pub failed: bool,

    /// La partie est-elle en pause ?
    pub paused: bool,

    /// Millisecondes restantes avant le démarrage de l'audio
    /// Commence à 3000.0, descend vers 0.0 — progress_ms = -pregame_remaining_ms pendant ce temps
    pub pregame_remaining_ms: f64,

    /// L'audio Kira a-t-il été démarré ?
    pub audio_started: bool,
}

impl Default for AttemptState {
    fn default() -> Self {
        Self {
            progress_ms: -3000.0,   // Début 3 secondes avant le beat zéro
            health: 100.0,
            health_step: 15.0,
            score: 0,
            combo: 0,
            best_combo: 0,
            speed: 1.0,
            next_spawn_index: 0,
            hits: 0,
            misses: 0,
            failed: false,
            paused: false,
            pregame_remaining_ms: 3000.0,
            audio_started: false,
        }
    }
}

impl AttemptState {
    /// Accuracy en pourcentage [0.0, 100.0]
    pub fn accuracy(&self) -> f64 {
        let total = (self.hits + self.misses) as f64;
        if total == 0.0 { 100.0 } else { self.hits as f64 / total * 100.0 }
    }
}
