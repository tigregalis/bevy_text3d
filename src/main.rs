use ab_glyph::{Font, FontRef};
use bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
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
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(zoom_and_pan)
        .add_system(color)
        .run();
}

/// sets up a scene with textured entities
fn setup(
    commands: &mut Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // for some characters e.g. X there is no indices at all - why?
    const THE_CHAR: char = 'a';
    let the_char: char = THE_CHAR;

    let font = FontRef::try_from_slice(include_bytes!("../fonts/OpenSans-Italic.ttf")).unwrap();
    let glyph_id = font.glyph_id(the_char);
    // dbg!(font.h_advance_unscaled(glyph_id));
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

        // dbg!(min_x, min_y, max_x, max_y, width, height);
        {
            use ab_glyph::OutlineCurve::*;
            for curve in outline.curves {
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

        // let mut tessellator = FillTessellator::new();

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
                            // position: [x - min_x - width / 2.0, y - min_y - height / 2.0, 0.0],
                            position: [x, y, 0.0],
                            normal: NORMAL,
                            uv: [(x - min_x) / width, 1.0 - (y - min_y) / height],
                        }
                    }),
                )
                .unwrap();
        }

        // The tessellated geometry is ready to be uploaded to the GPU.
        print_geometry(&geometry, the_char);

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

        geometry.indices.reverse(); // believe this has something to do with the normals

        let indices = Indices::U32(geometry.indices);

        // dbg!(&positions);
        // dbg!(&normals);
        // dbg!(&uvs);
        // dbg!(&indices);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        // dbg!(&mesh);
        let mesh_handle = meshes.add(mesh);

        // // load a texture and retrieve its aspect ratio
        // // let texture_handle = asset_server.load("branding/bevy_logo_dark_big.png");
        // // let aspect = 0.25;

        // // create a new quad mesh. this is what we will apply the texture to
        // // let quad_width = 10.0;
        // // let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        // //     quad_width,
        // //     quad_width * aspect,
        // // ))));

        // // // this material renders the texture normally
        // // let material_handle = materials.add(StandardMaterial {
        // //     albedo_texture: Some(texture_handle.clone()),
        // //     shaded: false,
        // //     ..Default::default()
        // // });

        // // // this material modulates the texture to make it red (and slightly transparent)
        // // let red_material_handle = materials.add(StandardMaterial {
        // //     albedo: Color::rgba(1.0, 0.0, 0.0, 0.5),
        // //     albedo_texture: Some(texture_handle.clone()),
        // //     shaded: false,
        // // });

        // // // and lets make this one blue! (and also slightly transparent)
        // // let blue_material_handle = materials.add(StandardMaterial {
        // //     albedo: Color::rgba(0.0, 0.0, 1.0, 0.5),
        // //     albedo_texture: Some(texture_handle),
        // //     shaded: false,
        // // });

        // add entities to the world
        // commands
        //     // textured quad - normal
        //     // .spawn(PbrBundle {
        //     //     mesh: quad_handle.clone(),
        //     //     material: material_handle,
        //     //     transform: Transform {
        //     //         translation: Vec3::new(0.0, 0.0, 1.5),
        //     //         rotation: Quat::from_rotation_x(-std::f32::consts::PI / 5.0),
        //     //         ..Default::default()
        //     //     },
        //     //     draw: Draw {
        //     //         is_transparent: true,
        //     //         ..Default::default()
        //     //     },
        //     //     ..Default::default()
        //     // })
        //     // // textured quad - modulated
        //     // .spawn(PbrBundle {
        //     //     mesh: quad_handle.clone(),
        //     //     material: red_material_handle,
        //     //     transform: Transform {
        //     //         translation: Vec3::new(0.0, 0.0, 0.0),
        //     //         rotation: Quat::from_rotation_x(-std::f32::consts::PI / 5.0),
        //     //         ..Default::default()
        //     //     },
        //     //     draw: Draw {
        //     //         is_transparent: true,
        //     //         ..Default::default()
        //     //     },
        //     //     ..Default::default()
        //     // })
        //     // // textured quad - modulated
        //     // .spawn(PbrBundle {
        //     //     mesh: quad_handle,
        //     //     material: blue_material_handle,
        //     //     transform: Transform {
        //     //         translation: Vec3::new(0.0, 0.0, -1.5),
        //     //         rotation: Quat::from_rotation_x(-std::f32::consts::PI / 5.0),
        //     //         ..Default::default()
        //     //     },
        //     //     draw: Draw {
        //     //         is_transparent: true,
        //     //         ..Default::default()
        //     //     },
        //     //     ..Default::default()
        //     // })

        let material = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
        // let material = materials.add(StandardMaterial {
        //     albedo: Color::rgba(0.0, 0.0, 1.0, 1.0),
        //     ..Default::default()
        // });
        commands
            // plane
            .insert_resource(material.clone())
            .spawn(PbrBundle {
                mesh: mesh_handle,
                material,
                transform: Transform::from_scale(Vec2::splat(0.1).extend(1.0)),
                ..Default::default()
            })
            // camera
            .spawn(Camera3dBundle {
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(0.0, 0.0, 500.0))
                        .looking_at(Vec3::default(), Vec3::unit_y());
                    transform.scale = Vec2::splat(2.0).extend(1.0);
                    transform
                },
                ..Default::default()
            });
    }
}

fn color(
    mut timer: Local<Option<Timer>>,
    mut index: Local<u8>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_handle: Res<Handle<StandardMaterial>>,
) {
    if timer.is_none() {
        *timer = Some(Timer::from_seconds(0.5, true));
        return;
    }
    if let Some(material) = materials.get_mut(material_handle.clone()) {
        if let Some(timer) = timer.as_mut() {
            timer.tick(time.delta_seconds());
            if timer.finished() {
                material.albedo = match *index {
                    0 => Color::ALICE_BLUE,
                    1 => Color::ANTIQUE_WHITE,
                    2 => Color::AQUAMARINE,
                    3 => Color::AZURE,
                    4 => Color::BEIGE,
                    5 => Color::BISQUE,
                    6 => Color::BLACK,
                    7 => Color::BLUE,
                    8 => Color::CRIMSON,
                    9 => Color::CYAN,
                    10 => Color::DARK_GRAY,
                    11 => Color::DARK_GREEN,
                    12 => Color::FUCHSIA,
                    13 => Color::GOLD,
                    14 => Color::GRAY,
                    15 => Color::GREEN,
                    16 => Color::INDIGO,
                    17 => Color::LIME_GREEN,
                    18 => Color::MAROON,
                    19 => Color::MIDNIGHT_BLUE,
                    20 => Color::NAVY,
                    21 => Color::OLIVE,
                    22 => Color::ORANGE,
                    23 => Color::ORANGE_RED,
                    24 => Color::PINK,
                    25 => Color::PURPLE,
                    26 => Color::RED,
                    27 => Color::SALMON,
                    28 => Color::SEA_GREEN,
                    29 => Color::SILVER,
                    30 => Color::TEAL,
                    31 => Color::TOMATO,
                    32 => Color::TURQUOISE,
                    33 => Color::VIOLET,
                    34 => Color::WHITE,
                    35 => Color::YELLOW,
                    36 => Color::YELLOW_GREEN,
                    _ => Color::NONE,
                };
                *index += 1;
                if *index > 36 {
                    *index = 0;
                }
            }
        }
    }
}

fn zoom_and_pan(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<bevy::render::camera::Camera>>,
) {
    for mut transform in query.iter_mut() {
        // let mut scale = transform.scale.x;
        if input.pressed(KeyCode::Down) {
            // scale += time.delta_seconds();
            transform.translation.z += 100.0 * time.delta_seconds();
        }
        if input.pressed(KeyCode::Up) {
            transform.translation.z -= 100.0 * time.delta_seconds();
            // scale -= time.delta_seconds();
        }
        // transform.scale = Vec2::splat(scale).extend(1.0);
        if input.pressed(KeyCode::W) {
            transform.translation.y += 100.0 * time.delta_seconds();
        }
        if input.pressed(KeyCode::A) {
            transform.translation.x -= 100.0 * time.delta_seconds();
        }
        if input.pressed(KeyCode::S) {
            transform.translation.y -= 100.0 * time.delta_seconds();
        }
        if input.pressed(KeyCode::D) {
            transform.translation.x += 100.0 * time.delta_seconds();
        }
    }
}

fn print_geometry(geometry: &VertexBuffers<MyVertex, u32>, the_char: char) {
    for triangle in geometry.indices.chunks_exact(3) {
        if let [index0, index1, index2] = triangle {
            let position0 = geometry.vertices[*index0 as usize].position;
            let position1 = geometry.vertices[*index1 as usize].position;
            let position2 = geometry.vertices[*index2 as usize].position;
            println!(
                "{}\t{}\t{}\t{}",
                the_char,
                position0[0],
                position0[1],
                the_char.is_uppercase()
            );
            println!(
                "{}\t{}\t{}\t{}",
                the_char,
                position1[0],
                position1[1],
                the_char.is_uppercase()
            );
            println!(
                "{}\t{}\t{}\t{}",
                the_char,
                position2[0],
                position2[1],
                the_char.is_uppercase()
            );
            println!(
                "{}\t{}\t{}\t{}",
                the_char,
                position0[0],
                position0[1],
                the_char.is_uppercase()
            );
        }
    }
}
