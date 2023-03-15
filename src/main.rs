use ab_glyph::{Font, FontRef};
use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::{camera::Camera, mesh::Indices, render_resource::PrimitiveTopology},
};
use lyon::{
    geom::euclid::Point2D,
    math::{point, Point},
    path::Path,
    tessellation::*,
};

trait ToLyonPoint {
    fn to_lyon_point(&self) -> Point;
}

impl ToLyonPoint for ab_glyph::Point {
    fn to_lyon_point(&self) -> Point {
        point(self.x, self.y)
    }
}

// Let's use our own custom vertex type instead of the default one.
#[derive(Copy, Clone, Debug)]
struct MyVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

struct Ctor {
    min_x: f32,
    min_y: f32,
    width: f32,
    height: f32,
}

impl FillVertexConstructor<MyVertex> for Ctor {
    fn new_vertex(&mut self, vertex: FillVertex) -> MyVertex {
        let position = vertex.position();
        let Point2D { x, y, .. } = position;
        MyVertex {
            position: [x, y, 0.0],
            normal: Vec3::Z.to_array(),
            uv: [
                (x - self.min_x) / self.width,
                1.0 - (y - self.min_y) / self.height,
            ],
        }
    }
}

enum MyCurve {
    Line,
    Quad(Point),
    Cubic(Point, Point),
}

fn build_mesh(font: FontRef, the_char: char) -> Option<GlyphMesh> {
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

    {
        use ab_glyph::OutlineCurve::*;

        // start path
        let mut iterator = outline
            .curves
            .into_iter()
            .map(|curve| match curve {
                Line(from, to) => (from.to_lyon_point(), to.to_lyon_point(), MyCurve::Line),
                Quad(from, ctrl, to) => (
                    from.to_lyon_point(),
                    to.to_lyon_point(),
                    MyCurve::Quad(ctrl.to_lyon_point()),
                ),
                Cubic(from, ctrl1, ctrl2, to) => (
                    from.to_lyon_point(),
                    to.to_lyon_point(),
                    MyCurve::Cubic(ctrl1.to_lyon_point(), ctrl2.to_lyon_point()),
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
                MyCurve::Line => builder.line_to(to),
                MyCurve::Quad(ctrl) => builder.quadratic_bezier_to(ctrl, to),
                MyCurve::Cubic(ctrl1, ctrl2) => builder.cubic_bezier_to(ctrl1, ctrl2, to),
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
    }

    let path = builder.build();

    // Will contain the result of the tessellation.
    let mut geometry: VertexBuffers<MyVertex, u32> = VertexBuffers::new();

    {
        // Compute the tessellation.
        FillTessellator::new()
            .tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(
                    &mut geometry,
                    Ctor {
                        min_x,
                        min_y,
                        width,
                        height,
                    },
                ),
            )
            .unwrap();
    }

    // The tessellated geometry is ready to be uploaded to the GPU.
    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();

    for MyVertex {
        position,
        normal,
        uv,
    } in geometry.vertices.iter()
    {
        positions.push([position[0], position[1], position[2]]);
        normals.push(*normal);
        uvs.push(*uv);
    }

    println!("{:#?}", &geometry);

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

struct GlyphMesh {
    mesh: Mesh,
    width: f32,
    height: f32,
}

/// This example shows various ways to configure texture materials in 3D
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

/// sets up a scene with textured entities
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
    let glyph_mesh = build_mesh(font, the_char).unwrap();

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
