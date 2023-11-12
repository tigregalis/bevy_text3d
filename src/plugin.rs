use bevy::{pbr::wireframe::Wireframe, prelude::*, text::YAxisOrientation};

use crate::pipeline::{queue_text, FontGlyphMeshMap, Text3d};

pub struct Text3dPlugin;

impl Plugin for Text3dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontGlyphMeshMap>()
            .add_systems(PreUpdate, queue_text_3d_system)
            .add_systems(Update, wireframe_system);
    }
}

fn queue_text_3d_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    fonts: Res<Assets<Font>>,
    mut font_char_mesh_map: ResMut<FontGlyphMeshMap>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    text_3ds_changed: Query<(Entity, &Text3d), Changed<Text3d>>,
    text_3ds_all: Query<(Entity, &Text3d)>,
    mut waiting_last_tick: Local<Vec<Entity>>,
    mut waiting_next_tick: Local<Vec<Entity>>,
) {
    for (entity, text_3d) in waiting_last_tick
        .drain(..)
        .filter_map(|entity| text_3ds_all.get(entity).ok())
    {
        eprintln!("queueing text for waiting entities");
        queue_text(
            entity,
            text_3d,
            &mut commands,
            &mut font_char_mesh_map,
            &mut waiting_next_tick,
            &fonts,
            &mut materials,
            &mut meshes,
            YAxisOrientation::BottomToTop,
        );
    }
    for (entity, text_3d) in text_3ds_changed.iter() {
        eprintln!("queueing text for changed entities");
        queue_text(
            entity,
            text_3d,
            &mut commands,
            &mut font_char_mesh_map,
            &mut waiting_next_tick,
            &fonts,
            &mut materials,
            &mut meshes,
            YAxisOrientation::BottomToTop,
        );
    }
    // TODO: is the below expensive to do every frame, or is checking waiting_next_tick.len() worse?
    std::mem::swap(&mut *waiting_last_tick, &mut *waiting_next_tick);
}

#[derive(Component)]
pub(crate) struct Wireframeable;

fn wireframe_system(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&Wireframe>), With<Wireframeable>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F) {
        for (entity, maybe_wireframe) in query.iter_mut() {
            if maybe_wireframe.is_some() {
                commands.entity(entity).remove::<Wireframe>();
            } else {
                commands.entity(entity).insert(Wireframe);
            }
        }
    }
}
