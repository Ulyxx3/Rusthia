// src/map/types.rs
// ==============================================================================
// Rusthia — Structures de données des maps
// Traduction de Map.cs, Note.cs, Constants.cs
// ==============================================================================

use serde::{Deserialize, Serialize};

// ==============================================================================
// CONSTANTES — depuis Constants.cs
// ==============================================================================

/// Fenêtre de frappe en millisecondes (Constants.HIT_WINDOW = 55)
/// Toute note > 55ms dans le passé est comptée comme MISS
pub const HIT_WINDOW_MS: f64 = 55.0;

/// Taille de la hitbox dans la grille normalisée [-1, 1]
/// 0.18 = ~36% de l'espace entre deux notes — adapté pour l'auto-hit sans clic
pub const HIT_BOX_SIZE: f32 = 0.18;

/// Taille de la grille en unités monde (Constants.GRID_SIZE = 3.0)
pub const GRID_SIZE: f32 = 3.0;

/// Position Z de la caméra — plus loin = plus de marges autour de la grille (CAMERA_Z = 6.5)
pub const CAMERA_Z: f32 = 6.5;

/// Taille du curseur (Constants.CURSOR_SIZE = 0.2625)
pub const CURSOR_SIZE: f32 = 0.2625;

/// Noms des difficultés (Constants.DIFFICULTIES)
pub const DIFFICULTIES: [&str; 6] = ["N/A", "Easy", "Medium", "Hard", "Insane", "Illogical"];

// ==============================================================================
// STRUCTURES DE DONNÉES
// ==============================================================================

/// Une note de la map — traduction de Note.cs
/// X et Y sont normalisés dans [-1.0, 1.0]
/// (0,0) = centre de la grille
#[derive(Debug, Clone, PartialEq)]
pub struct NoteData {
    /// Index de la note dans la map (pour la couleur cyclique)
    pub index: usize,
    /// Timestamp en millisecondes depuis le début de la chanson
    pub millisecond: u32,
    /// Position X sur la grille : -1.0 = gauche, 0.0 = centre, 1.0 = droite
    pub x: f32,
    /// Position Y sur la grille : -1.0 = bas, 0.0 = centre, 1.0 = haut
    pub y: f32,
}

/// Une map chargée et parsée — traduction de Map.cs
#[derive(Debug, Clone)]
pub struct MapData {
    /// Identifiant unique (Map.Id / Map.Name)
    pub id: String,
    /// Titre de la chanson
    pub title: String,
    /// Artiste
    pub artist: String,
    /// Liste des mappeurs
    pub mappers: Vec<String>,
    /// Niveau de difficulté [0..5] (0=N/A, 1=Easy...5=Illogical)
    pub difficulty: u8,
    /// Nom personnalisé de la difficulté (ex: "Expert+")
    pub difficulty_name: String,
    /// Durée totale de la map en millisecondes
    pub length_ms: u32,
    /// Buffer audio brut (MP3 ou OGG) — à passer directement à Kira
    pub audio: Vec<u8>,
    /// Buffer de la cover (PNG) — optionnel
    pub cover: Vec<u8>,
    /// Extension de l'audio ("mp3" ou "ogg")
    pub audio_ext: String,
    /// Notes triées par timestamp croissant
    pub notes: Vec<NoteData>,
}

impl MapData {
    /// Titre formatté (Artiste - Titre) comme Map.PrettyTitle
    pub fn pretty_title(&self) -> String {
        if self.artist.is_empty() {
            self.title.clone()
        } else {
            format!("{} - {}", self.artist, self.title)
        }
    }

    /// Mappers formattés séparés par des virgules
    pub fn pretty_mappers(&self) -> String {
        self.mappers.join(", ")
    }

    /// Vérifier si l'audio est au format OGG (depuis la signature "OggS")
    pub fn detect_audio_ext(bytes: &[u8]) -> &'static str {
        if bytes.len() >= 4 && &bytes[0..4] == b"OggS" {
            "ogg"
        } else {
            "mp3"
        }
    }
}

/// Métadonnées JSON du format PHXM (metadata.json dans le ZIP)
/// Équivalent de ce que `Map.EncodeMeta()` produit
#[derive(Deserialize, Serialize, Debug)]
pub struct PhxmMetadata {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Artist")]
    pub artist: String,
    #[serde(rename = "Mappers")]
    pub mappers: Vec<String>,
    #[serde(rename = "Difficulty")]
    pub difficulty: u8,
    #[serde(rename = "DifficultyName")]
    pub difficulty_name: String,
    #[serde(rename = "Length")]
    pub length: u32,
    #[serde(rename = "HasAudio")]
    pub has_audio: bool,
    #[serde(rename = "HasCover")]
    pub has_cover: bool,
    #[serde(rename = "HasVideo")]
    pub has_video: bool,
    #[serde(rename = "AudioExt")]
    pub audio_ext: String,
}
