#![allow(clippy::too_many_arguments)]

use ab_glyph::{Font as AbFontTrait, GlyphId, PxScale};
use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    text::YAxisOrientation,
    // render::camera::Camera,
    utils::HashMap,
};
use bevy_flycam::prelude::*;
use glyph_brush_layout::{FontId, GlyphPositioner, Layout, SectionGeometry, SectionText};
use mesh::MeshError;

mod mesh;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, NoCameraPlayerPlugin, WireframePlugin))
        .insert_resource(Msaa::Sample8)
        .insert_resource(ClearColor(Color::rgb(0.52734375, 0.8046875, 0.91796875)))
        .init_resource::<FontGlyphMeshMap>()
        .add_systems(Startup, setup)
        // .add_systems(Update, zoom_and_pan)
        .add_systems(Update, wireframe)
        .add_systems(Update, queue_text_3d_system)
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
    // TODO: alignment doesn't work properly
    // TODO: sizing
    // Plane at origin
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Plane {
    //         size: 10000.0,
    //         ..default()
    //     })),
    //     material: materials.add(Color::rgb(0.02734375, 0.1171875, 0.92578125).into()),
    //     ..default()
    // });

    // Text at origin
    commands
        .spawn((
            SpatialBundle::default(),
            Text3d(Text::from_sections([
                // TextSection::new(
                //     "Airstrip Four",
                //     TextStyle {
                //         font: asset_server.load("fonts/airstrip.ttf"),
                //         font_size: 40.0,
                //         color: Color::rgb(0.5, 0.9, 0.5),
                //     },
                // ),
                // TextSection::new(
                //     "Deja Vu Sans Mono\n",
                //     TextStyle {
                //         font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                //         font_size: 40.0,
                //         color: Color::rgb(0.9, 0.5, 0.5),
                //     },
                // ),
                // TextSection::new(
                //     "Open Sans Italic\n",
                //     TextStyle {
                //         font: asset_server.load("fonts/DejaVuSansMono.ttf"),
                //         font_size: 40.0,
                //         color: Color::rgb(0.9, 0.5, 0.5),
                //     },
                // ),
                TextSection::new(
                    "Press F to toggle wireframes.\n",
                    TextStyle {
                        font: asset_server.load("fonts/OpenSans-Italic.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.9, 0.5),
                    },
                ),
                TextSection::new(
                    "Press F to toggle wireframes.\n",
                    TextStyle {
                        font: asset_server.load("fonts/airstrip.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.9, 0.5),
                    },
                ),
                TextSection::new(
                    "Press F to toggle wireframes.\n",
                    TextStyle {
                        font: asset_server.load("fonts/OpenSans-Italic.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.9, 0.5),
                    },
                ),
                TextSection::new(
                    "Press F to toggle wireframes.\n",
                    TextStyle {
                        font: asset_server.load("fonts/airstrip.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.9, 0.5),
                    },
                ),
                // TextSection::new(
                //     "wwmm\n",
                //     TextStyle {
                //         font: asset_server.load("fonts/airstrip.ttf"),
                //         font_size: 40.0,
                //         color: Color::rgb(0.5, 0.9, 0.5),
                //     },
                // ),
                // TextSection::new(
                //     "wwmm\n",
                //     TextStyle {
                //         font: asset_server.load("fonts/OpenSans-Italic.ttf"),
                //         font_size: 40.0,
                //         color: Color::rgb(0.5, 0.9, 0.5),
                //     },
                // ),
            ])),
        ))
        .with_children(|parent| {
            // spawn plane
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(500.0, 10.0)))),
                material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..default()
            });
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(10.0, 500.0)))),
                material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..default()
            });
        });
}

#[derive(Component)]
struct Text3d(Text);

#[derive(Component)]
struct Text3dSize(Vec2);

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

fn queue_text<M: AsMut<Assets<Mesh>>, F: AsRef<Assets<Font>>>(
    entity: Entity,
    text_3d: &Text3d,
    commands: &mut Commands,
    font_char_mesh_map: &mut FontGlyphMeshMap,
    waiting: &mut Vec<Entity>,
    fonts: &F,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut M,
    y_axis_orientation: YAxisOrientation,
) {
    let (maybe_font_arcs, (sections, styles)): (Vec<Option<_>>, (Vec<_>, Vec<_>)) = text_3d
        .0
        .sections
        .iter()
        .enumerate()
        .map(|(idx, section)| {
            (
                fonts
                    .as_ref()
                    .get(&section.style.font)
                    .map(|f| f.font.clone()),
                (
                    SectionText {
                        text: &section.value,
                        scale: PxScale::from(section.style.font_size),
                        font_id: FontId(idx),
                    },
                    section.style.clone(),
                ),
            )
        })
        .unzip();
    let Some(font_arcs) = maybe_font_arcs.into_iter().collect::<Option<Vec<_>>>() else {
        waiting.push(entity);
        return;
    };

    let glyphs =
        Layout::default().calculate_glyphs(&font_arcs, &SectionGeometry::default(), &sections);

    let mut children = Vec::with_capacity(glyphs.len());
    let mut text_bounds = Rect {
        min: Vec2::splat(std::f32::MAX),
        max: Vec2::splat(std::f32::MIN),
    };
    for glyph in glyphs.iter() {
        let style = &styles[glyph.section_index];

        let (mesh_data, font_meta) =
            match font_char_mesh_map.get(meshes, fonts, style.font.clone(), glyph.glyph.id) {
                Ok((mesh, font_scale)) => (mesh, font_scale),
                Err(GlyphMeshCreationError::NoOutline) => {
                    continue;
                }
                Err(GlyphMeshCreationError::FontNotYetLoaded) => {
                    waiting.push(entity);
                    return;
                }
            };

        let font_size = style.font_size;

        let scaled_position = glyph.glyph.position;

        let scaled_h_advance = mesh_data.unscaled_h_advance * font_size / font_meta.scale;
        let scaled_descent = font_meta.unscaled_descent * font_size / font_meta.scale;

        let x_offset = 0.0;
        let y_offset = 0.0;

        text_bounds = text_bounds.union(Rect {
            min: Vec2::new(scaled_position.x, 0.),
            max: Vec2::new(
                scaled_position.x + scaled_h_advance,
                scaled_position.y - scaled_descent,
            ),
        });

        let position = Vec2::new(scaled_position.x + x_offset, -scaled_position.y + y_offset);
        children.push((
            Wireframeable,
            PbrBundle {
                mesh: mesh_data.handle,
                // TODO: this should be configurable
                material: materials.add(style.color.into()),
                transform: Transform::from_scale(
                    Vec2::splat(font_size / font_meta.scale).extend(0.0),
                )
                .with_translation(position.extend(0.0)),
                ..default()
            },
        ));
    }

    dbg!(text_bounds, text_bounds.center());
    let center = text_bounds.center();
    let offset = Vec2::new(center.x, -center.y).extend(0.0);

    let children = children
        .into_iter()
        .map(|(wireframeable, mut pbr_bundle)| {
            pbr_bundle.transform.translation -= offset;
            commands.spawn((wireframeable, pbr_bundle)).id()
        })
        .collect::<Vec<_>>();
    commands
        .entity(entity)
        .insert(Text3dSize(Vec2::new(1.0, 1.0)))
        .replace_children(&children);
}

#[derive(Resource, Default)]
struct FontGlyphMeshMap {
    font_to_char_mesh_map: HashMap<Handle<Font>, FontData>,
}

struct FontData {
    meta: FontMeta,
    glyph_mesh_map: HashMap<GlyphId, GlyphMesh>,
}

impl FontData {
    fn new(scale: f32, unscaled_descent: f32) -> Self {
        Self {
            meta: FontMeta::new(scale, unscaled_descent),
            glyph_mesh_map: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct FontMeta {
    scale: f32,
    unscaled_descent: f32,
}

impl FontMeta {
    fn new(scale: f32, unscaled_descent: f32) -> Self {
        Self {
            scale,
            unscaled_descent,
        }
    }
}

#[derive(Clone, Debug)]
struct GlyphMesh {
    handle: Handle<Mesh>,
    unscaled_size: Vec2,
    unscaled_h_advance: f32,
}

enum GlyphMeshCreationError {
    FontNotYetLoaded,
    NoOutline,
}

impl FontGlyphMeshMap {
    // Retrieve or create a glyph mesh
    fn get<M: AsMut<Assets<Mesh>>, F: AsRef<Assets<Font>>>(
        &mut self,
        meshes: &mut M,
        fonts: &F,
        font_handle: Handle<Font>,
        g: GlyphId,
    ) -> Result<(GlyphMesh, FontMeta), GlyphMeshCreationError> {
        let meshes = meshes.as_mut();
        let fonts = fonts.as_ref();
        let font = fonts
            .get(&font_handle)
            .ok_or(GlyphMeshCreationError::FontNotYetLoaded)?;
        let font_data = self
            .font_to_char_mesh_map
            .entry(font_handle.clone())
            .or_insert_with(|| {
                let font_scale = font.font.height_unscaled();
                let unscaled_descent = font.font.descent_unscaled();
                FontData::new(font_scale, unscaled_descent)
            });
        let (mesh_data, meta) = match font_data.glyph_mesh_map.get(&g) {
            // already in the map
            Some(mesh_data) => (mesh_data.clone(), font_data.meta),
            // not yet in the map
            None => {
                // build the mesh
                let mesh_data = match mesh::build_mesh(&font.font, g) {
                    // built the mesh
                    Ok(mesh) => {
                        let handle = meshes.add(mesh.mesh);
                        let unscaled_h_advance = font.font.h_advance_unscaled(g);
                        font_data.glyph_mesh_map.insert(
                            g,
                            GlyphMesh {
                                handle,
                                unscaled_size: Vec2::new(mesh.width, mesh.height),
                                unscaled_h_advance,
                            },
                        );
                        font_data.glyph_mesh_map.get(&g).unwrap()
                    }
                    // could not build the mesh for this glyph
                    Err(err) => match err {
                        // there was no outline for this glyph
                        MeshError::NoOutline => {
                            return Err(GlyphMeshCreationError::NoOutline);
                        }
                        // failed to tessellate this glyph
                        MeshError::TessellationError(tess_err) => {
                            todo!("not yet handled: {tess_err}")
                        }
                    },
                };
                (mesh_data.clone(), font_data.meta)
            }
        };
        Ok((mesh_data, meta))
    }
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

#[derive(Component)]
struct Wireframeable;

fn wireframe(
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

fn adjust_movement_settings(
    mut settings: ResMut<MovementSettings>,
    camera: Query<&Transform, With<FlyCam>>,
) {
    settings.speed =
        MovementSettings::default().speed + camera.single().translation.distance(Vec3::ZERO);
}
