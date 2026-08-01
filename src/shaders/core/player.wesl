struct Player {
    pos: vec3f,
    state_flags: u32,  // bit 0: is_grounded, bit 1: is_flying, bit 2: was_space_held
    velocity: vec3f,
    fly_timer_ms: u32,
}