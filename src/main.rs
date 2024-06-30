use std::time::Duration;

use animations::{perm_to_quat, rotate_animation_system, RotateAnimation};
use bevy::{
    animation::animation_player,
    input::keyboard::KeyboardInput,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    utils::HashSet,
};
use names::{cube_position_name, rubik_name};
use rubik::{
    colored::CubeFaceMap,
    cube::{Cube, CubeFace},
    permutation::CubePermutation,
    transform::{RubikLayerTransform, RubikTransform},
    CubePosition, RubikLayer,
};
mod animations;
mod names;
fn main() {
    App::new()
        .add_systems(Startup, init_orbit_camera)
        .add_plugins(DefaultPlugins)
        .add_plugins(RubikPlugin)
        .run();
}

#[derive(Debug, Component)]
pub struct MainCamera;

#[derive(Debug, Component)]
pub struct RubikBlock {
    pub position: CubePosition,
    pub perm: CubePermutation,
}

#[derive(Debug, Component)]
pub struct BlockId {
    pub init_position: CubePosition,
}

use bevy::animation::prelude::*;


#[derive(Resource)]
pub struct RubikColor {
    pub map: CubeFaceMap<Handle<StandardMaterial>>,
}

#[derive(Debug, Component, Default)]
pub struct Rubik {
    rubik: rubik::Rubik,
}

fn init_orbit_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        },
        MainCamera,
    ));
}

fn init_color_map(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let map = CubeFaceMap::new(
        materials.add(Color::RED),
        materials.add(Color::BLUE),
        materials.add(Color::WHITE),
        materials.add(Color::ORANGE),
        materials.add(Color::GREEN),
        materials.add(Color::YELLOW),
    );
    commands.insert_resource(RubikColor { map });
}

fn init_cube(mut commands: Commands, color: Res<RubikColor>, mut meshed: ResMut<Assets<Mesh>>) {
    use bevy::math::prelude::{Rectangle, Triangle2d};
    const CUBE_FACE_INNER_SIZE: f32 = 0.9;
    const CUBE_FACE_OUTER_SIZE: f32 = 1.0;
    const CUBE_FACE: Rectangle = Rectangle {
        half_size: Vec2::new(CUBE_FACE_INNER_SIZE / 2.0, CUBE_FACE_INNER_SIZE / 2.0),
    };
    const CUBE_EDGE_FACE: Rectangle = Rectangle {
        half_size: Vec2::new(
            CUBE_FACE_INNER_SIZE,
            (CUBE_FACE_OUTER_SIZE - CUBE_FACE_INNER_SIZE) * std::f32::consts::SQRT_2,
        ),
    };
    //
    //  0 - 1
    //
    //
    //
    //
    let vertices = [
        ([0.0, 0.5, 0.0]),   // 顶点 A
        ([0.5, -0.5, 0.0]),  // 顶点 B
        ([-0.5, -0.5, 0.0]), // 顶点 C
    ];
    let mut cube_block_mesh: Mesh = Mesh::new(
        PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices.to_vec())
    .with_inserted_indices(Indices::U16(vec![]));
    let face_and_tf: [(CubeFace, Transform); 6] = [
        (
            CubeFace::F,
            Transform::from_rotation(Quat::default()).with_translation(Vec3::new(
                0.0,
                0.0,
                CUBE_FACE_INNER_SIZE / 2.0,
            )),
        ),
        (
            CubeFace::B,
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI))
                .with_translation(Vec3::new(0.0, 0.0, -CUBE_FACE_INNER_SIZE / 2.0)),
        ),
        (
            CubeFace::R,
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(CUBE_FACE_INNER_SIZE / 2.0, 0.0, 0.0)),
        ),
        (
            CubeFace::L,
            Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(-CUBE_FACE_INNER_SIZE / 2.0, 0.0, 0.0)),
        ),
        (
            CubeFace::U,
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(0.0, CUBE_FACE_INNER_SIZE / 2.0, 0.0)),
        ),
        (
            CubeFace::D,
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(0.0, -CUBE_FACE_INNER_SIZE / 2.0, 0.0)),
        ),
    ];

    // commands.spawn(DirectionalLightBundle {
    //     transform: Transform::from_translation(Vec3::new(100.0, 100.0, 100.0)),

    //     ..Default::default()
    // });

    let rubik = commands
        .spawn((
            Rubik::default(),
            SpatialBundle::default(),
            AnimationPlayer::default(),
            rubik_name(),
        ))
        .id();
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                let tf = Transform::from_translation(
                    Vec3::new(x as f32, y as f32, z as f32) * CUBE_FACE_OUTER_SIZE
                        - Vec3::splat(CUBE_FACE_OUTER_SIZE) * 1.5,
                );
                let cube_position = CubePosition::try_from_u8(x + 3 * (2 - z) + 9 * (2 - y))
                    .expect("invalid cube position");
                let cube = commands
                    .spawn((
                        RubikBlock {
                            position: cube_position,
                            perm: CubePermutation::UNIT,
                        },
                        SpatialBundle {
                            transform: tf,
                            ..Default::default()
                        },
                        cube_position_name(cube_position),
                    ))
                    .id();
                for (face, tf) in face_and_tf.iter() {
                    commands
                        .spawn(PbrBundle {
                            mesh: meshed.add(CUBE_FACE),
                            material: color.map.get(*face).clone(),
                            transform: *tf,
                            ..Default::default()
                        })
                        .set_parent(cube);
                }
                commands.entity(cube).set_parent(rubik);
            }
        }
    }
}

pub struct RubikPlugin;
impl Plugin for RubikPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_color_map)
            .add_systems(Startup, init_cube.after(init_color_map))
            .add_systems(Update, (rotate_animation_system, handle_permutation_input));
    }
}

#[derive(Debug, Default, Component)]
pub struct Playing;
#[allow(clippy::single_match)]
fn handle_permutation_input(
    mut commands: Commands,
    mut q_rubik: Query<(Entity, &mut Rubik, &mut AnimationPlayer)>,
    mut q_blocks: Query<(Entity, &mut RubikBlock, &mut Name), Without<RotateAnimation>>,
    mut kdb_input_er: EventReader<KeyboardInput>,
    mut kdb: Res<ButtonInput<KeyCode>>,
) {
    let Ok((entity, mut rubik, mut player)) = q_rubik.get_single_mut() else {
        return;
    };
    for event in kdb_input_er.read() {
        match event.key_code {
            KeyCode::KeyR => {
                if kdb.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                    rubik
                        .rubik
                        .execute(&RubikTransform::Layer(RubikLayerTransform::RI));
                } else {
                    let rubik_tf = RubikLayerTransform::RI;
                    // let rot = rubik_tf.rotation();
                    for (block_id, mut block, name) in q_blocks.iter_mut() {
                        let to = block.perm.compose(CubePermutation::Y_3);
                        dbg!(to, block.perm);
                        if RubikLayer::R.contains(&(block.position as u8)) {
                            commands.entity(block_id).insert(RotateAnimation {
                                axis: Vec3::Y,
                                s: 0.0,
                                duration: Duration::from_secs(1),
                                from: perm_to_quat(block.perm),
                                to: perm_to_quat(to),
                            });
                            block.as_mut().perm = to;
                        }
                    }
                    rubik
                        .rubik
                        .execute(&RubikTransform::Layer(RubikLayerTransform::R));
                }
                break;
            }

            _ => {}
        }
    }
}
