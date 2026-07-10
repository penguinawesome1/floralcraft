override IS_DEBUG_MODE = false;

struct Config {
    max_trace_dist: f32,
    time_of_day: f32,
}

@group(1) @binding(2) var<uniform> config: Config;