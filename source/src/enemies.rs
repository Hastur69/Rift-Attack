//! The things that come through the rifts.

use crate::assets::Gfx;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// The basic horror - stands in for Heli Attack 2's helicopter.
    Gazer,
    /// Heavy - lobs acid and drops spawnlings.
    Maw,
    /// Fast - fans of light, blinks across the sky.
    Seraph,
    /// Boss - THE HOLLOW CROWN.
    Leviathan,
    /// Ground crawler dropped by Maws.
    Spawnling,
}

pub struct KindInfo {
    pub name: &'static str,
    pub hp: f32,
    pub radius: f32,
    /// sprite frame size
    pub fw: f32,
    pub fh: f32,
    /// hitbox center inside the frame
    pub anchor: Vec2,
    pub frames: u32,
    pub fps: f32,
    pub score: u64,
    pub ichor: Color,
}

pub fn info(k: Kind) -> KindInfo {
    match k {
        Kind::Gazer => KindInfo { name: "GAZER", hp: 120.0, radius: 21.0, fw: 64.0, fh: 72.0, anchor: vec2(32.0, 27.0), frames: 8, fps: 9.0, score: 100, ichor: Color::from_rgba(190, 255, 60, 255) },
        Kind::Maw => KindInfo { name: "MAW", hp: 260.0, radius: 31.0, fw: 88.0, fh: 80.0, anchor: vec2(44.0, 33.0), frames: 8, fps: 7.0, score: 250, ichor: Color::from_rgba(255, 140, 40, 255) },
        Kind::Seraph => KindInfo { name: "SERAPH", hp: 170.0, radius: 24.0, fw: 80.0, fh: 80.0, anchor: vec2(40.0, 38.0), frames: 8, fps: 12.0, score: 200, ichor: Color::from_rgba(120, 220, 255, 255) },
        Kind::Leviathan => KindInfo { name: "THE HOLLOW CROWN", hp: 2400.0, radius: 60.0, fw: 192.0, fh: 160.0, anchor: vec2(96.0, 68.0), frames: 8, fps: 6.0, score: 5000, ichor: Color::from_rgba(255, 60, 140, 255) },
        Kind::Spawnling => KindInfo { name: "SPAWNLING", hp: 14.0, radius: 6.0, fw: 16.0, fh: 12.0, anchor: vec2(8.0, 7.0), frames: 4, fps: 12.0, score: 10, ichor: Color::from_rgba(150, 220, 60, 255) },
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BossMove {
    Drift,
    Beam,
    Rain,
    Summon,
}

pub struct Enemy {
    pub id: u32,
    pub kind: Kind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    pub t: f32,
    /// 0..1 while emerging from the rift
    pub emerge: f32,
    pub fire_t: f32,
    pub burst: u32,
    pub burst_t: f32,
    pub hover: Vec2,
    pub retarget_t: f32,
    pub flash: f32,
    /// time since harpooned (None if free)
    pub harpooned: Option<f32>,
    pub dead: bool,
    pub facing: f32,
    pub look: Vec2,
    // boss
    pub boss_move: BossMove,
    pub move_t: f32,
    pub beam_x: f32,
    pub beam_dir: f32,
    // spawnling
    pub on_ground: bool,
    pub life: f32,
}

impl Enemy {
    pub fn new(id: u32, kind: Kind, pos: Vec2, hp_scale: f32) -> Enemy {
        let inf = info(kind);
        let hp = inf.hp * hp_scale;
        Enemy {
            id,
            kind,
            pos,
            vel: Vec2::ZERO,
            hp,
            max_hp: hp,
            t: macroquad::rand::gen_range(0.0, 10.0),
            emerge: if kind == Kind::Spawnling { 1.0 } else { 0.0 },
            fire_t: macroquad::rand::gen_range(1.4, 2.6),
            burst: 0,
            burst_t: 0.0,
            hover: pos,
            retarget_t: 0.0,
            flash: 0.0,
            harpooned: None,
            dead: false,
            facing: 1.0,
            look: Vec2::ZERO,
            boss_move: BossMove::Drift,
            move_t: 3.0,
            beam_x: 0.0,
            beam_dir: 1.0,
            on_ground: false,
            life: 7.0,
        }
    }

    pub fn radius(&self) -> f32 {
        info(self.kind).radius * self.emerge.max(0.2)
    }

    pub fn eye(&self) -> Vec2 {
        match self.kind {
            Kind::Gazer => self.pos + vec2(0.0, -1.0),
            Kind::Leviathan => self.pos + vec2(0.0, -5.0),
            Kind::Maw => self.pos + vec2(0.0, 0.0),
            _ => self.pos,
        }
    }

    pub fn draw(&self, g: &Gfx) {
        let inf = info(self.kind);
        let tex = match self.kind {
            Kind::Gazer => &g.gazer,
            Kind::Maw => &g.maw,
            Kind::Seraph => &g.seraph,
            Kind::Leviathan => &g.leviathan,
            Kind::Spawnling => &g.spawnling,
        };
        let frame = ((self.t * inf.fps) as u32) % inf.frames;
        let s = self.emerge.clamp(0.0, 1.0);
        let s = 1.0 - (1.0 - s).powi(3);
        let w = inf.fw * s;
        let h = inf.fh * s;
        let mut col = WHITE;
        if self.flash > 0.0 {
            let f = (self.flash * 12.0).min(1.0);
            col = Color::new(1.0, 1.0 - f * 0.5, 1.0 - f * 0.5, 1.0);
        }
        let mut rot = 0.0;
        if let Some(ht) = self.harpooned {
            rot = (ht * 30.0).sin() * 0.25;
        }
        let dx = self.pos.x - inf.anchor.x * s;
        let dy = self.pos.y - inf.anchor.y * s;
        draw_texture_ex(tex, dx, dy, col, DrawTextureParams {
            source: Some(Rect::new(frame as f32 * inf.fw, 0.0, inf.fw, inf.fh)),
            dest_size: Some(vec2(w, h)),
            flip_x: self.kind == Kind::Spawnling && self.facing < 0.0,
            rotation: rot,
            ..Default::default()
        });
        // tracking pupils
        if s > 0.9 {
            match self.kind {
                Kind::Gazer => {
                    let c = self.eye();
                    let p = c + self.look.clamp_length_max(1.0) * 4.0;
                    draw_ellipse(p.x, p.y, 3.0, 5.0, 0.0, Color::from_rgba(8, 4, 6, 255));
                    draw_rectangle(p.x - 2.0, p.y - 3.0, 1.0, 1.0, WHITE);
                }
                Kind::Leviathan => {
                    let c = self.eye();
                    let p = c + self.look.clamp_length_max(1.0) * 8.0;
                    let dilate = if self.boss_move == BossMove::Beam { 2.0 } else { 6.0 };
                    draw_ellipse(p.x, p.y, dilate, 10.0, 0.0, Color::from_rgba(6, 0, 4, 255));
                    draw_rectangle(p.x - 3.0, p.y - 6.0, 2.0, 2.0, WHITE);
                }
                _ => {}
            }
        }
    }
}
