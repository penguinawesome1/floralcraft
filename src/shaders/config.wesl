override IS_DEBUG_MODE = false;

struct Config {
    max_trace_dist: f32,
    time_of_day: f32,
    pending_action: u32, // 0 = none, 1 = break, 2 = place
}
