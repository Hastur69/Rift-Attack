//! World rendering for a run. Called with the low-res world camera active.

use crate::assets::{Gfx, Materials};
use crate::enemies::{BossMove, Kind};
use crate::fx::glow_at;
use crate::game::{EShotKind, Game};
use crate::level::{GROUND_Y, VH, VW};
use crate::weapons::*;
use macroquad::prelude::*;

impl Game {
    /// Screen-space backdrop (sky, parallax silhouettes).
    pub fn draw_backdrop(&self, g: &Gfx) {
        self.level.draw_backdrop(g, self.cam_x, self.time, self.lightning);
    }

    /// Everything that lives in world space.
    pub fn draw_world(&self, g: &Gfx, m: &Materials) {
        let t = self.time;
        self.level.draw_world(g);
        self.fx.draw_decals();

        // rifts (additive)
        gl_use_material(&m.additive);
        for r in &self.rifts {
            let big = r.kind == Kind::Leviathan;
            let open = (r.t / 0.6).clamp(0.0, 1.0);
            let close = if r.t > 1.2 { (1.0 - (r.t - 1.2) / 1.0).clamp(0.0, 1.0) } else { 1.0 };
            let s = (if big { 230.0 } else { 70.0 }) * (1.0 - (1.0 - open).powi(3)) * close;
            let frame = ((t * 14.0) as u32 % 8) as f32;
            glow_at(&g.glow, r.pos, s * 2.2, Color::new(0.6, 0.2, 1.0, 0.5 * close));
            draw_texture_ex(&g.rift, r.pos.x - s * 0.5, r.pos.y - s * 0.5, WHITE, DrawTextureParams {
                source: Some(Rect::new(frame * 64.0, 0.0, 64.0, 64.0)),
                dest_size: Some(vec2(s, s)),
                rotation: t * 2.0,
                ..Default::default()
            });
        }
        // fire pillars
        for p in &self.pillars {
            let a = (p.life / 0.4).min(1.0) * (((3.0 - p.life) / 0.2).min(1.0));
            let h = p.y - 40.0;
            let w = 26.0 + (t * 30.0).sin() * 3.0;
            draw_texture_ex(&g.glow, p.x - w, 40.0, Color::new(1.0, 0.4, 0.1, 0.8 * a), DrawTextureParams { dest_size: Some(vec2(w * 2.0, h + 20.0)), ..Default::default() });
            draw_texture_ex(&g.glow, p.x - w * 0.35, 40.0, Color::new(1.0, 0.9, 0.5, 0.9 * a), DrawTextureParams { dest_size: Some(vec2(w * 0.7, h + 10.0)), ..Default::default() });
        }
        gl_use_default_material();

        // crates
        for c in &self.crates {
            if c.landed && c.life < 3.0 && ((c.life * 8.0) as i32) % 2 == 0 {
                continue;
            }
            if !c.landed {
                draw_texture_ex(&g.crates, c.pos.x - 12.0, c.pos.y - 34.0, WHITE, DrawTextureParams {
                    source: Some(Rect::new(128.0, 0.0, 24.0, 18.0)),
                    ..Default::default()
                });
            }
            let idx = c.sprite_index() as f32;
            draw_texture_ex(&g.crates, c.pos.x - 8.0, c.pos.y - 18.0, WHITE, DrawTextureParams {
                source: Some(Rect::new(idx * 16.0, 0.0, 16.0, 18.0)),
                ..Default::default()
            });
        }

        // horrors
        for e in &self.enemies {
            e.draw(g);
        }
        // harpoon tethers
        for (_, id) in &self.harpoon_lines {
            if let Some(e) = self.enemies.iter().find(|e| e.id == *id) {
                let a = self.player.shoulder();
                draw_line(a.x, a.y, e.pos.x, e.pos.y, 1.0, Color::from_rgba(200, 180, 140, 255));
            }
        }

        self.draw_player(g);

        // projectiles (solid parts)
        for b in &self.bullets {
            let d = b.vel.normalize_or_zero();
            match b.shot {
                Shot::Rocket | Shot::Rpg | Shot::Seeker => {
                    let tail = b.pos - d * 7.0;
                    draw_line(tail.x, tail.y, b.pos.x, b.pos.y, 3.0, Color::from_rgba(80, 84, 70, 255));
                    draw_line(b.pos.x - d.x * 2.0, b.pos.y - d.y * 2.0, b.pos.x, b.pos.y, 3.0, Color::from_rgba(200, 60, 40, 255));
                }
                Shot::Grenade => draw_circle(b.pos.x, b.pos.y, 2.5, Color::from_rgba(90, 70, 120, 255)),
                Shot::Mine => {
                    draw_rectangle(b.pos.x - 4.0, b.pos.y - 2.0, 8.0, 4.0, Color::from_rgba(90, 40, 30, 255));
                    if ((b.age * 10.0) as i32) % 2 == 0 {
                        draw_rectangle(b.pos.x - 1.0, b.pos.y - 3.0, 2.0, 1.0, RED);
                    }
                }
                Shot::Singularity => {
                    let r = 7.0 + (t * 20.0).sin();
                    draw_circle(b.pos.x, b.pos.y, r, BLACK);
                    draw_circle_lines(b.pos.x, b.pos.y, r + 2.0, 1.0, Color::from_rgba(220, 160, 255, 255));
                }
                Shot::Harpoon => {
                    let a = self.player.shoulder();
                    draw_line(a.x, a.y, b.pos.x, b.pos.y, 1.0, Color::from_rgba(200, 180, 140, 255));
                    let tail = b.pos - d * 9.0;
                    draw_line(tail.x, tail.y, b.pos.x, b.pos.y, 2.0, Color::from_rgba(220, 220, 220, 255));
                }
                _ => {}
            }
        }
        self.fx.draw_normal();

        // ---- additive light pass ----------------------------------------
        gl_use_material(&m.additive);
        for b in &self.bullets {
            let d = b.vel.normalize_or_zero();
            match b.shot {
                Shot::Bullet | Shot::Pellet | Shot::Smart => {
                    let c = if b.shot == Shot::Smart { Color::new(1.0, 0.35, 0.35, 1.0) } else { Color::new(1.0, 0.85, 0.45, 1.0) };
                    let tail = b.pos - d * 9.0;
                    draw_line(tail.x, tail.y, b.pos.x, b.pos.y, 1.5, c);
                    glow_at(&g.glow, b.pos, 10.0, Color::new(c.r, c.g, c.b, 0.6));
                }
                Shot::Rocket | Shot::Rpg | Shot::Seeker => {
                    let back = b.pos - d * 8.0;
                    let burning = b.shot != Shot::Rpg || b.age > 0.28;
                    if burning {
                        glow_at(&g.glow, back, 22.0 + (t * 50.0).sin() * 4.0, Color::new(1.0, 0.6, 0.2, 0.9));
                    }
                }
                Shot::Grenade => glow_at(&g.glow, b.pos, 10.0, Color::new(0.8, 0.4, 1.0, 0.6)),
                Shot::Flame => {
                    let k = (b.age / 0.5).clamp(0.0, 1.0);
                    let s = 8.0 + k * 30.0;
                    let c = Color::new(1.0, 0.9 - k * 0.6, 0.4 - k * 0.35, 0.75 * (1.0 - k * 0.7));
                    glow_at(&g.glow, b.pos, s, c);
                }
                Shot::Singularity => glow_at(&g.glow, b.pos, 50.0 + (t * 8.0).sin() * 8.0, Color::new(0.6, 0.2, 1.0, 0.8)),
                Shot::Mine => glow_at(&g.glow, b.pos, 14.0, Color::new(1.0, 0.3, 0.1, 0.5)),
                _ => {}
            }
        }
        for s in &self.eshots {
            let (c, size) = match s.kind {
                EShotKind::Bolt => (Color::new(0.75, 1.0, 0.25, 1.0), 16.0),
                EShotKind::Glob => (Color::new(1.0, 0.55, 0.1, 1.0), 18.0),
                EShotKind::Orb => (Color::new(0.4, 0.85, 1.0, 1.0), 16.0),
                EShotKind::Rain => (Color::new(1.0, 0.2, 0.5, 1.0), 14.0),
            };
            glow_at(&g.glow, s.pos, size, c);
            draw_circle(s.pos.x, s.pos.y, 2.0, Color::new(1.0, 1.0, 1.0, 0.9));
        }
        // muzzle flash
        if self.player.muzzle_flash > 0.0 && self.player.alive {
            let w = &WEAPONS[self.player.weapon];
            let m = self.player.shoulder() + self.player.aim_dir * (w.muzzle - 4.0);
            glow_at(&g.glow, m, 30.0, Color::new(1.0, 0.85, 0.5, 0.9));
        }
        // the Hollow Crown's eye-beam
        for e in &self.enemies {
            if e.kind == Kind::Leviathan && e.boss_move == BossMove::Beam && e.emerge >= 1.0 {
                let elapsed = 3.4 - e.move_t;
                let a = e.eye();
                let b = vec2(e.beam_x, GROUND_Y);
                if elapsed < 1.2 {
                    if ((t * 16.0) as i32) % 2 == 0 {
                        draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(1.0, 0.1, 0.3, 0.8));
                    }
                } else {
                    let w = 10.0 + (t * 40.0).sin() * 2.0;
                    draw_line(a.x, a.y, b.x, b.y, w * 2.5, Color::new(1.0, 0.1, 0.4, 0.35));
                    draw_line(a.x, a.y, b.x, b.y, w, Color::new(1.0, 0.3, 0.6, 0.8));
                    draw_line(a.x, a.y, b.x, b.y, 3.0, WHITE);
                    glow_at(&g.glow, b, 60.0, Color::new(1.0, 0.3, 0.5, 0.9));
                }
                glow_at(&g.glow, a, 50.0, Color::new(1.0, 0.2, 0.4, 0.8));
            }
        }
        // elder ward sigil
        if self.power(Power::ElderWard) && self.player.alive {
            let c = self.player.center();
            for k in 0..5 {
                let a0 = t * 2.0 + k as f32 * std::f32::consts::TAU / 5.0;
                let a1 = a0 + std::f32::consts::TAU * 2.0 / 5.0;
                let r = 22.0;
                draw_line(c.x + a0.cos() * r, c.y + a0.sin() * r, c.x + a1.cos() * r, c.y + a1.sin() * r, 1.0, Color::new(1.0, 0.85, 0.3, 0.7));
            }
            glow_at(&g.glow, c, 64.0, Color::new(1.0, 0.8, 0.3, 0.35));
        }
        // marine's helmet lamp
        if self.player.alive {
            let lamp = self.player.pos + vec2(self.player.facing * 4.0, -37.0);
            glow_at(&g.glow, lamp, 12.0, Color::new(1.0, 0.95, 0.6, 0.6));
        }
        self.fx.draw_additive(&g.glow);
        gl_use_default_material();
    }

    fn draw_player(&self, g: &Gfx) {
        let p = &self.player;
        let frame = if !p.alive {
            13 + ((p.death_t * 6.0) as usize).min(3)
        } else if !p.on_ground {
            if p.vel.y < 0.0 { 10 } else { 11 }
        } else if p.crouch_t > 0.0 {
            12
        } else if p.vel.x.abs() > 10.0 {
            let dir = if p.vel.x.signum() == p.facing { 1 } else { -1 };
            2 + (((p.anim_t * 8.0) as i32) * dir).rem_euclid(8) as usize
        } else {
            ((self.time * 1.6) as usize) % 2
        };
        let void = self.power(Power::VoidSight);
        let mut col = WHITE;
        if p.hurt_flash > 0.0 {
            col = Color::new(1.0, 1.0 - p.hurt_flash * 0.7, 1.0 - p.hurt_flash * 0.7, 1.0);
        }
        if p.iframes > 0.0 && p.alive && ((p.iframes * 20.0) as i32) % 2 == 0 {
            col.a = 0.4;
        }
        if void {
            col.a = 0.25 + 0.1 * (self.time * 9.0).sin();
        }
        let flip = p.facing < 0.0;
        draw_texture_ex(&g.marine, (p.pos.x - 16.0).round(), (p.pos.y - 39.0).round(), col, DrawTextureParams {
            source: Some(Rect::new(frame as f32 * 32.0, 0.0, 32.0, 40.0)),
            flip_x: flip,
            ..Default::default()
        });
        if !p.alive {
            return;
        }
        // weapon arm
        let sh = p.shoulder() - p.aim_dir * p.recoil;
        let ang = p.aim_dir.y.atan2(p.aim_dir.x);
        draw_texture_ex(&g.weapons, sh.x - 4.0, sh.y - 8.0, col, DrawTextureParams {
            source: Some(Rect::new(0.0, p.weapon as f32 * 16.0, 48.0, 16.0)),
            rotation: ang,
            pivot: Some(sh),
            flip_y: flip,
            ..Default::default()
        });
    }

    /// Off-screen threat arrows for horrors the camera can't see.
    pub fn draw_edge_markers(&self) {
        for e in &self.enemies {
            if e.kind == Kind::Spawnling {
                continue;
            }
            let sx = e.pos.x - self.cam_x;
            if sx < -10.0 || sx > VW + 10.0 {
                let x = sx.clamp(6.0, VW - 6.0);
                let y = e.pos.y.clamp(20.0, VH - 40.0);
                let dir = if sx < 0.0 { -1.0 } else { 1.0 };
                let blink = ((self.time * 6.0) as i32) % 2 == 0;
                let c = if blink { Color::from_rgba(255, 60, 90, 255) } else { Color::from_rgba(150, 30, 60, 255) };
                draw_triangle(vec2(x + dir * 5.0, y), vec2(x - dir * 3.0, y - 5.0), vec2(x - dir * 3.0, y + 5.0), c);
            }
        }
    }
}

