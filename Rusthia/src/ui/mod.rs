// src/ui/mod.rs
// ==============================================================================
// Rusthia — UI Plugin (tous les écrans)
// Gère : Loading → MainMenu → InGame HUD → Results
// ==============================================================================

mod hud;
mod main_menu;
mod results;
pub mod loading;


use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Loading (remplace go_to_main_menu)
            .add_plugins(loading::LoadingPlugin)

            // Main Menu
            .add_plugins(main_menu::MainMenuPlugin)

            // HUD de jeu
            .add_plugins(hud::HudPlugin)

            // Résultats
            .add_plugins(results::ResultsPlugin);
    }
}
