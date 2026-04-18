// src/game/save.rs
// ==============================================================================
// Rusthia — Système de Sauvegarde (Scores)
// Gestion simplifiée pour stocker les High Scores et Max Combos par map
// ==============================================================================

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SAVE_FILE_PATH: &str = "save.json";

/// Structure persistée dans le fichier save.json
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveData {
    /// Associe un Map ID (nom du fichier ou hash interne) à son record
    pub scores: HashMap<String, MapRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MapRecord {
    pub best_score: i64,
    pub best_combo: u32,
    pub best_accuracy: f64,
    /// Mods utilisés lors du meilleur score (ex: "NF • 1.5x"), vide si aucun
    #[serde(default)]
    pub best_score_mods: String,
}

/// Ressource permettant de gérer la sauvegarde en mémoire
#[derive(Resource, Default)]
pub struct SaveManager {
    pub data: SaveData,
}

impl SaveManager {
    /// Charge les données depuis le disque
    pub fn load() -> Self {
        if Path::new(SAVE_FILE_PATH).exists() {
            if let Ok(file_content) = fs::read_to_string(SAVE_FILE_PATH) {
                if let Ok(data) = serde_json::from_str(&file_content) {
                    return Self { data };
                }
            }
        }
        Self::default() // Fichier inexistant ou corrompu -> nouveau
    }

    /// Écrit les données sur le disque
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(SAVE_FILE_PATH, json);
        }
    }

    /// Enregistrer un record potentiel pour une map donnee
    pub fn submit_result(&mut self, map_id: &str, score: i64, combo: u32, accuracy: f64, mods_label: &str) {
        let entry = self.data.scores.entry(map_id.to_string()).or_default();
        let mut updated = false;

        if score > entry.best_score {
            entry.best_score = score;
            entry.best_accuracy = accuracy;
            entry.best_score_mods = mods_label.to_string();
            updated = true;
        }
        if combo > entry.best_combo {
            entry.best_combo = combo;
            updated = true;
        }

        // Sauvegarder sur disque immédiat si qqch a été battu
        if updated {
            self.save();
        }
    }

    /// Obtenir le record d'une map s'il existe
    pub fn get_record(&self, map_id: &str) -> Option<MapRecord> {
        self.data.scores.get(map_id).cloned()
    }
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        // Chargement immédiat au démarrage du jeu
        app.insert_resource(SaveManager::load());
    }
}
