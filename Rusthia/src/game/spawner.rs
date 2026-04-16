// src/game/spawner.rs
// ==============================================================================
// Rusthia — Note Spawner & Renderer
// Traduction de LegacyRenderer.cs + NoteRenderer.cs
//
// Algorithme clé (LegacyRenderer.cs L46-68) :
//   at    = approach_distance / approach_rate          (temps d'approche en sec)
//   depth = (note.ms - progress_ms) / (1000 * at) * ad / speed
//   alpha = clamp((1 - depth/ad) / (fadeIn/100), 0, 1)
//   transform.Origin = Vector3(note.X, note.Y, -depth)
// ==============================================================================

use bevy::prelude::*;
use crate::{
    audio::AudioClock,
    game::{
        attempt::{AttemptState, GameSettings},
        judgments::GameCursor,
    },
    map::{ActiveMap, types::{GRID_SIZE, HIT_WINDOW_MS}},
};

// ==============================================================================
// COMPOSANTS ECS
// ==============================================================================

/// Marqueur sur les entités Note
#[derive(Component)]
pub struct NoteMarker;

/// Données de jeu d'une note
#[derive(Component)]
pub struct NoteComponent {
    pub note_index: usize,
    pub millisecond: u32,
    pub grid_x: f32,  // [-1, 1] normalisé
    pub grid_y: f32,  // [-1, 1] normalisé
    pub is_hit: bool,
    pub is_hittable: bool,
}

/// Marqueur du curseur visuel en jeu
#[derive(Component)]
pub struct CursorVisual;

#[derive(Component)]
pub struct HealthBar3d;

/// Marqueur le fond de grille
#[derive(Component)]
pub struct GridBackground;

/// Ressource stockant la texture générée de la note
#[derive(Resource)]
pub struct NoteTextureHandle(pub Handle<Image>);

// ==============================================================================
// COULEURS DES NOTES
// ==============================================================================

const NOTE_COLORS: &[Color] = &[
    Color::srgb(0.0, 0.8, 1.0),   // cyan neon
    Color::srgb(1.0, 0.2, 0.8),   // rose neon
    Color::srgb(0.4, 1.0, 0.4),   // vert neon
    Color::srgb(1.0, 0.8, 0.0),   // or neon
    Color::srgb(0.8, 0.2, 1.0),   // violet neon
];

const BLOOM_INTENSITY: f32 = 3.0;

// ==============================================================================
// EVENTS
// ==============================================================================

#[derive(Event, Debug)]
pub struct NoteHitEvent {
    pub note_index: usize,
    pub delta_ms: f64,
}

#[derive(Event, Debug)]
pub struct NoteMissEvent {
    pub note_index: usize,
}

// ==============================================================================
// SETUP
// ==============================================================================

/// Setup de la scène de jeu — caméra + grille + curseur visuel
pub fn setup_game_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    settings: Res<GameSettings>,
) {
    // --- Caméra avec Bloom ---
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, crate::map::types::CAMERA_Z)
            .looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            hdr: true,
            ..default()
        },
        bevy::core_pipeline::bloom::Bloom {
            intensity: 0.3,
            ..default()
        },
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
    ));

    // --- Lumière ambiante neon sombre ---
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.03, 0.03, 0.1),
        brightness: 30.0,
        ..default()
    });

    let half = GRID_SIZE / 2.0;

    // --- Fond de la grille (doit être transparent pour voir les notes arriver !) ---
    let grid_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.02, 0.02, 0.04, 0.1), // très sombre et très transparent
        emissive: LinearRgba::new(0.02, 0.02, 0.05, 1.0), // très légère lueur bleue pour deviner le cadre
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(GRID_SIZE, GRID_SIZE))),
        MeshMaterial3d(grid_mat),
        Transform::from_xyz(0.0, 0.0, -0.02),
        GridBackground,
    ));

    // --- Lignes de contour du cadre (Neon Bleu/Cyan) ---
    let line_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.5, 1.0, 0.8),
        emissive: LinearRgba::new(0.0, 0.8, 2.0, 1.0),
        unlit: true,
        ..default()
    });
    let thickness = 0.03;
    let size = GRID_SIZE;
    
    // Top
    commands.spawn((Mesh3d(meshes.add(Rectangle::new(size, thickness))), MeshMaterial3d(line_mat.clone()), Transform::from_xyz(0.0, half, -0.01)));
    // Bottom
    commands.spawn((Mesh3d(meshes.add(Rectangle::new(size, thickness))), MeshMaterial3d(line_mat.clone()), Transform::from_xyz(0.0, -half, -0.01)));
    // Left
    commands.spawn((Mesh3d(meshes.add(Rectangle::new(thickness, size))), MeshMaterial3d(line_mat.clone()), Transform::from_xyz(-half, 0.0, -0.01)));
    // Right
    commands.spawn((Mesh3d(meshes.add(Rectangle::new(thickness, size))), MeshMaterial3d(line_mat.clone()), Transform::from_xyz(half, 0.0, -0.01)));

    // --- Barre de santé 3D au bas du cadre ---
    let hp_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 1.0, 0.4),
        emissive: LinearRgba::new(0.2, 1.0, 0.4, 2.0),
        unlit: true,
        ..default()
    });
    commands.spawn((
        HealthBar3d,
        Mesh3d(meshes.add(Rectangle::new(1.0, thickness * 2.0))), // Base width 1.0, scaled by Transform
        MeshMaterial3d(hp_mat),
        Transform::from_xyz(0.0, -half - thickness * 2.0, -0.01)
            .with_scale(Vec3::new(GRID_SIZE, 1.0, 1.0)),
    ));

    // =========================================================================
    // CURSEUR VISUEL
    // Carré neon blanc/cyan visible sur le plan de frappe (Z=0)
    // Correspond à CursorPosition dans le code original Rhythia
    // =========================================================================
    let cursor_size = settings.cursor_size * half; // au lieu du constant CURSOR_SIZE
    let cursor_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.95),
        // Émission HDR pour le bloom — curseur bien visible
        emissive: LinearRgba::new(2.0, 2.5, 3.0, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        CursorVisual,
        Mesh3d(meshes.add(Rectangle::new(cursor_size, cursor_size))),
        MeshMaterial3d(cursor_mat),
        Transform::from_xyz(0.0, 0.0, 0.05), // légèrement devant le plan Z=0
    ));

    // --- Génération de la texture de Note (Squircle/Cercle) ---
    let shape = settings.note_shape;
    let note_tex_handle = {
        let size = 64;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                let cx = (x as i32 - size as i32 / 2) as f32 / (size as f32 / 2.0);
                let cy = (y as i32 - size as i32 / 2) as f32 / (size as f32 / 2.0);

                let mut border = false;

                if shape == 0 {
                    // Carré à bordure
                    let dx = cx.abs();
                    let dy = cy.abs();
                    // On affiche le contour si on est près des bords
                    if dx < 1.0 && dy < 1.0 && (dx > 0.80 || dy > 0.80) { border = true; }
                } else if shape == 1 {
                    // Squircle approx: x^4 + y^4 = 1
                    let d = cx.powi(4) + cy.powi(4);
                    if d > 0.65 && d < 1.0 { border = true; }
                } else {
                    // Cercle: x^2 + y^2 = 1
                    let d = cx.powi(2) + cy.powi(2);
                    if d > 0.75 && d < 1.0 { border = true; }
                }

                if border {
                    // Contour opaque blanc
                    data[idx] = 255; data[idx+1] = 255; data[idx+2] = 255; data[idx+3] = 255;
                } else {
                    // Intérieur et extérieur completement transparents
                    data[idx] = 0; data[idx+1] = 0; data[idx+2] = 0; data[idx+3] = 0;
                }
            }
        }

        let image = Image::new(
            bevy::render::render_resource::Extent3d { width: size as u32, height: size as u32, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2,
            data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::default(),
        );
        images.add(image)
    };

    commands.insert_resource(NoteTextureHandle(note_tex_handle));
}

/// Nettoyage de la scène de jeu
pub fn cleanup_game_scene(
    mut commands: Commands,
    query: Query<Entity, Or<(With<NoteMarker>, With<Camera3d>, With<GridBackground>, With<CursorVisual>, With<HealthBar3d>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn update_health_bar_3d(
    attempt: Res<AttemptState>,
    mut q_hp: Query<(&mut Transform, &MeshMaterial3d<StandardMaterial>), With<HealthBar3d>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut transform, mat_handle) in q_hp.iter_mut() {
        let h = attempt.health as f32 / 100.0;
        // Largeur varie de 0.0 à GRID_SIZE
        transform.scale.x = (h * GRID_SIZE).max(0.001);
        
        let color = if h > 0.5 {
            Color::srgb((1.0 - h) * 2.0, 1.0, 0.2)
        } else {
            Color::srgb(1.0, h * 2.0, 0.1)
        };
        
        if let Some(mat) = materials.get_mut(mat_handle) {
            mat.base_color = color;
            mat.emissive = LinearRgba::from(color) * 2.0;
        }
    }
}

// ==============================================================================
// SYSTÈMES
// ==============================================================================

pub fn update_camera_parallax(
    cursor: Res<GameCursor>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    settings: Res<GameSettings>,
) {
    let parallax_strength = settings.parallax_strength;
    for mut transform in query.iter_mut() {
        // La caméra se déplace dans le même sens que le pointeur
        // Cela donne l'impression que la grille bouge à l'opposé du curseur.
        // ex: cursor = 1.0 (droite), caméra va à droite (+0.15), donc l'environnement
        // au centre recule vers la gauche.
        transform.translation.x = cursor.x * parallax_strength;
        transform.translation.y = cursor.y * parallax_strength;
    }
}

/// Mettre à jour la position 3D du curseur visuel chaque frame.
/// Le curseur suit les mouvements de souris en temps réel à Z=0 (plan de frappe).
pub fn update_cursor_visual(
    cursor: Res<GameCursor>,
    mut query: Query<&mut Transform, With<CursorVisual>>,
) {
    let half = GRID_SIZE / 2.0;
    for mut transform in query.iter_mut() {
        transform.translation.x = cursor.x * half;
        transform.translation.y = cursor.y * half;
        // Z fixe légèrement devant les notes au moment du hit (depth=0)
        transform.translation.z = 0.05;
    }
}

/// Spawner les nouvelles notes selon la progression audio.
/// Gère aussi le countdown pré-jeu (progress_ms négatif pendant 3 secondes).
pub fn spawn_notes(
    mut commands: Commands,
    mut attempt: ResMut<AttemptState>,
    clock: Res<AudioClock>,
    settings: Res<GameSettings>,
    map_res: Option<Res<ActiveMap>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    note_texture: Option<Res<NoteTextureHandle>>,
    time: Res<Time>,
) {
    // =========================================================================
    // SYNCHRONISATION TEMPORELLE
    // Pendant le pregame: countdown manuel
    // Après: sync depuis AudioClock (précision sample Kira)
    // =========================================================================
    if !attempt.audio_started {
        // Décrémenter le countdown avec le delta temps Bevy
        let dt_ms = time.delta_secs_f64() * 1000.0;
        attempt.pregame_remaining_ms = (attempt.pregame_remaining_ms - dt_ms).max(0.0);
        attempt.progress_ms = -attempt.pregame_remaining_ms;
    } else {
        // Audio en cours — sync depuis le handle Kira
        attempt.progress_ms = clock.position_ms;
    }

    let Some(active_map) = map_res else { return };
    let map = &active_map.0;

    let at = settings.approach_time_secs();
    let approach_window_ms = at as f64 * 1000.0;

    while attempt.next_spawn_index < map.notes.len() {
        let note = &map.notes[attempt.next_spawn_index];
        let time_until_ms = note.millisecond as f64 - attempt.progress_ms;

        // Trop loin dans le futur = on attend
        if time_until_ms > approach_window_ms { break; }

        // Trop loin dans le passé = skip (déjà raté)
        if time_until_ms < -(HIT_WINDOW_MS * 2.0) {
            attempt.next_spawn_index += 1;
            continue;
        }

        // Couleur cyclique
        let base_color = NOTE_COLORS[note.index % NOTE_COLORS.len()];
        let LinearRgba { red, green, blue, .. } = base_color.to_linear();

        let note_mat = materials.add(StandardMaterial {
            base_color,
            emissive: LinearRgba::new(
                red * BLOOM_INTENSITY,
                green * BLOOM_INTENSITY,
                blue * BLOOM_INTENSITY,
                1.0,
            ),
            base_color_texture: note_texture.as_ref().map(|t| t.0.clone()),
            emissive_texture: note_texture.as_ref().map(|t| t.0.clone()),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        // =====================================================================
        // TAILLE DES NOTES
        // Formule corrigée : note_size en fraction de la GRID_SIZE
        // 0.25 * 3.0 = 0.75 world units — visible et jouable
        // (ancien bug: division par 4 donnait 0.1875 — trop petit)
        // =====================================================================
        let note_size = settings.note_size * GRID_SIZE;

        commands.spawn((
            NoteMarker,
            NoteComponent {
                note_index: note.index,
                millisecond: note.millisecond,
                grid_x: note.x,
                grid_y: note.y,
                is_hit: false,
                is_hittable: true,
            },
            Mesh3d(meshes.add(Rectangle::new(note_size, note_size))),
            MeshMaterial3d(note_mat),
            Transform::default(),
        ));

        attempt.next_spawn_index += 1;
    }
}

/// Déplacer les notes selon la formule du LegacyRenderer.cs
pub fn move_notes(
    mut note_query: Query<(&NoteComponent, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    attempt: Res<AttemptState>,
    settings: Res<GameSettings>,
) {
    let at = settings.approach_time_secs();
    let ad = settings.approach_distance;
    let hit_window_depth = settings.hit_window_depth();
    let fade_out_scale = settings.fade_out / 10.0;
    let half_grid = GRID_SIZE / 2.0;

    for (note, mut transform, mat_handle) in note_query.iter_mut() {
        if note.is_hit { continue; }

        // depth = (note.ms - progress) / (1000 * at) * ad / speed
        let depth = (note.millisecond as f64 - attempt.progress_ms)
            / (1000.0 * at as f64)
            * ad as f64
            / attempt.speed as f64;

        // alpha = clamp((1 - depth/ad) / (fadeIn/100), 0, 1)
        let mut alpha = ((1.0 - depth / ad as f64) / (settings.fade_in as f64 / 100.0))
            .clamp(0.0, 1.0) as f32;

        if settings.ghost_mode {
            alpha -= ((ad as f64 - depth) / (ad as f64 / 2.0)).min(1.0) as f32;
        } else if settings.fade_out > 0.0 {
            alpha *= ((depth + hit_window_depth as f64)
                / (ad as f64 * fade_out_scale as f64 + hit_window_depth as f64))
                .min(1.0) as f32;
        }

        // Cacher les notes passées (mode non-pushback)
        if !settings.pushback && note.millisecond as f64 - attempt.progress_ms <= 0.0 {
            alpha = 0.0;
        }

        alpha = (alpha * settings.note_opacity).clamp(0.0, 1.0);

        // Position 3D — Z négatif = en face de la caméra, approche vers Z=0
        transform.translation = Vec3::new(
            note.grid_x * half_grid,
            note.grid_y * half_grid,
            -depth as f32,
        );

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let LinearRgba { red, green, blue, .. } = mat.emissive;
            mat.base_color = mat.base_color.with_alpha(alpha);
            mat.emissive = LinearRgba::new(
                red * alpha.max(0.01), // garder une légère lueur même à alpha faible
                green * alpha.max(0.01),
                blue * alpha.max(0.01),
                1.0,
            );
        }
    }
}

/// Dépawner les notes frappées ou ratées
pub fn despawn_processed_notes(
    mut commands: Commands,
    query: Query<(Entity, &NoteComponent)>,
    attempt: Res<AttemptState>,
    mut miss_events: EventWriter<NoteMissEvent>,
) {
    for (entity, note) in query.iter() {
        if note.is_hit {
            commands.entity(entity).despawn();
            continue;
        }

        // Miss: la note est passée au-delà de la fenêtre de frappe
        let delta = attempt.progress_ms - note.millisecond as f64;
        if delta > HIT_WINDOW_MS {
            miss_events.send(NoteMissEvent { note_index: note.note_index });
            commands.entity(entity).despawn();
        }
    }
}
