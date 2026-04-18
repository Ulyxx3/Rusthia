// src/audio/mod.rs
// ==============================================================================
// Rusthia — Plugin Audio (Kira)
// Traduction de SoundManager.cs (Godot AudioStreamPlayer) vers Kira
//
// Équivalences principales :
//   SoundManager.Song          → KiraManager.song_handle
//   Song.GetPlaybackPosition() → handle.position() (en secondes)
//   Song.Play()                → manager.play(StaticSoundData)
//   Song.VolumeDb              → handle.set_volume(Value, Tween)
// ==============================================================================

use bevy::prelude::*;
use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend,
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    Tween,
};
use std::io::Cursor;

/// Plugin principal audio — s'enregistre dans App::new()
pub struct RusthiaAudioPlugin;

impl Plugin for RusthiaAudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KiraManager::new())
            .insert_resource(AudioClock::default())
            .add_systems(Update, sync_audio_clock);
    }
}

// ==============================================================================
// RESSOURCES
// ==============================================================================

/// Gestionnaire audio Kira — équivalent de SoundManager.cs (singleton)
/// Détient le AudioManager principal et le handle de la chanson en cours
#[derive(Resource)]
pub struct KiraManager {
    /// Le manager Kira — gère le thread audio en arrière-plan
    pub manager: AudioManager<DefaultBackend>,
    /// Handle vers la piste audio principale (= SoundManager.Song)
    pub song_handle: Option<StaticSoundHandle>,
    /// Volume maître [0.0, 1.0]
    pub volume_master: f64,
    /// Volume de la musique [0.0, 1.0]
    pub volume_music: f64,
    /// Volume SFX [0.0, 1.0]
    pub volume_sfx: f64,
}

impl KiraManager {
    pub fn new() -> Self {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .expect("Impossible d'initialiser le backend audio Kira.\nVérifiez que vos pilotes audio sont fonctionnels.");

        Self {
            manager,
            song_handle: None,
            volume_master: 1.0,
            volume_music: 1.0,
            volume_sfx: 1.0,
        }
    }

    /// Charger et jouer un fichier audio depuis un buffer en mémoire.
    /// Équivalent de `SoundManager.PlayJukebox(map)` avec `Song.Stream = ...`
    ///
    /// # Arguments
    /// * `audio_bytes` — Buffer audio brut (MP3 ou OGG)
    pub fn play_from_bytes(&mut self, audio_bytes: &[u8]) {
        // Arrêter la chanson précédente si elle tourne
        if let Some(mut handle) = self.song_handle.take() {
            let _ = handle.stop(Tween::default());
        }

        let cursor = Cursor::new(audio_bytes.to_vec());
        let sound_data = StaticSoundData::from_cursor(cursor)
            .expect("Impossible de décoder l'audio. Format supporté : MP3, OGG, WAV");

        // Kira 0.12 accepte f32 directement via From<f32>::into() pour le volume
        // Un volume de 1.0 = amplitude nominale (0 dB)
        let settings = StaticSoundSettings::default()
            .volume(self.compute_volume_linear() as f32);

        let handle = self.manager.play(sound_data.with_settings(settings))
            .expect("Impossible de jouer le son : limite de sons atteinte");
        self.song_handle = Some(handle);
    }

    /// Calculer le volume linéaire combiné maître + musique.
    /// Traduction de `SoundManager.ComputeVolumeDb()` (approx. en linéaire)
    pub fn compute_volume_linear(&self) -> f64 {
        self.volume_master * self.volume_music
    }

    /// Appliquer les nouvelles valeurs de volume à la piste en cours.
    /// Équivalent de `SoundManager.UpdateVolume()`
    pub fn update_volume(&mut self) {
        // Calculer le volume AVANT d'emprunter le handle (borrow checker)
        let vol = self.compute_volume_linear() as f32;
        if let Some(ref mut handle) = self.song_handle {
            let _ = handle.set_volume(vol, Tween::default());
        }
    }

    /// Is a song currently playing?
    pub fn is_playing(&self) -> bool {
        self.song_handle.is_some()
    }

    /// Jouer un son SFX court en parallèle de la musique principale.
    /// Le handle n'est pas stocké — Kira libère le son automatiquement.
    /// Équivalent de `SoundManager.PlaySFX()` dans l'original.
    pub fn play_sfx(&mut self, audio_bytes: &[u8]) {
        let cursor = Cursor::new(audio_bytes.to_vec());
        let Ok(sound_data) = StaticSoundData::from_cursor(cursor) else {
            warn!("Impossible de décoder le SFX");
            return;
        };
        let vol = (self.volume_master * self.volume_sfx) as f32;
        let settings = StaticSoundSettings::default().volume(vol);
        // On ignore l'erreur si trop de sons simultanés
        let _ = self.manager.play(sound_data.with_settings(settings));
    }
}

/// Horloge audio précise — synchronise la position audio chaque frame.
///
/// C'est LE composant de synchronisation critique de Rusthia.
/// Équivalent de `LegacyRunner.CurrentAttempt.Progress` dans l'original.
///
/// `position_ms` est mis à jour depuis le thread Bevy via `Kira handle.position()`
/// qui retourne la position exacte du dernier sample rendu — pas une interpolation.
/// C'est la même sémantique que `AudioStreamPlayer.GetPlaybackPosition()` de Godot.
#[derive(Resource, Default, Debug)]
pub struct AudioClock {
    /// Position audio en millisecondes — mis à jour chaque frame
    /// = Song.GetPlaybackPosition() * 1000.0 dans le code original
    pub position_ms: f64,
    /// Est-ce que l'audio est en cours de lecture ?
    pub is_playing: bool,
}

// ==============================================================================
// SYSTÈMES
// ==============================================================================

/// System Bevy — synchronise l'AudioClock depuis le handle Kira.
/// S'exécute CHAQUE frame avant les systèmes de jeu.
///
/// Traduction directe de :
/// `LegacyRunner.CurrentAttempt.Progress = Song.GetPlaybackPosition() * 1000.0`
///
/// La précision de Kira est au niveau du sample (~0.02ms à 48kHz).
pub fn sync_audio_clock(
    mut clock: ResMut<AudioClock>,
    kira: Res<KiraManager>,
) {
    if let Some(ref handle) = kira.song_handle {
        // `handle.position()` retourne la position en SECONDES du dernier sample
        // rendu par le thread audio Kira — équivalent exact de GetPlaybackPosition()
        let pos_secs = handle.position();
        clock.position_ms = pos_secs * 1000.0;
        clock.is_playing = true;
    } else {
        clock.position_ms = 0.0;
        clock.is_playing = false;
    }
}

// ==============================================================================
// ÉVÉNEMENTS
// ==============================================================================

/// Événement pour lancer la lecture d'une map chargée
#[derive(Event)]
pub struct PlayMapAudio {
    /// Raw bytes de l'audio (MP3/OGG)
    pub audio_bytes: Vec<u8>,
}

/// Événement pour arrêter la musique (ex: retour au menu)
#[derive(Event)]
pub struct StopAudio;
