import package::core::item::Item;

override IS_DEBUG_MODE = false;

struct Config {
    cam_rotation: mat4x4f,
    cam_yaw: f32,
    max_trace_dist: f32,
    time_of_day: f32,
    delta_time: f32,
    pending_action: u32, // 0=none, 1=break, 2=place
    held_item_kind: u32, // 0=none, 1=block
    held_item_id: u32,
    // 0=w or arrow up
    // 1=s or arrow down
    // 2=a or arrow left
    // 3=d or arrow right
    // 4=space
    // 5=left shift
    input_flags: u32,
}
