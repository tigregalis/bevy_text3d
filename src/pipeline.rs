use std::collections::HashMap;

use crate::{
    mesh::{self, MeshError},
    plugin::Wireframeable,
};

use ab_glyph::{Font as _, GlyphId, PxScale};
use bevy::{prelude::*, text::YAxisOrientation};
use glyph_brush_layout::{FontId, GlyphPositioner, Layout, SectionGeometry, SectionText};

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct Text3d(pub Text);

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct Text3dSize(Vec2);

pub(crate) fn queue_text<M: AsMut<Assets<Mesh>>, F: AsRef<Assets<Font>>>(
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
pub struct FontGlyphMeshMap {
    font_to_char_mesh_map: HashMap<Handle<Font>, FontData>,
}

struct FontData {
    meta: FontMeta,
    glyph_mesh_map: HashMap<GlyphId, GlyphMeshMeta>,
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
struct GlyphMeshMeta {
    handle: Handle<Mesh>,
    _unscaled_size: Vec2,
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
    ) -> Result<(GlyphMeshMeta, FontMeta), GlyphMeshCreationError> {
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
                            GlyphMeshMeta {
                                handle,
                                _unscaled_size: Vec2::new(mesh.width, mesh.height),
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
