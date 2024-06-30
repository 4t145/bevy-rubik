use std::{collections::HashMap, sync::OnceLock, time::Duration};

use bevy::prelude::*;
use rubik::{permutation::CubePermutation, transform::RubikLayerTransform, CubePosition};

use crate::names::{cube_position_name, rubik_name};

pub struct TransformAnimationMap {}
pub fn perm_to_quat(perm: rubik::permutation::CubePermutation) -> Quat {
    let (rot_0, rot_1, rot_2) = perm.factor_3();
    let rot_0 = match rot_0 {
        CubePermutation::UNIT => Quat::default(),
        CubePermutation::X_2 => Quat::from_rotation_z(std::f32::consts::PI),
        CubePermutation::Y_2 => Quat::from_rotation_x(std::f32::consts::PI),
        CubePermutation::Z_2 => Quat::from_rotation_y(std::f32::consts::PI),
        _ => unreachable!(),
    };
    let rot_1 = match rot_1 {
        CubePermutation::UNIT => Quat::default(),
        CubePermutation::C1 => {
            Quat::from_axis_angle(Vec3::new(1.0, 1.0, 1.0).normalize(), std::f32::consts::FRAC_PI_3 * 2.0)
        }
        CubePermutation::C2 => {
            Quat::from_axis_angle(Vec3::new(1.0, 1.0, 1.0).normalize(), -std::f32::consts::FRAC_PI_3 * 2.0)
        }
        _ => unreachable!(),
    };
    let rot_2 = match rot_2 {
        CubePermutation::UNIT => Quat::default(),
        CubePermutation::I => Quat::from_rotation_y(std::f32::consts::PI)
            .mul_quat(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        _ => unreachable!(),
    };
    rot_0.mul_quat(rot_1).mul_quat(rot_2)
}


#[derive(Debug, Component)]
pub struct RotateAnimation {
    pub axis: Vec3,
    pub s: f32,
    pub duration: Duration,
    pub from: Quat,
    pub to: Quat,
}

pub fn rotate_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RotateAnimation, &mut Transform)>,
) {
    for (entity, mut animation, mut transform) in query.iter_mut() {
        if animation.s < 1.0 {
            animation.s += time.delta_seconds() / animation.duration.as_secs_f32();
            let rot = Quat::slerp(animation.from, animation.to, animation.s);
            transform.rotation = rot;
            if animation.s >= 1.0 {
                transform.rotation = animation.to;
                commands.entity(entity).remove::<RotateAnimation>();
            }
        }
    }
}
