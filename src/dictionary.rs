use blueprint::bake_toml;

bake_toml! {
    "assets/blocks.toml",
    u32,
    is_hoverable: 1,
    is_visible: 1,
    is_breakable: 1,
    is_collidable: 1,
    is_replaceable: 1,
    is_transparent: 1,
}
