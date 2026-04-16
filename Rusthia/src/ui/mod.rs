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
            .add_plugins(results::ResultsPlugin)
            
            // Compteur de FPS global
            .add_systems(Startup, setup_fps_counter)
            .add_systems(Update, update_fps_counter);
    }
}

#[derive(Component)]
struct FpsText;

fn setup_fps_counter(mut commands: Commands) {
    // Le texte prendra de toute façon la police "font.ttf" grâce au système global 'apply_custom_font'
    commands.spawn((
        FpsText,
        Text::default(),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            display: Display::None,
            ..default()
        },
        ZIndex(1000), // Ultra prioritaire pour passer par-dessus les menus
    ));
}

fn update_fps_counter(
    time: Res<Time>,
    settings: Res<crate::game::attempt::GameSettings>,
    mut q_fps: Query<(&mut Text, &mut Node), With<FpsText>>,
    mut timer: Local<f32>,
    mut frames: Local<u32>,
    mut last_fps: Local<f32>,
) {
    *timer += time.delta_secs();
    *frames += 1;

    if *timer >= 0.2 {
        *last_fps = *frames as f32 / *timer;
        *timer = 0.0;
        *frames = 0;
    }

    for (mut text, mut node) in q_fps.iter_mut() {
        if settings.show_fps {
            node.display = Display::Flex;
            if *last_fps > 0.0 {
                **text = format!("FPS: {:.0}", *last_fps);
            }
        } else {
            node.display = Display::None;
        }
    }
}
