use terrain_data::prelude::world;

world! {
    chunk_width: 16,
    chunk_height: 16,
    subchunk_depth: 16,
    num_subchunks: 16,
    Block r#as block: u8 = 4,
    BlockLight r#as block_light: u8 = 4,
    SkyLight r#as sky_light: u8 = 4,
    Exposed r#as is_exposed: bool = 1,
}
