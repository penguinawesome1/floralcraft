override IS_DEBUG_MODE = false;

struct Config {
    cam_rotation: mat4x4f,
    cam_yaw: f32,
    max_trace_dist: f32,
    time_of_day: f32,
    delta_time: f32,
    // bits 0-1: action (0=none, 1=break, 2=place)
    // bit 2: arrow up
    // bit 3: arrow down
    // bit 4: arrow left
    // bit 5: arrow right
    // bit 6: space
    // bit 7: left shift
    pending_action: u32,
}
