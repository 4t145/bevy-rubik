use bevy::{input::keyboard::KeyboardInput, prelude::*, utils::HashSet};
use rubik::{
    colored::CubeFaceMap,
    cube::{Cube, CubeFace},
    permutation::CubePermutation,
    transform::{RubikLayerTransform, RubikTransform},
    RubikLayer,
};

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
    pub permutation_index: u8,
}

#[derive(Component)]
struct RotateAnimation {
    axis: Vec3,
    angle: f32,
    duration: f32,
    elapsed: f32,
}

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
        half_size: Vec2::new(CUBE_FACE_INNER_SIZE, CUBE_FACE_INNER_SIZE),
    };
    const CUBE_EDGE_FACE: Rectangle = Rectangle {
        half_size: Vec2::new(
            CUBE_FACE_INNER_SIZE,
            (CUBE_FACE_OUTER_SIZE - CUBE_FACE_INNER_SIZE) * std::f32::consts::SQRT_2,
        ),
    };
    
    let face_and_tf: [(CubeFace, Transform); 6] = [
        (
            CubeFace::F,
            Transform::from_rotation(Quat::default()).with_translation(Vec3::new(0.0, 0.0, 1.0)),
        ),
        (
            CubeFace::B,
            Transform::from_rotation(Quat::default()).with_translation(Vec3::new(0.0, 0.0, -1.0)),
        ),
        (
            CubeFace::R,
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(1.0, 0.0, 0.0)),
        ),
        (
            CubeFace::L,
            Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(-1.0, 0.0, 0.0)),
        ),
        (
            CubeFace::U,
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(0.0, 1.0, 0.0)),
        ),
        (
            CubeFace::D,
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(0.0, -1.0, 0.0)),
        ),
    ];

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(50.0, 50.0, 50.0)),

        ..Default::default()
    });
    let rubik = commands
        .spawn((Rubik::default(), SpatialBundle::default()))
        .id();
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                let tf = Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32) * 2.0);
                let cube = commands
                    .spawn((
                        RubikBlock {
                            permutation_index: 9 * x + 3 * y + z,
                        },
                        SpatialBundle {
                            transform: tf,
                            ..Default::default()
                        },
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

fn rotate_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut RotateAnimation, &mut Transform)>,
) {
    for (mut animation, mut transform) in query.iter_mut() {
        if animation.elapsed < animation.duration {
            let delta_angle = animation.angle * (time.delta_seconds() / animation.duration);
            transform.rotate_around(
                Vec3::ZERO,
                Quat::from_axis_angle(animation.axis, delta_angle),
            );
            animation.elapsed += time.delta_seconds();
        }
    }
}
#[allow(clippy::single_match)]
fn handle_permutation_input(
    mut commands: Commands,
    mut q_rubik: Query<&mut Rubik>,
    mut q_blocks: Query<(Entity, &mut RubikBlock)>,
    mut kdb_input_er: EventReader<KeyboardInput>,
    mut kdb: Res<ButtonInput<KeyCode>>,
) {
    let mut rubik = q_rubik.single_mut();
    for event in kdb_input_er.read() {
        match event.key_code {
            KeyCode::KeyR => {
                let cube_set = rubik
                    .rubik
                    .iter_by_layer(&RubikLayer::R)
                    .copied()
                    .collect::<HashSet<Cube>>();
                if kdb.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                    rubik
                        .rubik
                        .execute(&RubikTransform::Layer(RubikLayerTransform::RI));
                } else {
                    rubik
                        .rubik
                        .execute(&RubikTransform::Layer(RubikLayerTransform::R));
                }
            }

            _ => {}
        }
    }
}