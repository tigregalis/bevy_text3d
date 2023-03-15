use ab_glyph::{Font, FontRef};
use bevy::{
    prelude::*,
    render::{camera::Camera, mesh::Indices, render_resource::PrimitiveTopology},
};
use lyon::math::{point, Point};
use lyon::path::Path;
use lyon::tessellation::*;

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

/// This example shows various ways to configure texture materials in 3D
fn main() {
    App::new()
        .insert_resource(Msaa::Sample8)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(zoom_and_pan)
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
    let glyph_id = font.glyph_id(the_char);
    if let Some(outline) = font.outline(glyph_id) {
        let mut builder = Path::builder();
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
            for curve in outline.curves {
                dbg!(&curve);
                let current_position = builder.current_position();
                match curve {
                    Line(from, to) => {
                        // TODO: see if there is an idiomatic way to do something like eq(a, b).then(|a, b| {})
                        let from = from.to_lyon_point();
                        if current_position != from {
                            builder.move_to(from);
                        }

                        builder.line_to(to.to_lyon_point());
                    }
                    Quad(from, ctrl, to) => {
                        let from = from.to_lyon_point();
                        if current_position != from {
                            builder.move_to(from);
                        }

                        builder.quadratic_bezier_to(ctrl.to_lyon_point(), to.to_lyon_point());
                    }
                    Cubic(from, ctrl1, ctrl2, to) => {
                        let from = from.to_lyon_point();
                        if current_position != from {
                            builder.move_to(from);
                        }

                        builder.cubic_bezier_to(
                            ctrl1.to_lyon_point(),
                            ctrl2.to_lyon_point(),
                            to.to_lyon_point(),
                        );
                    }
                }
            }
        }
        builder.close();
        let path = builder.build();

        // Will contain the result of the tessellation.
        let mut geometry: VertexBuffers<MyVertex, u32> = VertexBuffers::new();

        const NORMAL: [f32; 3] = [0.0, 0.0, 1.0];
        {
            // Compute the tessellation.
            FillTessellator::new()
                .tessellate_path(
                    &path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: FillAttributes| {
                        let (x, y) = pos.to_tuple();
                        MyVertex {
                            position: [x, y, 0.0],
                            normal: NORMAL,
                            uv: [(x - min_x) / width, 1.0 - (y - min_y) / height],
                        }
                    }),
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
        let mesh = meshes.add(mesh);

        let material = materials.add(Color::rgb(0.3, 0.5, 0.3).into());

        // plane
        commands.spawn(PbrBundle {
            mesh,
            material,
            transform: Transform::from_scale(Vec2::splat(0.1).extend(1.0)),
            ..Default::default()
        });

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
