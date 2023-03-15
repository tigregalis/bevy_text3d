use ab_glyph::{Font, FontRef, OutlineCurve};

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use lyon::{geom::euclid::Point2D, path::Path, tessellation::*};

pub fn build_mesh(font: FontRef, the_char: char) -> Option<GlyphMesh> {
    let glyph_id = font.glyph_id(the_char);
    let Some(outline) = font.outline(glyph_id) else { return None };
    let mut builder = Path::builder();
    builder.reserve(outline.curves.len(), 2 * outline.curves.len());
    let a = outline.bounds.min;
    let b = outline.bounds.max;
    let min_x = a.x.min(b.x);
    let max_x = a.x.max(b.x);
    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);
    let width = max_x - min_x;
    let height = max_y - min_y;

    // start path
    let mut iterator = outline
        .curves
        .into_iter()
        .map(|curve| match curve {
            OutlineCurve::Line(from, to) => {
                (from.to_lyon_point(), to.to_lyon_point(), CurveMapping::Line)
            }
            OutlineCurve::Quad(from, ctrl, to) => (
                from.to_lyon_point(),
                to.to_lyon_point(),
                CurveMapping::Quad(ctrl.to_lyon_point()),
            ),
            OutlineCurve::Cubic(from, ctrl1, ctrl2, to) => (
                from.to_lyon_point(),
                to.to_lyon_point(),
                CurveMapping::Cubic(ctrl1.to_lyon_point(), ctrl2.to_lyon_point()),
            ),
        })
        .enumerate()
        .peekable();
    while let Some((idx, (from, to, curve_type))) = iterator.next() {
        // if first path, start path
        if idx == 0 {
            builder.begin(from);
        }
        // take path
        match curve_type {
            CurveMapping::Line => builder.line_to(to),
            CurveMapping::Quad(ctrl) => builder.quadratic_bezier_to(ctrl, to),
            CurveMapping::Cubic(ctrl1, ctrl2) => builder.cubic_bezier_to(ctrl1, ctrl2, to),
        };
        // if required (next is different), finish path, start next path
        if let Some((_, (next_from, _, _))) = iterator.peek() {
            if *next_from != to {
                builder.end(false);
                builder.begin(*next_from);
            }
        } else {
            builder.close();
        }
    }

    let path = builder.build();

    let mut geometry: VertexBuffers<VertexInfo, u32> = VertexBuffers::new();

    FillTessellator::new()
        .tessellate_path(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(
                &mut geometry,
                VertexFiller {
                    min_x,
                    min_y,
                    width,
                    height,
                },
            ),
        )
        .unwrap();

    let normals = vec![Vec3::Z.to_array(); geometry.vertices.len()];
    let mut positions = Vec::<[f32; 3]>::with_capacity(geometry.vertices.len());
    let mut uvs = Vec::<[f32; 2]>::with_capacity(geometry.vertices.len());

    for VertexInfo { position, uv } in geometry.vertices.iter() {
        positions.push(*position);
        uvs.push(*uv);
    }

    geometry.indices.reverse(); // bevy has a right-handed coordinate system

    let indices = Indices::U32(geometry.indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    Some(GlyphMesh {
        mesh,
        width,
        height,
    })
}

trait ToLyonPoint {
    fn to_lyon_point(&self) -> lyon::math::Point;
}

impl ToLyonPoint for ab_glyph::Point {
    fn to_lyon_point(&self) -> lyon::math::Point {
        lyon::math::point(self.x, self.y)
    }
}

enum CurveMapping {
    Line,
    Quad(lyon::math::Point),
    Cubic(lyon::math::Point, lyon::math::Point),
}

struct VertexFiller {
    min_x: f32,
    min_y: f32,
    width: f32,
    height: f32,
}

impl FillVertexConstructor<VertexInfo> for VertexFiller {
    fn new_vertex(&mut self, vertex: FillVertex) -> VertexInfo {
        let position = vertex.position();
        let Point2D { x, y, .. } = position;
        VertexInfo {
            position: [x, y, 0.0],
            uv: [
                (x - self.min_x) / self.width,
                1.0 - (y - self.min_y) / self.height,
            ],
        }
    }
}

pub struct GlyphMesh {
    pub mesh: Mesh,
    pub width: f32,
    pub height: f32,
}

#[derive(Copy, Clone, Debug)]
struct VertexInfo {
    position: [f32; 3],
    uv: [f32; 2],
}
