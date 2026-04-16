// src/ui/loading.rs
// ==============================================================================
// Rusthia — Écran de Chargement
// Initialise les ressources et parse le dossier "maps/" au démarrage
// ==============================================================================

use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;

use crate::{GameState, map::parser::parse_map, map::types::MapData};

#[derive(Resource, Default)]
pub struct MapDatabase {
    pub entries: Vec<MapListEntry>,
}

pub struct MapListEntry {
    pub path: PathBuf,
    pub data: MapData,
}

pub struct LoadingPlugin;

#[derive(Component)]
struct LoadingScreen;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MapDatabase::default())
            .add_systems(OnEnter(GameState::Loading), start_loading)
            .add_systems(Update, process_loading.run_if(in_state(GameState::Loading)))
            .add_systems(OnExit(GameState::Loading), cleanup_loading_ui);
    }
}

fn start_loading(mut commands: Commands) {
    // Créer le répertoire s'il n'existe pas
    let _ = fs::create_dir_all("maps");

    commands.spawn((
        Camera2d::default(),
        LoadingScreen,
    ));

    commands.spawn((
        LoadingScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        }
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Recherche des maps..."),
            TextFont { font_size: 40.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.9)),
        ));
    });
}

fn process_loading(
    mut db: ResMut<MapDatabase>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Dans un vrai jeu commercial, ceci serait asynchrone pour ne pas geler
    // l'écran de chargement. Pour Rusthia, la rapidité de Rust suffit pour
    // quelques dizaines de maps stockées en local au démarrage.
    let root = PathBuf::from("maps");

    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext = ext.to_lowercase();
                    if matches!(ext.as_str(), "phxm" | "sspm" | "txt") {
                        if let Ok(bytes) = fs::read(&path) {
                            let filename = path.file_name().unwrap().to_string_lossy().to_string();
                            match parse_map(&bytes, &filename) {
                                Ok(mut map_data) => {
                                    // Optionnel : on pourrait vider map_data.audio ici si on voulait
                                    // l'optimiser, et le relire lors de la sélection.
                                    // Mais pour ce prototype, on le garde en mémoire pour la fluidité.
                                    db.entries.push(MapListEntry { path, data: map_data });
                                }
                                Err(e) => {
                                    error!("Erreur de chargement pour {}: {}", filename, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Chargement terminé : {} maps trouvées", db.entries.len());
    
    // Sortir rapidement vers le Song Select (MainMenu)
    next_state.set(GameState::MainMenu);
}

fn cleanup_loading_ui(mut commands: Commands, query: Query<Entity, With<LoadingScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
