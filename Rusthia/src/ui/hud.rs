// src/ui/hud.rs
// ==============================================================================
// Rusthia — HUD de jeu (InGame)
// Santé, combo, score, accuracy
// ==============================================================================

use bevy::prelude::*;
use crate::{GameState, game::attempt::AttemptState};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), setup_hud)
            .add_systems(OnExit(GameState::InGame),  cleanup_hud)
            .add_systems(
                Update,
                update_hud.run_if(in_state(GameState::InGame)),
            );
    }
}

// ==============================================================================
// COMPOSANTS
// ==============================================================================

#[derive(Component)] struct HudRoot;
#[derive(Component)] struct HealthFill;
#[derive(Component)] struct ComboText;
#[derive(Component)] struct ScoreText;
#[derive(Component)] struct AccuracyText;
#[derive(Component)] struct ProgressText;

// ==============================================================================
// SETUP
// ==============================================================================

fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
        ))
        .with_children(|parent| {

            // ---- HAUT : Score + Combo ----
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
                })
                .with_children(|row| {
                    // Score (gauche)
                    row.spawn((
                        ScoreText,
                        Text::new("0"),
                        TextFont { font_size: 28.0, ..default() },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                    ));

                    // Accuracy (centre)
                    row.spawn((
                        AccuracyText,
                        Text::new("100.00%"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgba(0.8, 0.8, 1.0, 0.7)),
                    ));

                    // Combo (droite)
                    row.spawn((
                        ComboText,
                        Text::new(""),
                        TextFont { font_size: 28.0, ..default() },
                        TextColor(Color::srgb(0.0, 0.85, 1.0)),
                    ));
                });

            // ---- BAS : Barre de santé ----
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|bottom| {
                    // Progress audio
                    bottom.spawn((
                        ProgressText,
                        Text::new("0:00"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgba(0.5, 0.5, 0.7, 0.6)),
                    ));
                });
        });
}

fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ==============================================================================
// MISE À JOUR
// ==============================================================================

fn update_hud(
    attempt: Res<AttemptState>,
    mut score_q:    Query<&mut Text, (With<ScoreText>, Without<ComboText>, Without<AccuracyText>, Without<ProgressText>)>,
    mut combo_q:    Query<&mut Text, (With<ComboText>, Without<ScoreText>, Without<AccuracyText>, Without<ProgressText>)>,
    mut acc_q:      Query<&mut Text, (With<AccuracyText>, Without<ScoreText>, Without<ComboText>, Without<ProgressText>)>,
    mut progress_q: Query<&mut Text, (With<ProgressText>, Without<ScoreText>, Without<ComboText>, Without<AccuracyText>)>,
) {
    // Score
    for mut t in score_q.iter_mut() {
        **t = format_score(attempt.score);
    }

    // Combo
    for mut t in combo_q.iter_mut() {
        **t = if attempt.combo >= 2 {
            format!("{}x", attempt.combo)
        } else {
            String::new()
        };
    }

    // Accuracy
    for mut t in acc_q.iter_mut() {
        **t = format!("{:.2}%", attempt.accuracy());
    }

    for mut t in progress_q.iter_mut() {
        let total_secs = (attempt.progress_ms / 1000.0) as u32;
        **t = format!("{}:{:02}", total_secs / 60, total_secs % 60);
    }
}

/// Formater un score avec séparateurs pour la lisibilité (ex: 1,234,567)
fn format_score(score: i64) -> String {
    let s = score.abs().to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.push(','); }
        result.push(c);
    }
    if score < 0 { result.push('-'); }
    result.chars().rev().collect()
}
