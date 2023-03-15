use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::camera::Camera,
};

use ab_glyph::FontRef;

mod mesh;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample8)
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_system(zoom_and_pan)
        .add_system(wireframe)
        .run();
}

fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // for some characters e.g. X there is no indices at all - why?
    const THE_CHAR: char = 'v';
    let the_char: char = THE_CHAR;

    let font = FontRef::try_from_slice(include_bytes!("../fonts/airstrip.ttf")).unwrap();
    let glyph_mesh = mesh::build_mesh(font, the_char).unwrap();

    let mesh = meshes.add(glyph_mesh.mesh);

    let material = materials.add(Color::rgb(0.3, 0.5, 0.3).into());

    const SCALE: f32 = 0.25;

    // plane
    commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform: Transform::from_scale(Vec2::splat(SCALE).extend(1.0)).with_translation(
                Vec3::new(
                    -glyph_mesh.width * SCALE / 2.0,
                    -glyph_mesh.height * SCALE / 2.0,
                    0.0,
                ),
            ),
            ..Default::default()
        },
        Wireframeable,
    ));

    // camera
    commands.spawn(Camera3dBundle {
        transform: {
            let mut transform = Transform::from_translation(Vec3::new(0.0, 0.0, 200.0))
                .looking_at(Vec3::default(), Vec3::Y);
            transform.scale = Vec2::splat(2.0).extend(1.0);
            transform
        },
        ..Default::default()
    });
}

fn zoom_and_pan(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        if input.pressed(KeyCode::Down) {
            transform.translation.z += transform.translation.z * time.delta_seconds();
        }
        if input.pressed(KeyCode::Up) {
            transform.translation.z -= transform.translation.z * time.delta_seconds();
        }
        if input.pressed(KeyCode::W) {
            transform.translation.y += transform.translation.z * time.delta_seconds();
        }
        if input.pressed(KeyCode::A) {
            transform.translation.x -= transform.translation.z * time.delta_seconds();
        }
        if input.pressed(KeyCode::S) {
            transform.translation.y -= transform.translation.z * time.delta_seconds();
        }
        if input.pressed(KeyCode::D) {
            transform.translation.x += transform.translation.z * time.delta_seconds();
        }
    }
}

#[derive(Component)]
struct Wireframeable;

fn wireframe(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&Wireframe>), With<Wireframeable>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for (entity, maybe_wireframe) in query.iter_mut() {
            if maybe_wireframe.is_some() {
                commands.entity(entity).remove::<Wireframe>();
            } else {
                commands.entity(entity).insert(Wireframe);
            }
        }
    }
}
