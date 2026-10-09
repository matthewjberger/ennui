#define_import_path ennui::quad

const NO_PICTURE: u32 = 0xFFFFFFFFu;
const FIELD: f32 = 16.0;
const TAU: f32 = 6.2831853;

struct Quad {
    center: vec2<f32>,
    size: vec2<f32>,
    color: vec4<f32>,
    edge: vec4<f32>,
    shape: vec3<f32>,
    clip: array<f32, 4>,
    picture: array<u32, 4>,
    uv: array<f32, 4>,
    shadow: array<f32, 4>,
    blur: array<f32, 4>,
    effect: array<f32, 4>,
    depth: f32,
}

struct Placed {
    at: vec2<f32>,
    local: vec2<f32>,
    uv: vec2<f32>,
}

fn four(held: array<f32, 4>) -> vec4<f32> {
    return vec4<f32>(held[0], held[1], held[2], held[3]);
}

fn shadow_margin(shadow: f32, blur: vec4<f32>) -> f32 {
    if shadow <= 0.0 {
        return 0.0;
    }
    return blur.z * 1.5 + blur.w + max(abs(blur.x), abs(blur.y));
}

fn quad_placed(quad: Quad, corner: u32) -> Placed {
    let offset = vec2<f32>(f32(corner & 1u), f32(corner >> 1u)) * 2.0 - 1.0;
    let margin = shadow_margin(quad.shadow[3], four(quad.blur));
    let outer = quad.size + vec2<f32>(margin, margin);
    let turn = quad.effect[3];
    let sine = sin(turn);
    let cosine = cos(turn);
    let reach = offset * outer;
    var placed: Placed;
    placed.at = quad.center + vec2<f32>(
        reach.x * cosine - reach.y * sine,
        reach.x * sine + reach.y * cosine,
    );
    placed.local = reach;
    let share = vec2<f32>(offset.x * 0.5 + 0.5, 0.5 - offset.y * 0.5);
    placed.uv = mix(vec2<f32>(quad.uv[0], quad.uv[1]), vec2<f32>(quad.uv[2], quad.uv[3]), share);
    return placed;
}

fn spread_coverage(away: f32, sigma: f32) -> f32 {
    let held = away / max(sigma, 0.00001);
    return 1.0 / (1.0 + exp(1.5976 * held + 0.070565 * held * held * held));
}

fn rounded(at: vec2<f32>, half: vec2<f32>, round: f32) -> f32 {
    let held = min(round, min(half.x, half.y));
    let apart = abs(at) - half + vec2<f32>(held, held);
    return length(max(apart, vec2<f32>(0.0, 0.0))) + min(max(apart.x, apart.y), 0.0) - held;
}

fn quad_clipped(quad: Quad, at: vec2<f32>) -> bool {
    let clip = four(quad.clip);
    if clip.z <= 0.0 {
        return false;
    }
    let apart = abs(at - clip.xy);
    return apart.x > clip.z || apart.y > clip.w;
}

fn quad_inside(quad: Quad, placed: Placed) -> bool {
    if quad_clipped(quad, placed.at) {
        return false;
    }
    return rounded(placed.local, quad.size, quad.shape.x) <= 0.0;
}

fn quad_spots(uv: vec2<f32>, across: vec2<f32>, down: vec2<f32>) -> array<vec2<f32>, 4> {
    let near = across * 0.125 + down * 0.375;
    let far = across * 0.375 - down * 0.125;
    return array<vec2<f32>, 4>(uv + near, uv - near, uv + far, uv - far);
}

fn quad_covered(spread: vec4<f32>, level: f32, step: f32) -> f32 {
    return dot(
        smoothstep(vec4<f32>(level - step), vec4<f32>(level + step), spread),
        vec4<f32>(0.25),
    );
}

fn quad_color(quad: Quad, placed: Placed, held: vec4<f32>, spread: vec4<f32>) -> vec4<f32> {
    if quad_clipped(quad, placed.at) {
        discard;
    }
    let away = rounded(placed.local, quad.size, quad.shape.x);
    let feather = max(fwidth(away), 0.000001);
    let held_step = max(fwidth(held.a), 0.0001) * 0.5;
    let disc_feather = fwidth(length(placed.local / max(quad.size, vec2<f32>(0.00001, 0.00001)))) + 0.00001;
    let inside = clamp(0.5 - away / feather, 0.0, 1.0);
    let shadow = four(quad.shadow);
    if inside <= 0.0 && shadow.a <= 0.0 {
        discard;
    }
    var color = vec4<f32>(to_linear(quad.color.rgb), quad.color.a);
    let effect = four(quad.effect);
    if effect.x > 0.5 {
        color = shaded(placed.local, quad.size, color, effect, disc_feather);
    }
    let blur = four(quad.blur);
    if quad.picture[0] != NO_PICTURE {
        if quad.shape.z > 0.5 {
            let step = held_step + blur.y / FIELD;
            let fill = quad_covered(spread, 0.5, step);
            if blur.x > 0.0 && shadow.a <= 0.0 {
                let rim = 0.5 - blur.x / FIELD;
                let ringed = quad_covered(spread, rim, step);
                let edge = vec4<f32>(to_linear(quad.edge.rgb), quad.edge.a);
                color = vec4<f32>(
                    mix(edge.rgb, color.rgb, fill),
                    mix(edge.a * ringed, color.a, fill),
                );
            } else {
                color = vec4<f32>(color.rgb, color.a * fill);
            }
        } else {
            color = vec4<f32>(color.rgb * held.rgb, color.a * held.a);
        }
    }
    let border = quad.shape.y;
    if border > 0.0 {
        let rim = clamp(0.5 - (away + border) / feather, 0.0, 1.0);
        let edge = vec4<f32>(to_linear(quad.edge.rgb), quad.edge.a);
        color = mix(edge, color, rim);
    }
    let covered = color.a * inside;
    if shadow.a > 0.0 {
        let thrown = rounded(
            placed.local - blur.xy,
            quad.size + vec2<f32>(blur.w, blur.w),
            quad.shape.x + blur.w,
        );
        let sigma = max(blur.z, 0.0005) * 0.5;
        let under = spread_coverage(thrown, sigma) * shadow.a * (1.0 - inside);
        let total = covered + under;
        if total < 0.001 {
            discard;
        }
        let mixed = color.rgb * covered + to_linear(shadow.rgb) * under;
        return vec4<f32>(mixed / total, total);
    }
    if covered < 0.001 {
        discard;
    }
    return vec4<f32>(color.rgb, covered);
}

fn hue_to_rgb(hue: f32, saturation: f32, value: f32) -> vec3<f32> {
    let held = abs(fract(vec3<f32>(hue) + vec3<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0);
    return value * mix(vec3<f32>(1.0), clamp(held - vec3<f32>(1.0), vec3<f32>(0.0), vec3<f32>(1.0)), saturation);
}

fn shaded(local: vec2<f32>, half: vec2<f32>, base: vec4<f32>, effect: vec4<f32>, feather: f32) -> vec4<f32> {
    let unit = local / max(half, vec2<f32>(0.00001, 0.00001));
    if effect.x < 1.5 {
        let away = length(unit);
        let saturation = clamp(away, 0.0, 1.0);
        let hue = fract(atan2(unit.y, unit.x) / TAU + 1.0);
        let disc = 1.0 - smoothstep(1.0 - feather, 1.0, away);
        let shade = hue_to_rgb(hue, saturation, clamp(effect.y, 0.0, 1.0));
        return vec4<f32>(to_linear(shade), base.a * disc);
    }
    let saturation = clamp(unit.x * 0.5 + 0.5, 0.0, 1.0);
    let value = clamp(0.5 - unit.y * 0.5, 0.0, 1.0);
    return vec4<f32>(to_linear(hue_to_rgb(clamp(effect.y, 0.0, 1.0), saturation, value)), base.a);
}

fn to_linear(shade: vec3<f32>) -> vec3<f32> {
    let low = shade / 12.92;
    let high = pow((shade + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, shade <= vec3<f32>(0.04045));
}
