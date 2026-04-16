// src/map/mod.rs
// ==============================================================================
// Rusthia — Plugin Map
// Gère le chargement et le stockage de la map active
// ==============================================================================

pub mod parser;
pub mod types;

use bevy::prelude::*;
use types::MapData;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadMapEvent>()
            .add_event::<MapLoadedEvent>()
            .add_systems(Update, handle_load_map_event);
    }
}

// ==============================================================================
// RESSOURCES
// ==============================================================================

/// La map chargée et prête à jouer — ressource globale
/// Insérée quand une map est chargée avec succès
#[derive(Resource)]
pub struct ActiveMap(pub MapData);

// ==============================================================================
// ÉVÉNEMENTS
// ==============================================================================

/// Demander le chargement d'une map depuis des bytes (ex: file drag-and-drop)
#[derive(Event)]
pub struct LoadMapEvent {
    pub bytes: Vec<u8>,
    pub filename: String,
}

/// Émis quand la map a été parsée avec succès
#[derive(Event)]
pub struct MapLoadedEvent;

// ==============================================================================
// SYSTÈMES
// ==============================================================================

/// System: charger une map quand un LoadMapEvent est reçu
fn handle_load_map_event(
    mut events: EventReader<LoadMapEvent>,
    mut commands: Commands,
    mut loaded_events: EventWriter<MapLoadedEvent>,
) {
    for event in events.read() {
        match parser::parse_map(&event.bytes, &event.filename) {
            Ok(map_data) => {
                info!(
                    "Map chargée: {} ({} notes)",
                    map_data.pretty_title(),
                    map_data.notes.len()
                );
                commands.insert_resource(ActiveMap(map_data));
                loaded_events.send(MapLoadedEvent);
            }
            Err(e) => {
                error!("Erreur de parsing de la map '{}': {}", event.filename, e);
            }
        }
    }
}
