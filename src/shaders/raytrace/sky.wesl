import package::config::config;

const PI = 3.1415;

fn sun_dir() -> vec3f {
    let angle = (config.time_of_day - 0.25) * 2.0 * PI;
    let elevation = sin(angle);
    let horizontal = cos(angle);
    return normalize(vec3f(horizontal, elevation, 0.0));
}

fn color(ray_dir: vec3f) -> vec3f {
    return vec3f(0.7, 0.75, 0.95);
}
