use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

use bevy::prelude::*;
use bevy_flycam::prelude::*;
use bevy_text3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, NoCameraPlayerPlugin, Text3dPlugin))
        .insert_resource(Msaa::Sample8)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_things)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        FlyCam,
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.5, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));

    commands
        .spawn((
            Name::new("Plane"),
            PbrBundle {
                mesh: meshes.add(shape::Plane::from_size(250.0).into()),
                material: materials.add(Color::BLACK.into()),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_x(0.4253503)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                // Name::new("Text"),
                Text3dBundle {
                    transform: {
                        let mut transform = Transform::from_xyz(0.0, 0.1, 0.0);
                        transform.rotate_x(-FRAC_PI_2);
                        transform
                    },
                    text: Text::from_section(
                        TEXT,
                        TextStyle {
                            font: asset_server.load("fonts/Fira_Mono-Bold.ttf"),
                            font_size: 10.0,
                            color: Color::YELLOW,
                        },
                    )
                    .into(),
                    ..default()
                },
            ));
        });
}

fn slide_text(time: Res<Time>, mut query: Query<(&Name, &mut Transform)>) {
    let dt = time.delta_seconds();
    for (_, mut transform) in query.iter_mut() {
        transform.translation.y += 0.1;
    }
}

#[derive(Component)]
struct Sliding;

fn rotate_things(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Name, &mut Transform)>,
) {
    for (name, mut transform) in query.iter_mut() {
        // if name.as_str() == "Text" {
        //     transform.rotate_x(time.delta_seconds() * 2.0);
        // } else if name.as_str() == "Plane" {
        //     transform.rotate_x(time.delta_seconds());
        // }
        if name.as_str() == "Plane" {
            let dt = time.delta_seconds();
            if input.pressed(KeyCode::Up) {
                transform.rotate_x(-dt);
                dbg!(transform.rotation.to_axis_angle());
            } else if input.pressed(KeyCode::Down) {
                transform.rotate_x(dt);
                dbg!(transform.rotation.to_axis_angle());
            }
        }
    }
}

const TEXT: &str = "It is a period of civil wars in the 
galaxy. A brave alliance of underground 
freedom fighters has challenged the 
tyranny and oppression of the awesome 
GALACTIC EMPIRE.

Striking from a fortress hidden among the 
billion stars of the galaxy, rebel 
spaceships have won their first victory 
in a battle with the powerful Imperial 
Starfleet. The EMPIRE fears that another 
defeat could bring a thousand more solar 
systems into the rebellion, and Imperial 
control over the galaxy would be lost 
forever.

To crush the rebellion once and for all, 
the EMPIRE is constructing a sinister new 
battle station. Powerful enough to 
destroy an entire planet, its completion 
spells certain doom for the champions of 
freedom.";
