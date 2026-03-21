pub mod dictionary;

use aether::prelude::*;
use bevy::prelude::Resource;

world! {
    #[derive(Resource)]
    ==
    [16, 16, 16; 16],
    block: u8,
    sky_light: u8,
    is_exposed: bool,
}
