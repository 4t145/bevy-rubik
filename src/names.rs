use std::sync::OnceLock;

use bevy::core::Name;
use rubik::CubePosition;

pub const RUBIK_NAME: &str = "Rubik";
pub fn rubik_name() -> Name {
    Name::new(RUBIK_NAME)
}
pub fn cube_position_name(pos: CubePosition) -> Name {
    static MAP: OnceLock<[Name; 27]> = OnceLock::new();
    MAP.get_or_init(|| {
        std::array::from_fn(|index| {
            Name::new(format!(
                "{:?}",
                CubePosition::try_from_u8(index as u8).unwrap()
            ))
        })
    })[pos as usize]
        .clone()
}
