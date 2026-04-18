// src/ui/results.rs
// ==============================================================================
// Rusthia — Écran de résultats (après fin de jeu ou fail)
// Traduction de SceneResults dans l'original
// ==============================================================================

use bevy::prelude::*;
use crate::{GameState, game::attempt::AttemptState};

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Results), setup_results)
            .add_systems(OnExit(GameState::Results),  cleanup_results)
            .add_systems(
                Update,
                handle_results_input.run_if(in_state(GameState::Results)),
            );
    }
}

#[derive(Component)]
struct ResultsRoot;

fn setup_results(
    mut commands: Commands,
    attempt: Res<AttemptState>,
) {
    let accuracy = attempt.accuracy();
    let grade = match accuracy as u32 {
        100       => ("SS", Color::srgb(0.5, 1.0, 1.0)),
        95..=99   => ("S",  Color::srgb(1.0,  0.85, 0.0)),
        88..=94   => ("A",  Color::srgb(0.2,  1.0,  0.3)),
        75..=87   => ("B",  Color::srgb(0.2,  0.5,  1.0)),
        60..=74   => ("C",  Color::srgb(0.8,  0.6,  0.0)),
        _         => ("F",  Color::srgb(1.0,  0.1,  0.1)),
    };
    let mods_label = attempt.mods_label.clone();
    let failed = attempt.failed;

    commands.spawn((Camera2d::default(), ResultsRoot));

    commands
        .spawn((
            ResultsRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.02, 0.06)),
        ))
        .with_children(|parent| {

            // Grade
            parent.spawn((
                Text::new(grade.0),
                TextFont { font_size: 120.0, ..default() },
                TextColor(grade.1),
            ));

            // Score
            parent.spawn((
                Text::new(format!("Score : {}", attempt.score)),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::WHITE),
            ));

            // Modificateurs actifs (affiché seulement si mods non vides)
            if !mods_label.is_empty() {
                parent.spawn((
                    Text::new(format!("Mods : {}", mods_label)),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.65, 0.0)),
                ));
            }

            // Indicateur de fail en mode No Fail
            if failed {
                parent.spawn((
                    Text::new("FAIL"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.2, 0.2)),
                ));
            }

            // Stats en ligne
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(40.0),
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|row| {
                    stat_block(row, "Accuracy",    &format!("{:.2}%", accuracy));
                    stat_block(row, "Max Combo",   &format!("{}x", attempt.best_combo));
                    stat_block(row, "Hits",        &format!("{}", attempt.hits));
                    stat_block(row, "Misses",      &format!("{}", attempt.misses));
                });

            // Séparateur
            parent.spawn((
                Node { width: Val::Px(400.0), height: Val::Px(1.0), ..default() },
                BackgroundColor(Color::srgba(0.3, 0.3, 0.5, 0.4)),
            ));

            // Instructions
            parent.spawn((
                Text::new("[ Entrée ] Rejouer    [ Échap ] Menu"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgba(0.6, 0.6, 0.8, 0.7)),
            ));
        });
}

/// Afficher un bloc statistique (titre + valeur empilés)
fn stat_block(parent: &mut ChildBuilder, label: &str, value: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|block| {
            block.spawn((
                Text::new(value.to_string()),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::WHITE),
            ));
            block.spawn((
                Text::new(label.to_string()),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgba(0.5, 0.5, 0.7, 0.7)),
            ));
        });
}

fn cleanup_results(
    mut commands: Commands,
    query: Query<Entity, With<ResultsRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_results_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    map_res: Option<Res<crate::map::ActiveMap>>,
    mut attempt: ResMut<AttemptState>,
) {
    // Entrée = rejouer la même map
    if keyboard.just_pressed(KeyCode::Enter) && map_res.is_some() {
        *attempt = AttemptState::default();
        next_state.set(GameState::InGame);
    }

    // Échap = retour au menu
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}
