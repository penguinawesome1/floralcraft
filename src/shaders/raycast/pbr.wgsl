#include "./dda.wgsl"

const pbr__PI = 3.1415;
const pbr__EPS = 1e-6;

struct pbr__BlockMaterial {
    albedo: vec3f,
    roughness: f32,
    metallic: f32,
}

fn _pbr__distribution_GGX(N: vec3f, H: vec3f, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (pbr__PI * denom * denom);
}

fn _pbr__geometry_smith(N: vec3f, V: vec3f, L: vec3f, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let gv = NdotV / (NdotV * (1.0 - k) + k);
    let gl = NdotL / (NdotL * (1.0 - k) + k);
    return gv * gl;
}

fn _pbr__fresnel_schlick(V: vec3f, H: vec3f, F0: vec3f) -> vec3f {
    let VdotH = max(dot(V, H), 0.0);
    return F0 + (1.0 - F0) * pow(1.0 - VdotH, 5.0);
}

fn _pbr__block_material(id: u32) -> pbr__BlockMaterial {
    switch id {
        case 1: { return pbr__BlockMaterial(vec3f(1.0, 0.08, 0.58), 0.5, 1.0); }
        case 2: { return pbr__BlockMaterial(vec3f(0.18, 1.0, 1.0), 0.5, 1.0); }
        case 3: { return pbr__BlockMaterial(vec3f(0.96, 0.96, 0.45), 0.5, 0.5); }
        case 4: { return pbr__BlockMaterial(vec3f(0.96, 0.30, 0.16), 0.5, 0.5); }
        case 5: { return pbr__BlockMaterial(vec3f(1.0, 0.5, 0.31), 0.5, 0.5); }
        default: { return pbr__BlockMaterial(vec3f(0.5, 0.0, 1.0), 0.5, 0.5); }
    }
}

fn pbr__get(dir: vec3f, res: dda__TraceResult) -> vec4f {
    let material = _pbr__block_material(res.block_id);

    let N = dda__axis_normal(res.last_hit_axis, dir);
    let V = -dir;
    let L = normalize(vec3f(1.0, 2.0, 1.0));
    let H = normalize(V + L);

    let shadow_origin = res.hit_pos + N * pbr__EPS * 10.0;
    let shadow_ray = dda__Ray(shadow_origin, L);
    let shadow_res = dda__trace(shadow_ray);
    let in_shadow = shadow_res.block_id != 0u;

    let F0 = mix(vec3f(0.04), material.albedo, material.metallic);
    let F = _pbr__fresnel_schlick(V, H, F0);
    let D = _pbr__distribution_GGX(N, H, material.roughness);
    let G = _pbr__geometry_smith(N, V, L, material.roughness);

    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, V), 0.0);

    let specular = (D * G * F) / max(4.0 * NdotV * NdotL, pbr__EPS);
    let kD = (1.0 - F) * (1.0 - material.metallic);
    let diffuse = kD * material.albedo / pbr__PI;

    let ambient = vec3f(0.03) * material.albedo;
    let direct = select((diffuse + specular) * NdotL, vec3f(0.0), in_shadow);
    let Lo = ambient + direct;

    let mapped = Lo / (Lo + vec3f(1.0));
    let out = pow(mapped, vec3f(1.0 / 2.2));

    return vec4f(out, 1.0);
}