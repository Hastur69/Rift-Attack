//! Horror behaviour: hovering, stalking, firing patterns, the boss's attack
//! cycle, plus the rift spawner that ramps difficulty with the kill count.

use crate::audio::Audio;
use crate::enemies::{BossMove, Enemy, Kind};
use crate::fx::{rand_dir, PKind, Particle};
use crate::game::{EShot, EShotKind, Game, RiftPortal};
use crate::level::{GROUND_Y, VW, WORLD_W};
use crate::weapons::Power;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

impl Game {
    pub(crate) fn update_spawning(&mut self, dt: f32, audio: &mut Audio) {
        // advance open rifts
        let mut spawn = Vec::new();
        for r in self.rifts.iter_mut() {
            r.t += dt;
            if r.t >= 0.75 && !r.spawned {
                r.spawned = true;
                spawn.push((r.kind, r.pos, r.hp_scale));
            }
        }
        self.rifts.retain(|r| r.t < 2.2);
        for (kind, pos, hp) in spawn {
            let id = self.new_id();
            let mut e = Enemy::new(id, kind, pos, hp);
            if kind == Kind::Leviathan {
                self.boss_pending = false;
                e.fire_t = 3.0;
            }
            match kind {
                Kind::Gazer => audio.play("gazer", 0.45),
                Kind::Maw => audio.play("maw", 0.45),
                Kind::Seraph => audio.play("seraph", 0.45),
                Kind::Leviathan => audio.play("boss", 1.0),
                _ => {}
            }
            self.enemies.push(e);
        }

        if !self.player.alive || self.boss_pending || self.boss_alive() {
            return;
        }
        self.spawn_t -= dt;
        let flyers = self.enemies.iter().filter(|e| !e.dead && e.kind != Kind::Spawnling).count()
            + self.rifts.iter().filter(|r| !r.spawned).count();
        let max_alive = (1 + self.kills as usize / 8).min(5);
        if self.spawn_t <= 0.0 && flyers < max_alive {
            self.spawn_t = (2.8 - self.threat as f32 * 0.22).max(0.9) * gen_range(0.8, 1.2);
            // pick a horror
            let mut table: Vec<(Kind, f32)> = vec![(Kind::Gazer, 10.0)];
            if self.kills >= 8 {
                table.push((Kind::Maw, 2.5 + self.threat as f32));
            }
            if self.kills >= 18 {
                table.push((Kind::Seraph, 2.5 + self.threat as f32));
            }
            let total: f32 = table.iter().map(|t| t.1).sum();
            let mut r = gen_range(0.0, total);
            let mut kind = Kind::Gazer;
            for (k, w) in table {
                if r < w {
                    kind = k;
                    break;
                }
                r -= w;
            }
            // somewhere on screen, not right on top of the marine
            let mut x = 0.0;
            for _ in 0..8 {
                x = gen_range(self.cam_x + 50.0, self.cam_x + VW - 50.0);
                if (x - self.player.pos.x).abs() > 120.0 {
                    break;
                }
            }
            let y = gen_range(45.0, 150.0);
            let hp_scale = 1.0 + self.threat as f32 * 0.08;
            self.rifts.push(RiftPortal { pos: vec2(x.clamp(40.0, WORLD_W - 40.0), y), t: 0.0, kind, hp_scale, spawned: false });
            audio.play("rift", 0.45);
        }
    }

    pub(crate) fn update_enemies(&mut self, dt: f32, audio: &mut Audio) {
        let target = self.player.center();
        let player_alive = self.player.alive;
        let void_sight = self.power(Power::VoidSight);
        let rate = 1.0 + self.threat as f32 * 0.07;
        let threat = self.threat as f32;
        let mut shots: Vec<EShot> = Vec::new();
        let mut new_enemies: Vec<(Kind, Vec2)> = Vec::new();
        let mut kills: Vec<usize> = Vec::new();
        let mut hurt: f32 = 0.0;
        let mut sounds: Vec<(&'static str, f32)> = Vec::new();
        let spawnlings = self.enemies.iter().filter(|e| !e.dead && e.kind == Kind::Spawnling).count();
        let gazers = self.enemies.iter().filter(|e| !e.dead && e.kind == Kind::Gazer).count();
        let cam_x = self.cam_x;

        for (i, e) in self.enemies.iter_mut().enumerate() {
            if e.dead {
                continue;
            }
            e.t += dt;
            e.flash = (e.flash - dt).max(0.0);
            if e.emerge < 1.0 {
                e.emerge = (e.emerge + dt * if e.kind == Kind::Leviathan { 0.5 } else { 1.6 }).min(1.0);
            }
            e.look = (target - e.eye()) / 120.0;

            // dragged down by a harpoon
            if let Some(ht) = e.harpooned {
                let ht = ht + dt;
                e.harpooned = Some(ht);
                e.vel.y += 1100.0 * dt;
                e.vel.x *= 0.95;
                e.pos += e.vel * dt;
                if e.pos.y >= GROUND_Y - e.radius() * 0.4 || ht > 1.4 {
                    kills.push(i);
                }
                continue;
            }

            let mut aim_at = target;
            if void_sight || !player_alive {
                aim_at = target + rand_dir() * 260.0;
            }
            let to_player = aim_at - e.pos;
            let lead = |from: Vec2, speed: f32| -> Vec2 { (aim_at - from).normalize_or_zero() * speed };

            match e.kind {
                Kind::Gazer | Kind::Maw | Kind::Seraph => {
                    let (spd, alt_lo, alt_hi, retarget) = match e.kind {
                        Kind::Gazer => (110.0, 50.0, 150.0, (2.4, 4.4)),
                        Kind::Maw => (70.0, 45.0, 115.0, (3.0, 5.0)),
                        _ => (200.0, 40.0, 170.0, (1.0, 2.0)),
                    };
                    e.retarget_t -= dt;
                    if e.retarget_t <= 0.0 {
                        e.retarget_t = gen_range(retarget.0, retarget.1);
                        let side = if gen_range(0, 2) == 0 { -1.0 } else { 1.0 };
                        let hx = target.x + side * gen_range(90.0, 240.0);
                        e.hover = vec2(hx.clamp(40.0, WORLD_W - 40.0), gen_range(alt_lo, alt_hi));
                        // seraphs sometimes blink straight there
                        if e.kind == Kind::Seraph && e.emerge >= 1.0 && gen_range(0, 3) == 0 {
                            sounds.push(("seraph", 0.3));
                            e.pos = e.hover;
                            e.vel = Vec2::ZERO;
                        }
                    }
                    let desired = ((e.hover - e.pos) * 1.4).clamp_length_max(spd);
                    e.vel = e.vel.lerp(desired, (dt * 2.2).min(1.0));
                    let bob = vec2((e.t * 1.3).sin() * 6.0, (e.t * 2.1).cos() * 9.0);
                    e.pos += (e.vel + bob) * dt;
                    e.facing = if to_player.x < 0.0 { -1.0 } else { 1.0 };
                    if e.emerge < 1.0 || !player_alive {
                        continue;
                    }

                    e.fire_t -= dt * rate;
                    match e.kind {
                        Kind::Gazer => {
                            if e.fire_t <= 0.0 {
                                e.fire_t = gen_range(2.0, 3.1);
                                e.burst = 3;
                                e.burst_t = 0.0;
                            }
                            if e.burst > 0 {
                                e.burst_t -= dt;
                                if e.burst_t <= 0.0 {
                                    e.burst -= 1;
                                    e.burst_t = 0.16;
                                    let v = lead(e.eye(), 230.0 + threat * 8.0).rotate(Vec2::from_angle(gen_range(-0.08, 0.08)));
                                    shots.push(EShot { pos: e.eye() + v.normalize_or_zero() * 12.0, vel: v, kind: EShotKind::Bolt, damage: 7.0, life: 4.0, grav: 0.0, dead: false });
                                    sounds.push(("voidbolt", 0.35));
                                }
                            }
                        }
                        Kind::Maw => {
                            if e.fire_t <= 0.0 {
                                e.fire_t = gen_range(3.2, 4.2);
                                e.burst += 1;
                                let dx = to_player.x;
                                for k in 0..5 {
                                    let spread = (k as f32 - 2.0) * 32.0;
                                    let vx = (dx + spread) * 0.85;
                                    let vy = -150.0 - gen_range(0.0, 60.0);
                                    shots.push(EShot { pos: e.pos + vec2(0.0, 10.0), vel: vec2(vx.clamp(-320.0, 320.0), vy), kind: EShotKind::Glob, damage: 9.0, life: 5.0, grav: 380.0, dead: false });
                                }
                                sounds.push(("maw", 0.35));
                                if e.burst % 2 == 0 && spawnlings + new_enemies.len() < 4 {
                                    new_enemies.push((Kind::Spawnling, e.pos + vec2(0.0, 18.0)));
                                }
                            }
                        }
                        _ => {
                            if e.fire_t <= 0.0 {
                                e.fire_t = gen_range(2.3, 3.0);
                                let base = to_player.y.atan2(to_player.x);
                                for k in 0..7 {
                                    let a = base + (k as f32 - 3.0) * 0.16;
                                    let v = vec2(a.cos(), a.sin()) * (140.0 + threat * 5.0);
                                    shots.push(EShot { pos: e.pos, vel: v, kind: EShotKind::Orb, damage: 7.0, life: 6.0, grav: 0.0, dead: false });
                                }
                                sounds.push(("voidbolt", 0.4));
                            }
                        }
                    }
                }
                Kind::Spawnling => {
                    e.life -= dt;
                    e.vel.y += 700.0 * dt;
                    if e.on_ground {
                        let dir = (target.x - e.pos.x).signum();
                        e.vel.x = dir * 60.0;
                        e.facing = dir;
                    }
                    e.pos += e.vel * dt;
                    if e.pos.y >= GROUND_Y - 6.0 {
                        e.pos.y = GROUND_Y - 6.0;
                        e.vel.y = 0.0;
                        e.on_ground = true;
                    }
                    if player_alive && e.pos.distance(target + vec2(0.0, 10.0)) < 14.0 {
                        hurt += 12.0;
                        kills.push(i);
                    } else if e.life <= 0.0 {
                        kills.push(i);
                    }
                }
                Kind::Leviathan => {
                    // drift high above, following the marine
                    let hx = (target.x).clamp(110.0, WORLD_W - 110.0);
                    let hy = 88.0 + (e.t * 0.8).sin() * 10.0;
                    let desired = vec2((hx - e.pos.x) * 0.5, (hy - e.pos.y) * 1.2).clamp_length_max(55.0);
                    e.vel = e.vel.lerp(desired, (dt * 1.5).min(1.0));
                    e.pos += e.vel * dt;
                    if e.emerge < 1.0 || !player_alive {
                        continue;
                    }
                    let enraged = e.hp < e.max_hp * 0.5;
                    let speed = if enraged { 1.35 } else { 1.0 };
                    e.move_t -= dt * speed;
                    match e.boss_move {
                        BossMove::Drift => {
                            e.fire_t -= dt * speed;
                            if e.fire_t <= 0.0 {
                                e.fire_t = 0.9;
                                for k in -2..=2 {
                                    let v = lead(e.eye(), 250.0).rotate(Vec2::from_angle(k as f32 * 0.18));
                                    shots.push(EShot { pos: e.eye() + vec2(0.0, 20.0), vel: v, kind: EShotKind::Bolt, damage: 8.0, life: 4.0, grav: 0.0, dead: false });
                                }
                                sounds.push(("voidbolt", 0.5));
                            }
                        }
                        BossMove::Beam => {
                            // 1.2s of aiming, then the eye-beam sweeps the deck
                            let elapsed = 3.4 - e.move_t;
                            if elapsed > 1.2 {
                                e.beam_x += e.beam_dir * 230.0 * dt * speed;
                                let a = e.eye();
                                let b = vec2(e.beam_x, GROUND_Y);
                                let ab = b - a;
                                let t = ((target - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
                                if (a + ab * t).distance(target) < 12.0 {
                                    hurt += 14.0;
                                }
                                if gen_range(0, 2) == 0 {
                                    sounds.push(("flame", 0.25));
                                }
                            }
                        }
                        BossMove::Rain => {
                            e.fire_t -= dt * speed;
                            if e.fire_t <= 0.0 {
                                e.fire_t = 0.11;
                                let x = cam_x + gen_range(0.0, VW);
                                shots.push(EShot { pos: vec2(x, -10.0), vel: vec2(gen_range(-20.0, 20.0), gen_range(170.0, 240.0)), kind: EShotKind::Rain, damage: 7.0, life: 4.0, grav: 0.0, dead: false });
                            }
                        }
                        BossMove::Summon => {}
                    }
                    if e.move_t <= 0.0 {
                        e.boss_move = match e.boss_move {
                            BossMove::Drift => {
                                e.move_t = 3.4;
                                e.beam_dir = if target.x > e.pos.x { 1.0 } else { -1.0 };
                                e.beam_x = target.x - e.beam_dir * 230.0;
                                sounds.push(("boss", 0.6));
                                BossMove::Beam
                            }
                            BossMove::Beam => {
                                e.move_t = 2.6;
                                e.fire_t = 0.0;
                                BossMove::Rain
                            }
                            BossMove::Rain => {
                                e.move_t = 1.2;
                                if gazers < 3 {
                                    new_enemies.push((Kind::Gazer, e.pos + vec2(-110.0, 30.0)));
                                    new_enemies.push((Kind::Gazer, e.pos + vec2(110.0, 30.0)));
                                }
                                sounds.push(("rift", 0.6));
                                BossMove::Summon
                            }
                            BossMove::Summon => {
                                e.move_t = 3.5;
                                e.fire_t = 0.5;
                                BossMove::Drift
                            }
                        };
                    }
                }
            }
        }

        for (s, v) in sounds {
            audio.play(s, v);
        }
        for i in kills {
            if !self.enemies[i].dead {
                if self.enemies[i].harpooned.is_some() {
                    let p = self.enemies[i].pos;
                    self.explode(p, 34.0, 60.0, audio);
                }
                let hp = self.enemies[i].hp;
                self.damage_enemy(i, hp + 1.0, Vec2::ZERO, audio);
            }
        }
        self.harpoon_lines.retain(|(_, id)| self.enemies.iter().any(|e| e.id == *id && !e.dead));
        for (k, pos) in new_enemies {
            if k == Kind::Spawnling {
                let id = self.new_id();
                let mut e = Enemy::new(id, k, pos, 1.0);
                e.vel = vec2(gen_range(-40.0, 40.0), 0.0);
                self.enemies.push(e);
            } else {
                self.rifts.push(RiftPortal { pos, t: 0.0, kind: k, hp_scale: 1.0, spawned: false });
            }
        }
        self.eshots.extend(shots);
        if hurt > 0.0 {
            self.hurt_player(hurt, audio);
        }
        self.enemies.retain(|e| !e.dead);
    }

    pub(crate) fn update_eshots(&mut self, dt: f32, audio: &mut Audio) {
        let pr = self.player.rect();
        let mut hurt = 0.0;
        let alive = self.player.alive;
        for s in self.eshots.iter_mut() {
            s.life -= dt;
            s.vel.y += s.grav * dt;
            s.pos += s.vel * dt;
            if s.life <= 0.0 || s.pos.y > GROUND_Y + 4.0 || self.level.solid_at(s.pos) {
                s.dead = true;
                let c = match s.kind {
                    EShotKind::Glob => Color::from_rgba(255, 150, 40, 255),
                    _ => Color::from_rgba(200, 120, 255, 255),
                };
                self.fx.spark(s.pos, vec2(0.0, -1.0), 4, c, 120.0);
                if s.kind == EShotKind::Glob {
                    self.fx.ichor(s.pos, vec2(0.0, -1.0), 3, c, 60.0);
                }
                continue;
            }
            let r = match s.kind {
                EShotKind::Glob => 4.0,
                EShotKind::Orb => 3.5,
                _ => 3.0,
            };
            let hit = Rect::new(pr.x - r, pr.y - r, pr.w + r * 2.0, pr.h + r * 2.0);
            if alive && hit.contains(s.pos) {
                s.dead = true;
                hurt += s.damage;
            }
            if s.kind == EShotKind::Bolt && gen_range(0, 3) == 0 {
                self.fx.add(Particle { pos: s.pos, vel: -s.vel * 0.1, life: 0.2, max: 0.2, size: 1.0, color: Color::from_rgba(190, 255, 80, 255), kind: PKind::Spark, grav: 0.0, drag: 0.0 });
            }
        }
        self.eshots.retain(|s| !s.dead);
        if hurt > 0.0 {
            self.hurt_player(hurt, audio);
        }
    }
}
