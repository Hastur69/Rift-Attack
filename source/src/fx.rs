//! Particles, shockwaves, beams, decals, floating text and screen shake.

use crate::level::GROUND_Y;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

#[derive(Clone, Copy, PartialEq)]
pub enum PKind {
    /// bright additive spark
    Spark,
    /// soft additive glow blob
    Glow,
    /// alpha-blended smoke puff that grows
    Smoke,
    /// ichor droplet: falls, leaves a decal
    Ichor,
    /// brass casing: bounces once
    Shell,
    /// shard of the void: dark square
    Shard,
}

#[derive(Clone)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max: f32,
    pub size: f32,
    pub color: Color,
    pub kind: PKind,
    pub grav: f32,
    pub drag: f32,
}

pub struct Ring {
    pub pos: Vec2,
    pub r: f32,
    pub max_r: f32,
    pub life: f32,
    pub max: f32,
    pub color: Color,
}

pub struct Beam {
    pub a: Vec2,
    pub b: Vec2,
    pub life: f32,
    pub max: f32,
    pub color: Color,
    pub width: f32,
}

pub struct Popup {
    pub pos: Vec2,
    pub text: String,
    pub life: f32,
    pub color: Color,
    pub size: f32,
}

pub struct Decal {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub color: Color,
}

pub struct Fx {
    pub parts: Vec<Particle>,
    pub rings: Vec<Ring>,
    pub beams: Vec<Beam>,
    pub popups: Vec<Popup>,
    pub decals: Vec<Decal>,
    pub trauma: f32,
    pub flash: f32,
    pub flash_col: Color,
    pub shake_enabled: bool,
}

pub fn rand_dir() -> Vec2 {
    let a = gen_range(0.0, std::f32::consts::TAU);
    vec2(a.cos(), a.sin())
}

impl Fx {
    pub fn new() -> Fx {
        Fx {
            parts: Vec::with_capacity(2048),
            rings: Vec::new(),
            beams: Vec::new(),
            popups: Vec::new(),
            decals: Vec::new(),
            trauma: 0.0,
            flash: 0.0,
            flash_col: WHITE,
            shake_enabled: true,
        }
    }

    pub fn add(&mut self, p: Particle) {
        if self.parts.len() < 3000 {
            self.parts.push(p);
        }
    }

    pub fn shake(&mut self, amt: f32) {
        self.trauma = (self.trauma + amt * 0.08).min(1.0);
    }

    pub fn flash(&mut self, c: Color, amt: f32) {
        self.flash = self.flash.max(amt);
        self.flash_col = c;
    }

    pub fn shake_offset(&self, t: f32) -> Vec2 {
        if !self.shake_enabled {
            return Vec2::ZERO;
        }
        let s = self.trauma * self.trauma * 9.0;
        vec2((t * 71.0).sin() * s + (t * 113.0).cos() * s * 0.5, (t * 89.0).cos() * s + (t * 57.0).sin() * s * 0.5)
    }

    pub fn popup(&mut self, pos: Vec2, text: impl Into<String>, color: Color, size: f32) {
        self.popups.push(Popup { pos, text: text.into(), life: 1.4, color, size });
    }

    pub fn spark(&mut self, pos: Vec2, dir: Vec2, n: usize, color: Color, speed: f32) {
        for _ in 0..n {
            let d = (dir + rand_dir() * 0.7).normalize_or_zero();
            self.add(Particle {
                pos,
                vel: d * gen_range(speed * 0.3, speed),
                life: gen_range(0.1, 0.35),
                max: 0.35,
                size: gen_range(1.0, 2.0),
                color,
                kind: PKind::Spark,
                grav: 300.0,
                drag: 3.0,
            });
        }
    }

    pub fn ichor(&mut self, pos: Vec2, dir: Vec2, n: usize, color: Color, speed: f32) {
        for _ in 0..n {
            let d = (dir * 0.6 + rand_dir()).normalize_or_zero();
            self.add(Particle {
                pos,
                vel: d * gen_range(speed * 0.2, speed),
                life: gen_range(0.6, 1.6),
                max: 1.6,
                size: gen_range(1.0, 3.0),
                color,
                kind: PKind::Ichor,
                grav: 520.0,
                drag: 0.6,
            });
        }
    }

    pub fn smoke(&mut self, pos: Vec2, vel: Vec2, size: f32, life: f32, c: Color) {
        self.add(Particle { pos, vel, life, max: life, size, color: c, kind: PKind::Smoke, grav: -15.0, drag: 1.5 });
    }

    pub fn glow(&mut self, pos: Vec2, size: f32, life: f32, c: Color) {
        self.add(Particle { pos, vel: Vec2::ZERO, life, max: life, size, color: c, kind: PKind::Glow, grav: 0.0, drag: 0.0 });
    }

    pub fn shell(&mut self, pos: Vec2, facing: f32) {
        self.add(Particle {
            pos,
            vel: vec2(-facing * gen_range(40.0, 90.0), gen_range(-160.0, -90.0)),
            life: 1.2,
            max: 1.2,
            size: 1.0,
            color: Color::from_rgba(220, 180, 80, 255),
            kind: PKind::Shell,
            grav: 700.0,
            drag: 0.2,
        });
    }

    pub fn explosion(&mut self, pos: Vec2, radius: f32, tint: Color) {
        self.glow(pos, radius * 3.2, 0.35, Color::new(1.0, 0.7, 0.35, 1.0));
        self.glow(pos, radius * 1.8, 0.2, Color::new(1.0, 0.95, 0.8, 1.0));
        self.rings.push(Ring { pos, r: 4.0, max_r: radius * 1.5, life: 0.35, max: 0.35, color: tint });
        for _ in 0..(radius as usize / 2 + 10) {
            let d = rand_dir();
            self.add(Particle {
                pos: pos + d * gen_range(0.0, radius * 0.4),
                vel: d * gen_range(40.0, radius * 6.0),
                life: gen_range(0.25, 0.7),
                max: 0.7,
                size: gen_range(1.0, 3.0),
                color: Color::new(1.0, gen_range(0.5, 0.9), 0.2, 1.0),
                kind: PKind::Spark,
                grav: 200.0,
                drag: 3.5,
            });
        }
        for _ in 0..(radius as usize / 4 + 6) {
            let d = rand_dir();
            self.smoke(pos + d * gen_range(0.0, radius * 0.5), d * gen_range(10.0, 60.0), gen_range(radius * 0.25, radius * 0.6), gen_range(0.6, 1.4), Color::new(0.12, 0.08, 0.14, 0.7));
        }
        self.shake(radius / 9.0);
    }

    pub fn update(&mut self, dt: f32) {
        self.trauma = (self.trauma - dt * 1.4).max(0.0);
        self.flash = (self.flash - dt * 3.0).max(0.0);
        let mut new_decals = Vec::new();
        for p in self.parts.iter_mut() {
            p.life -= dt;
            p.vel.y += p.grav * dt;
            p.vel *= 1.0 / (1.0 + p.drag * dt);
            p.pos += p.vel * dt;
            match p.kind {
                PKind::Ichor => {
                    if p.pos.y >= GROUND_Y {
                        p.life = 0.0;
                        new_decals.push(Decal { x: p.pos.x, y: GROUND_Y + gen_range(0.0, 2.0), w: p.size * 2.0 + gen_range(0.0, 3.0), color: p.color });
                    }
                }
                PKind::Shell => {
                    if p.pos.y >= GROUND_Y - 1.0 && p.vel.y > 0.0 {
                        p.pos.y = GROUND_Y - 1.0;
                        p.vel.y *= -0.35;
                        p.vel.x *= 0.5;
                    }
                }
                PKind::Smoke => p.size += dt * 6.0,
                _ => {}
            }
        }
        self.parts.retain(|p| p.life > 0.0);
        for d in new_decals {
            if self.decals.len() > 400 {
                self.decals.remove(0);
            }
            self.decals.push(d);
        }
        for r in self.rings.iter_mut() {
            r.life -= dt;
            let k = 1.0 - (r.life / r.max).max(0.0);
            r.r = r.max_r * (1.0 - (1.0 - k).powi(3));
        }
        self.rings.retain(|r| r.life > 0.0);
        for b in self.beams.iter_mut() {
            b.life -= dt;
        }
        self.beams.retain(|b| b.life > 0.0);
        for p in self.popups.iter_mut() {
            p.life -= dt;
            p.pos.y -= dt * 22.0;
        }
        self.popups.retain(|p| p.life > 0.0);
    }

    pub fn draw_decals(&self) {
        for d in &self.decals {
            draw_rectangle(d.x - d.w * 0.5, d.y, d.w, 1.0, Color::new(d.color.r * 0.6, d.color.g * 0.6, d.color.b * 0.6, 0.8));
        }
    }

    /// Normal-blended particles.
    pub fn draw_normal(&self) {
        for p in &self.parts {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            match p.kind {
                PKind::Smoke => {
                    let c = Color::new(p.color.r, p.color.g, p.color.b, p.color.a * a);
                    draw_circle(p.pos.x, p.pos.y, p.size, c);
                }
                PKind::Ichor => draw_rectangle(p.pos.x - p.size * 0.5, p.pos.y - p.size * 0.5, p.size, p.size, p.color),
                PKind::Shell => draw_rectangle(p.pos.x, p.pos.y, 2.0, 1.0, p.color),
                PKind::Shard => {
                    let c = Color::new(p.color.r, p.color.g, p.color.b, a);
                    draw_rectangle(p.pos.x - p.size * 0.5, p.pos.y - p.size * 0.5, p.size, p.size, c);
                }
                _ => {}
            }
        }
        for r in &self.rings {
            let a = (r.life / r.max).clamp(0.0, 1.0);
            draw_circle_lines(r.pos.x, r.pos.y, r.r, 2.0 + 3.0 * a, Color::new(r.color.r, r.color.g, r.color.b, a * 0.8));
        }
    }

    /// Additive particles - drawn with the additive material active.
    pub fn draw_additive(&self, glow: &Texture2D) {
        for p in &self.parts {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            match p.kind {
                PKind::Spark => {
                    let c = Color::new(p.color.r, p.color.g, p.color.b, a);
                    let tail = p.pos - p.vel * 0.015;
                    draw_line(tail.x, tail.y, p.pos.x, p.pos.y, p.size, c);
                }
                PKind::Glow => {
                    let s = p.size * (0.6 + 0.4 * a);
                    let c = Color::new(p.color.r, p.color.g, p.color.b, a * p.color.a);
                    draw_texture_ex(glow, p.pos.x - s * 0.5, p.pos.y - s * 0.5, c, DrawTextureParams { dest_size: Some(vec2(s, s)), ..Default::default() });
                }
                _ => {}
            }
        }
        for b in &self.beams {
            let a = (b.life / b.max).clamp(0.0, 1.0);
            let c = Color::new(b.color.r, b.color.g, b.color.b, a);
            draw_line(b.a.x, b.a.y, b.b.x, b.b.y, b.width * (0.4 + a), c);
            draw_line(b.a.x, b.a.y, b.b.x, b.b.y, b.width * 3.0 * a, Color::new(c.r, c.g, c.b, a * 0.35));
            draw_line(b.a.x, b.a.y, b.b.x, b.b.y, 1.0, Color::new(1.0, 1.0, 1.0, a));
        }
    }
}

pub fn glow_at(glow: &Texture2D, pos: Vec2, size: f32, c: Color) {
    draw_texture_ex(glow, pos.x - size * 0.5, pos.y - size * 0.5, c, DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() });
}
