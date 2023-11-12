#![allow(clippy::too_many_arguments)]

use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_flycam::prelude::*;

mod mesh;
mod pipeline;
mod plugin;

use plugin::{Text3dBundle, Text3dPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            NoCameraPlayerPlugin,
            WireframePlugin,
            Text3dPlugin,
        ))
        .insert_resource(Msaa::Sample8)
        .insert_resource(ClearColor(Color::rgb(0.52734375, 0.8046875, 0.91796875)))
        .add_systems(Startup, setup)
        // .add_systems(Update, zoom_and_pan)
        .add_systems(Update, adjust_movement_settings)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((
        FlyCam,
        Camera3dBundle {
            transform: {
                let mut transform = Transform::from_translation(Vec3::new(0.0, 1.5, 100.0))
                    .looking_at(Vec3::ZERO, Vec3::Y);
                transform.scale = Vec2::splat(2.0).extend(1.0);
                transform
            },
            ..default()
        },
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::Rgba {
                red: 0.9,
                green: 0.9,
                blue: 0.1,
                alpha: 1.0,
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // TODO: colours don't work
    // TODO: Text3dSize
    // Plane at origin
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 10000.0,
            ..default()
        })),
        material: materials.add(Color::rgb(0.02734375, 0.1171875, 0.92578125).into()),
        ..default()
    });

    // Text at origin
    commands.spawn(Text3dBundle {
        text: Text::from_sections([
            TextSection::new(
                "Press F to toggle wireframes.\n",
                TextStyle {
                    font: asset_server.load("fonts/Dancing_Script-Medium.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.5, 0.9, 0.5),
                },
            ),
            TextSection::new(
                "Press F to toggle wireframes.\n",
                TextStyle {
                    font: asset_server.load("fonts/Airstrip_Four-Regular.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.5, 0.9, 0.5),
                },
            ),
            TextSection::new(
                "Press F to toggle wireframes.\n",
                TextStyle {
                    font: asset_server.load("fonts/Open_Sans-Italic.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.5, 0.9, 0.5),
                },
            ),
            TextSection::new(
                "Press F to toggle wireframes.\n",
                TextStyle {
                    font: asset_server.load("fonts/Fira_Mono-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.5, 0.9, 0.5),
                },
            ),
        ])
        .into(),
        ..default()
    });
}

// fn zoom_and_pan(
//     input: Res<Input<KeyCode>>,
//     time: Res<Time>,
//     mut query: Query<&mut Transform, With<Camera>>,
// ) {
//     for mut transform in query.iter_mut() {
//         if input.pressed(KeyCode::Down) {
//             transform.translation.z += transform.translation.z * time.delta_seconds();
//         }
//         if input.pressed(KeyCode::Up) {
//             transform.translation.z -= transform.translation.z * time.delta_seconds();
//         }
//         if input.pressed(KeyCode::W) {
//             transform.translation.y += transform.translation.z * time.delta_seconds();
//         }
//         if input.pressed(KeyCode::A) {
//             transform.translation.x -= transform.translation.z * time.delta_seconds();
//         }
//         if input.pressed(KeyCode::S) {
//             transform.translation.y -= transform.translation.z * time.delta_seconds();
//         }
//         if input.pressed(KeyCode::D) {
//             transform.translation.x += transform.translation.z * time.delta_seconds();
//         }
//     }
// }

fn adjust_movement_settings(
    mut settings: ResMut<MovementSettings>,
    camera: Query<&Transform, With<FlyCam>>,
) {
    settings.speed =
        MovementSettings::default().speed + camera.single().translation.distance(Vec3::ZERO);
}
