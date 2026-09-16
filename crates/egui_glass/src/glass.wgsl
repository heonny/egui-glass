// Liquid Glass fragment shader. One instance = one rounded rect drawn as a
// full-viewport triangle; egui-wgpu sets the viewport to the callback rect.
// All coordinates are physical pixels (framebuffer space).

struct Uniforms {
    rect_min: vec2<f32>,
    rect_max: vec2<f32>,
    bd_min: vec2<f32>,     // where the full backdrop image is mapped on screen
    bd_max: vec2<f32>,
    tint: vec4<f32>,       // straight alpha
    radius: f32,
    blur: f32,             // blur radius in px
    refraction: f32,       // max edge displacement in px
    edge_width: f32,       // width of the lens zone in px
    chroma: f32,
    brightness: f32,
    saturation: f32,
    specular: f32,
    border: f32,
    shadow: f32,
    shadow_radius: f32,
    srgb_out: f32,
    light_dir: vec2<f32>,
    max_lod: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var bd_tex: texture_2d<f32>;
@group(0) @binding(2) var bd_samp: sampler;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // Full-viewport triangle.
    let x = f32(i32(i & 1u) * 4 - 1);
    let y = f32(i32(i >> 1u) * 4 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

fn sd_rounded_box(p: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

fn sample_backdrop(p: vec2<f32>, lod: f32) -> vec3<f32> {
    let uv = (p - u.bd_min) / (u.bd_max - u.bd_min);
    // 5-tap rotated cross on top of the mip level to hide box artifacts.
    let s = max(u.blur, 0.0) * 0.5 / (u.bd_max - u.bd_min);
    let c = textureSampleLevel(bd_tex, bd_samp, uv, lod).rgb;
    let a = textureSampleLevel(bd_tex, bd_samp, uv + vec2<f32>( s.x,  s.y * 0.5), lod).rgb;
    let b = textureSampleLevel(bd_tex, bd_samp, uv + vec2<f32>(-s.x, -s.y * 0.5), lod).rgb;
    let d = textureSampleLevel(bd_tex, bd_samp, uv + vec2<f32>( s.x * 0.5, -s.y), lod).rgb;
    let e = textureSampleLevel(bd_tex, bd_samp, uv + vec2<f32>(-s.x * 0.5,  s.y), lod).rgb;
    return (c * 2.0 + a + b + d + e) / 6.0;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let p = frag.xy;
    let center = (u.rect_min + u.rect_max) * 0.5;
    let half = (u.rect_max - u.rect_min) * 0.5;
    let r = min(u.radius, min(half.x, half.y));

    let d = sd_rounded_box(p - center, half, r);
    let mask = 1.0 - smoothstep(-0.5, 0.5, d);

    // Shadow (outside the shape only), offset downwards.
    let sh_off = vec2<f32>(0.0, u.shadow_radius * 0.35);
    let ds = sd_rounded_box(p - center - sh_off, half, r);
    let sh_t = 1.0 - smoothstep(-u.shadow_radius * 0.25, u.shadow_radius, ds);
    let shadow = u.shadow * sh_t * sh_t * (1.0 - mask); // quadratic tail: soft, mostly near the edge

    if (mask <= 0.0 && shadow <= 0.0) {
        discard;
    }

    // Outward normal via finite differences of the SDF.
    let eps = 1.0;
    let n = normalize(vec2<f32>(
        sd_rounded_box(p - center + vec2<f32>(eps, 0.0), half, r) - sd_rounded_box(p - center - vec2<f32>(eps, 0.0), half, r),
        sd_rounded_box(p - center + vec2<f32>(0.0, eps), half, r) - sd_rounded_box(p - center - vec2<f32>(0.0, eps), half, r),
    ) + vec2<f32>(1e-5, 0.0));

    // Lens profile: 0 deep inside, 1 at the edge; rounded-glass falloff.
    let t = clamp((d + u.edge_width) / max(u.edge_width, 1.0), 0.0, 1.0);
    let lens = 1.0 - sqrt(max(1.0 - t * t, 0.0));
    let disp = n * u.refraction * lens;

    let lod = clamp(log2(max(u.blur * 0.5, 1.0)), 0.0, u.max_lod);
    var col: vec3<f32>;
    if (u.chroma > 0.001) {
        let k = u.chroma * 0.5;
        col = vec3<f32>(
            sample_backdrop(p + disp * (1.0 + k), lod).r,
            sample_backdrop(p + disp, lod).g,
            sample_backdrop(p + disp * (1.0 - k), lod).b,
        );
    } else {
        col = sample_backdrop(p + disp, lod);
    }

    // Vibrancy: saturation + brightness, then tint.
    let lum = dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));
    col = mix(vec3<f32>(lum), col, u.saturation) * u.brightness;
    col = mix(col, u.tint.rgb, u.tint.a);

    // Specular rim: strong towards the light, faint counter-rim opposite.
    let ndl = dot(n, u.light_dir);
    let rim = pow(t, 5.0) * (max(ndl, 0.0) + 0.35 * max(-ndl, 0.0) + 0.08);
    let sheen = 0.05 * (1.0 - clamp((p.y - u.rect_min.y) / max(half.y * 2.0, 1.0), 0.0, 1.0));
    col += vec3<f32>(1.0) * u.specular * (rim + sheen);

    // Thin inner border, brighter on the lit side.
    let ring = 1.0 - smoothstep(0.0, 1.5, abs(d + 0.9));
    let ring_a = u.border * ring * (0.25 + 0.55 * max(ndl, 0.0) + 0.15 * max(-ndl, 0.0));
    col = mix(col, vec3<f32>(1.0), ring_a);

    col = clamp(col, vec3<f32>(0.0), vec3<f32>(1.0));
    if (u.srgb_out > 0.5) {
        col = srgb_to_linear(col);
    }

    // Premultiplied output: glass over shadow.
    let alpha = mask + shadow;
    return vec4<f32>(col * mask, alpha);
}
