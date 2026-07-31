import package::core::noise;

const PI = 3.14159265;

fn _direction_basis(N: vec3f) -> mat3x3f {
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(N.y) > 0.99);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);
    return mat3x3f(tangent, bitangent, N);
}

fn _sample_cosine_hemisphere(N: vec3f, seed: vec2f) -> vec3f {
    let u1 = noise::hash2d_to_1d(seed);
    let u2 = noise::hash2d_to_1d(seed + vec2f(13.37, 42.11));

    let r = sqrt(u1);
    let phi = 2.0 * PI * u2;

    let local_dir = vec3f(r * cos(phi), r * sin(phi), sqrt(max(0.0, 1.0 - u1)));
    let basis = _direction_basis(N);
    return normalize(basis * local_dir);
}

fn sample_rough_bounce(curr_dir: vec3f, N: vec3f, roughness: f32, seed: vec2f) -> vec3f {
    let spec_dir = reflect(curr_dir, N);
    let diff_dir = _sample_cosine_hemisphere(N, seed);
    return normalize(mix(spec_dir, diff_dir, roughness * roughness));
}

fn distribution_GGX(N: vec3f, H: vec3f, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let NdotH = max(dot(N, H), 0.0);
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

fn geometry_smith(N: vec3f, V: vec3f, L: vec3f, alpha: f32) -> f32 {
    let k = alpha * 0.5;
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let gv = NdotV / (NdotV * (1.0 - k) + k);
    let gl = NdotL / (NdotL * (1.0 - k) + k);
    return gv * gl;
}

fn fresnel_schlick_spectral(V: vec3f, H: vec3f, F0: vec4f) -> vec4f {
    let VdotH = max(dot(V, H), 0.0);
    return F0 + (vec4f(1.0) - F0) * pow(1.0 - VdotH, 5.0);
}