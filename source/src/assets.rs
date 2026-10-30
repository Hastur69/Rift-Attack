//! Asset loading. Every texture, font and sound is embedded into the executable
//! with `include_bytes!`, so `RiftAttack.exe` runs on its own. The same files are
//! shipped loose in `assets/` so they can be inspected, remixed and rebuilt.

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams};
use macroquad::prelude::*;

macro_rules! tex {
    ($file:literal) => {{
        let t = Texture2D::from_file_with_format(
            include_bytes!(concat!("../assets/gfx/", $file)),
            Some(ImageFormat::Png),
        );
        t.set_filter(FilterMode::Nearest);
        t
    }};
}

pub struct Gfx {
    pub marine: Texture2D,
    pub weapons: Texture2D,
    pub gazer: Texture2D,
    pub maw: Texture2D,
    pub seraph: Texture2D,
    pub leviathan: Texture2D,
    pub spawnling: Texture2D,
    pub rift: Texture2D,
    pub crates: Texture2D,
    pub sky: Texture2D,
    pub far: Texture2D,
    pub mid: Texture2D,
    pub ground: Texture2D,
    pub block: Texture2D,
    pub catwalk: Texture2D,
    pub glow: Texture2D,
    pub title: Texture2D,
}

impl Gfx {
    pub fn load() -> Gfx {
        let g = Gfx {
            marine: tex!("marine.png"),
            weapons: tex!("weapons.png"),
            gazer: tex!("gazer.png"),
            maw: tex!("maw.png"),
            seraph: tex!("seraph.png"),
            leviathan: tex!("leviathan.png"),
            spawnling: tex!("spawnling.png"),
            rift: tex!("rift.png"),
            crates: tex!("crates.png"),
            sky: tex!("bg_sky.png"),
            far: tex!("bg_far.png"),
            mid: tex!("bg_mid.png"),
            ground: tex!("tile_ground.png"),
            block: tex!("tile_block.png"),
            catwalk: tex!("tile_catwalk.png"),
            glow: tex!("glow.png"),
            title: tex!("title.png"),
        };
        // smooth sampling for the soft glow & title logo
        g.glow.set_filter(FilterMode::Linear);
        g.title.set_filter(FilterMode::Linear);
        g
    }
}

pub struct Fonts {
    pub mono: Font,
    pub bold: Font,
}

impl Fonts {
    pub fn load() -> Fonts {
        let mono = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/DejaVuSansMono.ttf"))
            .expect("font");
        let bold = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/DejaVuSansMono-Bold.ttf"))
            .expect("font");
        Fonts { mono, bold }
    }
}

/// Window / taskbar icon, downsampled at startup from the embedded 256px icon.
pub fn window_icon() -> Option<miniquad::conf::Icon> {
    let img = Image::from_file_with_format(include_bytes!("../assets/gfx/icon.png"), Some(ImageFormat::Png)).ok()?;
    fn sample<const N: usize>(img: &Image, size: usize) -> [u8; N] {
        let mut out = [0u8; N];
        let (w, h) = (img.width as usize, img.height as usize);
        let step = w / size;
        for y in 0..size {
            for x in 0..size {
                let mut acc = [0u32; 4];
                let mut cnt = 0u32;
                for sy in 0..step {
                    for sx in 0..step {
                        let px = (x * step + sx).min(w - 1);
                        let py = (y * step + sy).min(h - 1);
                        let i = (py * w + px) * 4;
                        for c in 0..4 {
                            acc[c] += img.bytes[i + c] as u32;
                        }
                        cnt += 1;
                    }
                }
                let o = (y * size + x) * 4;
                for c in 0..4 {
                    out[o + c] = (acc[c] / cnt.max(1)) as u8;
                }
            }
        }
        let _ = h;
        out
    }
    Some(miniquad::conf::Icon {
        small: sample::<{ 16 * 16 * 4 }>(&img, 16),
        medium: sample::<{ 32 * 32 * 4 }>(&img, 32),
        big: sample::<{ 64 * 64 * 4 }>(&img, 64),
    })
}

// ---------------------------------------------------------------------------
// Shaders
// ---------------------------------------------------------------------------
const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const ADDITIVE_FRAG: &str = r#"#version 100
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}
"#;

/// Final screen pass: chromatic aberration, scanlines, vignette, grain,
/// reality-warp wobble, Time Rift tint and Void Sight colour inversion.
const POST_FRAG: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
uniform float time;
uniform float aberr;
uniform float invert;
uniform float rift;
uniform float warp;
uniform float grain;
uniform float redout;
uniform vec2 res;

float rand(vec2 c) { return fract(sin(dot(c, vec2(12.9898, 78.233))) * 43758.5453); }

void main() {
    vec2 p = uv;
    vec2 cc = p - 0.5;
    float d = dot(cc, cc);
    p = 0.5 + cc * (1.0 + 0.035 * d);
    p.x += sin(p.y * 38.0 + time * 6.0) * 0.0035 * warp;
    p.y += cos(p.x * 27.0 + time * 4.6) * 0.0035 * warp;
    float a = 0.0007 + aberr * 0.004;
    vec2 dir = normalize(cc + 0.0001) * a;
    vec3 col;
    col.r = texture2D(Texture, p + dir).r;
    col.g = texture2D(Texture, p).g;
    col.b = texture2D(Texture, p - dir).b;
    float sl = 0.93 + 0.07 * sin(p.y * res.y * 3.14159);
    col *= sl;
    float l = dot(col, vec3(0.3, 0.59, 0.11));
    col = mix(col, vec3(l * 0.55, l * 0.95, l * 1.3), rift * 0.75);
    col = mix(col, vec3(1.0) - col.gbr, invert);
    col = mix(col, vec3(l * 1.2, l * 0.15, l * 0.2), redout * 0.6);
    col *= 1.0 - d * (1.15 + redout * 1.2);
    col += (rand(p * res + fract(time)) - 0.5) * grain;
    if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0) col = vec3(0.0);
    gl_FragColor = vec4(col, 1.0);
}
"#;

pub struct Materials {
    pub additive: Material,
    pub post: Material,
}

impl Materials {
    pub fn load() -> Materials {
        let additive = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: ADDITIVE_FRAG },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("additive material");
        let post = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: POST_FRAG },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("aberr", UniformType::Float1),
                    UniformDesc::new("invert", UniformType::Float1),
                    UniformDesc::new("rift", UniformType::Float1),
                    UniformDesc::new("warp", UniformType::Float1),
                    UniformDesc::new("grain", UniformType::Float1),
                    UniformDesc::new("redout", UniformType::Float1),
                    UniformDesc::new("res", UniformType::Float2),
                ],
                ..Default::default()
            },
        )
        .expect("post material");
        Materials { additive, post }
    }
}
