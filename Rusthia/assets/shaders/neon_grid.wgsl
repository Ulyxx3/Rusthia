// assets/shaders/neon_grid.wgsl
// ==============================================================================
// Rusthia — Shader de fond neon animé
// Traduction de sky_noise_waves.gdshader + noise_waves.gdshader
//
// Algorithme :
//   1. Simplex 3D Noise (Ian McEwan / Stefan Gustavson) sur la position 3D + temps
//   2. Lignes primaires : mod(floor(noise * 96), 20) == 0
//   3. Lignes secondaires : mod(floor(noise * 384), 20) == 0
//   4. Cyclage de couleur rouge/violet via sin(time / 70)
// ==============================================================================

#import bevy_pbr::forward_io::VertexOutput

// Uniform passé depuis Rust (temps et opacité)
@group(2) @binding(0)
var<uniform> time: f32;

@group(2) @binding(1)
var<uniform> opacity: f32;

// ==============================================================================
// SIMPLEX 3D NOISE — Ian McEwan / Stefan Gustavson
// Traduction WGSL fidèle du gdshader original
// ==============================================================================

fn permute4(x: vec4<f32>) -> vec4<f32> {
    return (((x * 34.0) + 1.0) * x) % vec4<f32>(289.0);
}

fn taylor_inv_sqrt4(r: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(1.79284291400159) - vec4<f32>(0.85373472095314) * r;
}

fn snoise3(v_in: vec3<f32>) -> f32 {
    let C = vec2<f32>(1.0 / 6.0, 1.0 / 3.0);
    let D = vec4<f32>(0.0, 0.5, 1.0, 2.0);

    // First corner
    let i  = floor(v_in + dot(v_in, C.yyy));
    let x0 = v_in - i + dot(i, C.xxx);

    // Other corners
    let g  = step(x0.yzx, x0.xyz);
    let l  = vec3<f32>(1.0) - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    let x1 = x0 - i1 + C.xxx;
    let x2 = x0 - i2 + 2.0 * C.xxx;
    let x3 = x0 - vec3<f32>(1.0) + 3.0 * C.xxx;

    // Permutations
    let i_mod = i % vec3<f32>(289.0);
    let p = permute4(
        permute4(
            permute4(i_mod.z + vec4<f32>(0.0, i1.z, i2.z, 1.0))
            + i_mod.y + vec4<f32>(0.0, i1.y, i2.y, 1.0)
        )
        + i_mod.x + vec4<f32>(0.0, i1.x, i2.x, 1.0)
    );

    // Gradients (N*N points uniformly over a square, mapped onto an octahedron)
    let n_  = 1.0 / 7.0;
    let ns  = n_ * D.wyz - D.xzx;

    let j  = p - 49.0 * floor(p * ns.z * ns.z);

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7.0 * x_);

    let x  = x_ * ns.x + ns.yyyy;
    let y  = y_ * ns.x + ns.yyyy;
    let h  = vec4<f32>(1.0) - abs(x) - abs(y);

    let b0 = vec4<f32>(x.xy, y.xy);
    let b1 = vec4<f32>(x.zw, y.zw);

    let s0 = floor(b0) * 2.0 + vec4<f32>(1.0);
    let s1 = floor(b1) * 2.0 + vec4<f32>(1.0);
    let sh = -step(h, vec4<f32>(0.0));

    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3<f32>(a0.xy, h.x);
    var p1 = vec3<f32>(a0.zw, h.y);
    var p2 = vec3<f32>(a1.xy, h.z);
    var p3 = vec3<f32>(a1.zw, h.w);

    // Normalise gradients
    let norm = taylor_inv_sqrt4(vec4<f32>(
        dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)
    ));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;

    // Mix final noise value
    let m = max(
        0.6 - vec4<f32>(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)),
        vec4<f32>(0.0)
    );
    let m2 = m * m;

    return 42.0 * dot(
        m2 * m2,
        vec4<f32>(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3))
    );
}

// ==============================================================================
// FRAGMENT SHADER PRINCIPAL — traduction de sky_noise_waves.gdshader void sky()
// ==============================================================================

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos = in.world_position.xyz;

    // Traduction de sky_noise_waves.gdshader L91-94
    pos.y = abs(pos.y);
    pos.y += time / 70.0;

    // Calculer le noise
    let noise_raw = (snoise3(pos) + 1.0) / 2.0;
    let noise = clamp(noise_raw, 0.0, 1.0);

    // Lignes primaires (sky_noise_waves.gdshader L99-101)
    let primary   = (floor(noise * 96.0)  % 20.0) == 0.0;
    let secondary = (floor(noise * 384.0) % 20.0) == 0.0;
    let t = select(select(0.0, 0.2, secondary), 1.0, primary);

    // Cyclage de couleur via sine (L102)
    let sine = sin(time / 70.0) * 0.5 + 0.5;

    // Mélange couleur rouge-violet (adapté de la palette SoundSpace+)
    // sky_noise_waves.gdshader L106: color = mix(vec3(1,0.1,noise/4), imageColor*1.5, imageColor.a)
    let color_a = vec3<f32>(1.0, 0.1, noise / 4.0);   // rouge avec teinte noise
    let color_b = vec3<f32>(0.3, 0.0, 1.0);            // violet profond
    let base_color = mix(color_a, color_b, sine);

    // Intensité finale avec les deux vagues (L107)
    // Des valeurs >1 sont possibles ici pour le HDR bloom
    let intensity = (t + (1.0 - pow(noise, 100.0)) / 50.0) * opacity;
    var color = mix(vec3<f32>(0.0), base_color, intensity);

    // Color swap cyclique — noise_waves.gdshader L95:
    // COLOR = mix(COLOR, COLOR.bgra, sin(TIME / 10.0) / 2.0 + 0.5)
    let swap_factor = sin(time / 10.0) * 0.5 + 0.5;
    color = mix(color, color.bgr, swap_factor);

    // Fade sur les bords de l'écran (L113)
    let edge_fade = min(1.0, 2.0 * 1.0 - abs(in.world_position.y * 0.1));
    color = mix(vec3<f32>(0.0), color, max(0.0, edge_fade));

    return vec4<f32>(color, 1.0);
}
