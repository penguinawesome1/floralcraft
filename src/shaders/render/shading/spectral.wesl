import package::core::noise;
import package::render::shading::cie_data::{CIE_TABLE_SIZE, CIE_TABLE_STEP, cie_table};

const _LAMBDA_MIN: f32 = 380.0;
const _LAMBDA_MAX: f32 = 720.0;
const _LAMBDA_RANGE: f32 = _LAMBDA_MAX - _LAMBDA_MIN;

const _XYZ_TO_LINEAR_SRGB: mat3x3f = mat3x3f(
    vec3f(3.2404542, -0.9692660, 0.0556434),
    vec3f(-1.5371385, 1.8760108, -0.2040259),
    vec3f(-0.4985314, 0.0415560, 1.0572252)
);

fn sample_hero_wavelengths(seed: vec2f) -> vec4f {
    let u = noise::hash2d_to_1d(seed);
    let hero = _LAMBDA_MIN + u * _LAMBDA_RANGE;
    let step = _LAMBDA_RANGE / 4.0;

    var w1 = hero + step;
    if w1 > _LAMBDA_MAX { w1 -= _LAMBDA_RANGE; }

    var w2 = hero + 2.0 * step;
    if w2 > _LAMBDA_MAX { w2 -= _LAMBDA_RANGE; }

    var w3 = hero + 3.0 * step;
    if w3 > _LAMBDA_MAX { w3 -= _LAMBDA_RANGE; }

    return vec4f(hero, w1, w2, w3);
}

fn _eval_spectral_coeffs(coeffs: vec3f, lambda: f32) -> f32 {
    let norm_lambda = (lambda - _LAMBDA_MIN) / _LAMBDA_RANGE;
    let x = coeffs.x * norm_lambda * norm_lambda + coeffs.y * norm_lambda + coeffs.z;
    let s = x / sqrt(1.0 + x * x); // sigmoid, maps to roughly [-1,1]
    return clamp(0.5 * s + 0.5, 0.0, 1.0); // remap to [0,1] reflectance
}

fn rgb_to_spectrum(coeffs: vec3f, wavelengths: vec4f) -> vec4f {
    return vec4f(
        _eval_spectral_coeffs(coeffs, wavelengths.x),
        _eval_spectral_coeffs(coeffs, wavelengths.y),
        _eval_spectral_coeffs(coeffs, wavelengths.z),
        _eval_spectral_coeffs(coeffs, wavelengths.w),
    );
}

fn _cie_linear_interpolation(lambda: f32) -> vec3f {
    let t = clamp((lambda - _LAMBDA_MIN) / CIE_TABLE_STEP, 0.0, f32(CIE_TABLE_SIZE - 1u));
    let i0 = u32(floor(t));
    let i1 = min(i0 + 1u, CIE_TABLE_SIZE - 1u);
    let frac = t - f32(i0);
    return mix(cie_table[i0], cie_table[i1], frac);
}

fn cie_xyz(wavelengths: vec4f, spectral: vec4f) -> vec3f {
    var xyz = vec3f(0.0);
    for (var i = 0u; i < 4u; i++) {
        xyz += _cie_linear_interpolation(wavelengths[i]) * spectral[i];
    }
    return xyz / 4.0;
}

fn xyz_to_linear_srgb(xyz: vec3f) -> vec3f {
    return max(vec3f(0.0), _XYZ_TO_LINEAR_SRGB * xyz);
}
