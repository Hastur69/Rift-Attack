//! Core simulation: the marine, the arsenal in flight, the horrors, crates,
//! power-ups, spawning, scoring - everything that happens during a run.

use crate::audio::{Audio, Track};
use crate::enemies::{info, Enemy, Kind};
use crate::fx::{rand_dir, Fx, PKind, Particle};
use crate::level::{Level, GROUND_Y, VH, VW, WORLD_W};
use crate::weapons::*;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

// ---------------------------------------------------------------------------
// Input (filled from keyboard/mouse, or from the autoplay bot)
// ---------------------------------------------------------------------------
#[derive(Default, Clone)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub down: bool,
    pub fire: bool,
    pub aim: Vec2,
    pub next: bool,
    pub prev: bool,
    pub select: Option<usize>,
    pub distort: bool,
    pub suicide: bool,
}

// ---------------------------------------------------------------------------
// The Void Marine
// ---------------------------------------------------------------------------
pub const PW: f32 = 10.0;
pub const PH: f32 = 28.0;
pub const MAX_HP: f32 = 100.0;

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub on_ground: bool,
    pub on_platform: bool,
    pub jumps_left: u32,
    pub facing: f32,
    pub hp: f32,
    pub iframes: f32,
    pub hurt_flash: f32,
    pub crouch_t: f32,
    pub hyper: bool,
    pub drop_t: f32,
    pub anim_t: f32,
    pub alive: bool,
    pub death_t: f32,
    pub weapon: usize,
    pub owned: [bool; NUM_WEAPONS],
    pub ammo: [u32; NUM_WEAPONS],
    pub fire_cd: f32,
    pub distort: f32,
    pub recoil: f32,
    pub aim_dir: Vec2,
    pub pre_void_weapon: usize,
    pub muzzle_flash: f32,
}

impl Player {
    pub fn new() -> Player {
        let mut owned = [false; NUM_WEAPONS];
        owned[PULSE] = true;
        Player {
            pos: vec2(WORLD_W * 0.5, GROUND_Y),
            vel: Vec2::ZERO,
            on_ground: true,
            on_platform: false,
            jumps_left: 1,
            facing: 1.0,
            hp: MAX_HP,
            iframes: 1.0,
            hurt_flash: 0.0,
            crouch_t: 0.0,
            hyper: false,
            drop_t: 0.0,
            anim_t: 0.0,
            alive: true,
            death_t: 0.0,
            weapon: PULSE,
            owned,
            ammo: [0; NUM_WEAPONS],
            fire_cd: 0.0,
            distort: 1.0,
            recoil: 0.0,
            aim_dir: vec2(1.0, 0.0),
            pre_void_weapon: PULSE,
            muzzle_flash: 0.0,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x - PW * 0.5, self.pos.y - PH, PW, PH)
    }

    pub fn center(&self) -> Vec2 {
        self.pos + vec2(0.0, -PH * 0.5)
    }

    pub fn shoulder(&self) -> Vec2 {
        let crouch = if self.crouch_t > 0.0 && self.on_ground { 5.0 } else { 0.0 };
        self.pos + vec2(0.0, -21.0 + crouch)
    }
}

// ---------------------------------------------------------------------------
// Projectiles
// ---------------------------------------------------------------------------
pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub shot: Shot,
    pub damage: f32,
    pub splash: f32,
    pub radius: f32,
    pub life: f32,
    pub age: f32,
    pub speed: f32,
    pub target: Option<u32>,
    pub hit: Vec<u32>,
    pub dead: bool,
    pub landed: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EShotKind {
    Bolt,
    Glob,
    Orb,
    Rain,
}

pub struct EShot {
    pub pos: Vec2,
    pub vel: Vec2,
    pub kind: EShotKind,
    pub damage: f32,
    pub life: f32,
    pub grav: f32,
    pub dead: bool,
}

pub struct Pillar {
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub dps: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Content {
    Weapon(usize),
    Power(Power),
    Medkit,
    Mystery,
}

pub struct Crate {
    pub pos: Vec2,
    pub vel: Vec2,
    pub content: Content,
    pub landed: bool,
    pub life: f32,
    pub t: f32,
}

impl Crate {
    pub fn sprite_index(&self) -> usize {
        match self.content {
            Content::Weapon(w) => WEAPONS[w].crate_kind as usize,
            Content::Power(_) => 6,
            Content::Medkit => 7,
            Content::Mystery => ((self.t * 5.0) as usize) % 7,
        }
    }
}

pub struct RiftPortal {
    pub pos: Vec2,
    pub t: f32,
    pub kind: Kind,
    pub hp_scale: f32,
    pub spawned: bool,
}

pub struct Banner {
    pub title: String,
    pub sub: String,
    pub t: f32,
    pub color: Color,
}

pub struct Stats {
    pub shots: u32,
    pub hits: u32,
    pub aerial_kills: u32,
    pub damage: f32,
}

// ---------------------------------------------------------------------------
pub struct Game {
    pub player: Player,
    pub bullets: Vec<Bullet>,
    pub eshots: Vec<EShot>,
    pub enemies: Vec<Enemy>,
    pub crates: Vec<Crate>,
    pub pillars: Vec<Pillar>,
    pub rifts: Vec<RiftPortal>,
    pub fx: Fx,
    pub level: Level,
    pub kills: u32,
    pub score: u64,
    pub time: f32,
    pub threat: u32,
    pub next_id: u32,
    pub spawn_t: f32,
    pub next_boss_at: u32,
    pub bosses_killed: u32,
    pub boss_pending: bool,
    pub powers: [f32; 5],
    pub banner: Option<Banner>,
    pub lightning: f32,
    pub lightning_t: f32,
    pub tracker_t: f32,
    pub tracker_sweep: f32,
    pub kills_since_crate: u32,
    pub next_medkit: u32,
    pub cam_x: f32,
    pub distorting: bool,
    pub stats: Stats,
    pub over_t: f32,
    pub harpoon_lines: Vec<(Vec2, u32)>,
}

impl Game {
    pub fn new() -> Game {
        let mut g = Game {
            player: Player::new(),
            bullets: Vec::new(),
            eshots: Vec::new(),
            enemies: Vec::new(),
            crates: Vec::new(),
            pillars: Vec::new(),
            rifts: Vec::new(),
            fx: Fx::new(),
            level: Level::new(),
            kills: 0,
            score: 0,
            time: 0.0,
            threat: 0,
            next_id: 1,
            spawn_t: 2.0,
            next_boss_at: 40,
            bosses_killed: 0,
            boss_pending: false,
            powers: [0.0; 5],
            banner: None,
            lightning: 0.0,
            lightning_t: 5.0,
            tracker_t: 0.0,
            tracker_sweep: 0.0,
            kills_since_crate: 0,
            next_medkit: 15,
            cam_x: WORLD_W * 0.5 - VW * 0.5,
            distorting: false,
            stats: Stats { shots: 0, hits: 0, aerial_kills: 0, damage: 0.0 },
            over_t: 0.0,
            harpoon_lines: Vec::new(),
        };
        g.banner = Some(Banner {
            title: "LZ GRAVEN".into(),
            sub: "HOLD THE LINE. NOBODY IS COMING.".into(),
            t: 3.5,
            color: Color::from_rgba(180, 255, 160, 255),
        });
        // debug hooks used for automated testing
        if let Ok(k) = std::env::var("RIFT_KILLS") {
            if let Ok(k) = k.parse::<u32>() {
                g.kills = k;
                g.threat = k / 10;
                while g.next_medkit <= k {
                    g.next_medkit *= 2;
                }
            }
        }
        if std::env::var("RIFT_GIVE_ALL").is_ok() {
            for w in 1..NULL_LANCE {
                g.player.owned[w] = true;
                g.player.ammo[w] = WEAPONS[w].clip * 4;
            }
        }
        g
    }

    pub fn power(&self, p: Power) -> bool {
        self.powers[p as usize] > 0.0
    }

    pub(crate) fn new_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    pub fn banner(&mut self, title: &str, sub: &str, color: Color) {
        self.banner = Some(Banner { title: title.into(), sub: sub.into(), t: 3.0, color });
    }

    // =======================================================================
    // UPDATE
    // =======================================================================
    pub fn update(&mut self, dt: f32, input: &Input, audio: &mut Audio) {
        // ---- time scales -------------------------------------------------
        self.distorting = input.distort && self.player.distort > 0.02 && self.player.alive;
        if self.distorting {
            self.player.distort = (self.player.distort - dt * 0.32).max(0.0);
        } else {
            self.player.distort = (self.player.distort + dt * 0.07).min(1.0);
        }
        let gd = if self.distorting { 0.42 } else { 1.0 };
        let pdt = dt * gd;
        let edt = pdt * if self.power(Power::TimeRift) { 0.33 } else { 1.0 };

        self.time += dt;
        if self.player.alive {
            for i in 0..5 {
                if self.powers[i] > 0.0 {
                    self.powers[i] -= dt;
                    if self.powers[i] <= 0.0 {
                        self.powers[i] = 0.0;
                        self.power_ended(i, audio);
                    }
                }
            }
        }
        if let Some(b) = &mut self.banner {
            b.t -= dt;
            if b.t <= 0.0 {
                self.banner = None;
            }
        }

        self.update_player(pdt, dt, input, audio);
        self.update_bullets(pdt, audio);
        self.update_enemies(edt, audio);
        self.update_eshots(edt, audio);
        self.update_pillars(pdt, audio);
        self.update_crates(pdt, audio);
        self.update_spawning(edt, audio);
        self.fx.update(pdt);

        // ambient lightning in the tear
        self.lightning = (self.lightning - dt * 3.0).max(0.0);
        self.lightning_t -= dt;
        if self.lightning_t <= 0.0 {
            self.lightning = 1.0;
            self.lightning_t = gen_range(5.0, 13.0);
        }
        // motion tracker
        self.tracker_sweep = (self.tracker_sweep + dt * 0.9) % 1.0;
        self.tracker_t -= dt;
        if self.tracker_t <= 0.0 {
            self.tracker_t = 1.1;
            if self.player.alive && self.enemies.iter().any(|e| !e.dead) {
                audio.play("tracker", 0.22);
            }
        }

        // camera follows the marine, leaning towards the aim
        let target = self.player.pos.x - VW * 0.5 + self.player.aim_dir.x * 50.0;
        self.cam_x += (target - self.cam_x) * (1.0 - (-dt * 5.0).exp());
        self.cam_x = self.cam_x.clamp(0.0, WORLD_W - VW);

        if !self.player.alive {
            self.over_t += dt;
        }
    }

    fn power_ended(&mut self, i: usize, audio: &mut Audio) {
        if i == Power::VoidSight as usize {
            let p = &mut self.player;
            p.owned[NULL_LANCE] = false;
            if p.weapon == NULL_LANCE {
                p.weapon = if p.owned[p.pre_void_weapon] { p.pre_void_weapon } else { PULSE };
            }
        }
        if i == Power::TimeRift as usize {
            audio.play("timerift", 0.5);
        }
    }

    // -----------------------------------------------------------------------
    fn update_player(&mut self, dt: f32, real_dt: f32, input: &Input, audio: &mut Audio) {
        let jets = self.power(Power::JumpJets);
        let p = &mut self.player;
        p.iframes = (p.iframes - dt).max(0.0);
        p.hurt_flash = (p.hurt_flash - real_dt * 4.0).max(0.0);
        p.recoil = (p.recoil - dt * 30.0).max(0.0);
        p.muzzle_flash = (p.muzzle_flash - dt).max(0.0);
        p.drop_t = (p.drop_t - dt).max(0.0);

        if !p.alive {
            p.death_t += real_dt;
            p.vel.y += 900.0 * dt;
            p.vel.x *= 0.9;
            p.pos += p.vel * dt;
            if p.pos.y > GROUND_Y {
                p.pos.y = GROUND_Y;
                p.vel = Vec2::ZERO;
            }
            return;
        }

        if input.suicide {
            p.hp = 0.0;
            self.kill_player(audio);
            return;
        }

        // aim
        let sh = p.shoulder();
        let d = input.aim - sh;
        if d.length() > 2.0 {
            p.aim_dir = d.normalize();
        }
        p.facing = if p.aim_dir.x < 0.0 { -1.0 } else { 1.0 };

        // horizontal movement
        let ax = (input.right as i32 - input.left as i32) as f32;
        let crouching = p.on_ground && input.down;
        let max_speed = if crouching { 0.0 } else { 150.0 };
        let accel = if p.on_ground { 1500.0 } else { 950.0 };
        let target = ax * max_speed;
        if (target - p.vel.x).abs() < accel * dt {
            p.vel.x = target;
        } else {
            p.vel.x += (target - p.vel.x).signum() * accel * dt;
        }
        if crouching {
            p.crouch_t += dt;
            if p.crouch_t > 0.35 && ((p.crouch_t * 10.0) as i32) % 2 == 0 {
                let c = p.pos + vec2(gen_range(-6.0, 6.0), -gen_range(0.0, 4.0));
                self.fx.add(Particle { pos: c, vel: vec2(0.0, -40.0), life: 0.3, max: 0.3, size: 1.0, color: Color::from_rgba(150, 255, 200, 255), kind: PKind::Spark, grav: 0.0, drag: 0.0 });
            }
        } else if !input.down {
            p.crouch_t = 0.0;
        }

        // jumping
        if input.jump_pressed {
            if p.on_ground {
                if input.down && p.on_platform && p.crouch_t < 0.35 {
                    p.drop_t = 0.25;
                    p.on_ground = false;
                    p.pos.y += 1.0;
                } else if p.crouch_t >= 0.35 {
                    p.vel.y = -660.0;
                    p.hyper = true;
                    p.on_ground = false;
                    p.jumps_left = 1;
                    audio.play("hyperjump", 0.7);
                    for _ in 0..16 {
                        let d = vec2(gen_range(-1.0, 1.0), gen_range(0.0, 0.5));
                        self.fx.smoke(p.pos, d * 60.0, gen_range(3.0, 6.0), 0.6, Color::new(0.4, 0.9, 0.7, 0.4));
                    }
                    self.fx.rings.push(crate::fx::Ring { pos: p.pos, r: 2.0, max_r: 30.0, life: 0.3, max: 0.3, color: Color::from_rgba(120, 255, 190, 255) });
                } else {
                    p.vel.y = -380.0;
                    p.on_ground = false;
                    p.jumps_left = 1;
                    audio.play("jump", 0.5);
                }
                p.crouch_t = 0.0;
            } else if p.jumps_left > 0 && !jets {
                p.vel.y = -340.0;
                p.jumps_left -= 1;
                audio.play("jump", 0.5);
                for _ in 0..6 {
                    self.fx.smoke(p.pos, vec2(gen_range(-40.0, 40.0), gen_range(10.0, 40.0)), 3.0, 0.4, Color::new(0.6, 0.6, 0.7, 0.4));
                }
            }
        }
        if jets && input.jump_held && !p.on_ground {
            p.vel.y = (p.vel.y - 2100.0 * dt).max(-210.0);
            for _ in 0..2 {
                self.fx.add(Particle {
                    pos: p.pos + vec2(-p.facing * 5.0, -10.0),
                    vel: vec2(gen_range(-20.0, 20.0), gen_range(120.0, 200.0)),
                    life: 0.25,
                    max: 0.25,
                    size: 2.0,
                    color: Color::from_rgba(120, 200, 255, 255),
                    kind: PKind::Spark,
                    grav: 0.0,
                    drag: 1.0,
                });
            }
        }

        // gravity (shorter hops when jump released early)
        let mut grav = 1050.0;
        if p.vel.y < 0.0 && !input.jump_held && !p.hyper {
            grav *= 1.7;
        }
        p.vel.y = (p.vel.y + grav * dt).min(620.0);

        // integrate X
        let was_ground = p.on_ground;
        let prev_bottom = p.pos.y;
        p.pos.x += p.vel.x * dt;
        p.pos.x = p.pos.x.clamp(8.0, WORLD_W - 8.0);
        for s in &self.level.solids {
            let r = Rect::new(p.pos.x - PW * 0.5, p.pos.y - PH, PW, PH);
            if hits(&r, s) {
                if p.vel.x > 0.0 || (p.pos.x < s.x + s.w * 0.5) {
                    p.pos.x = s.x - PW * 0.5 - 0.01;
                } else {
                    p.pos.x = s.x + s.w + PW * 0.5 + 0.01;
                }
                p.vel.x = 0.0;
            }
        }
        // integrate Y
        p.pos.y += p.vel.y * dt;
        p.on_ground = false;
        p.on_platform = false;
        for s in &self.level.solids {
            let r = Rect::new(p.pos.x - PW * 0.5, p.pos.y - PH, PW, PH);
            if hits(&r, s) {
                if p.vel.y >= 0.0 {
                    p.pos.y = s.y;
                    p.on_ground = true;
                } else {
                    p.pos.y = s.y + s.h + PH;
                }
                p.vel.y = 0.0;
            }
        }
        if p.vel.y >= 0.0 && p.drop_t <= 0.0 {
            for pl in &self.level.platforms {
                if p.pos.x + PW * 0.4 > pl.x && p.pos.x - PW * 0.4 < pl.x + pl.w && prev_bottom <= pl.y + 0.5 && p.pos.y >= pl.y {
                    p.pos.y = pl.y;
                    p.vel.y = 0.0;
                    p.on_ground = true;
                    p.on_platform = true;
                }
            }
        }
        if p.on_ground {
            p.jumps_left = 1;
            if !was_ground {
                p.hyper = false;
                audio.play("land", 0.4);
                for _ in 0..4 {
                    self.fx.smoke(p.pos, vec2(gen_range(-50.0, 50.0), -10.0), 2.5, 0.35, Color::new(0.5, 0.45, 0.55, 0.35));
                }
            }
        }
        if p.on_ground && p.vel.x.abs() > 1.0 {
            p.anim_t += dt * p.vel.x.abs() / 150.0;
        } else {
            p.anim_t = 0.0;
        }

        // weapon select
        let p = &mut self.player;
        let mut changed = false;
        if let Some(sel) = input.select {
            if sel < NUM_WEAPONS && p.owned[sel] && p.weapon != sel {
                p.weapon = sel;
                changed = true;
            }
        }
        if input.next || input.prev {
            let step = if input.next { 1 } else { NUM_WEAPONS - 1 };
            let mut w = p.weapon;
            for _ in 0..NUM_WEAPONS {
                w = (w + step) % NUM_WEAPONS;
                if p.owned[w] {
                    break;
                }
            }
            if w != p.weapon {
                p.weapon = w;
                changed = true;
            }
        }
        if changed {
            p.fire_cd = p.fire_cd.max(0.12);
            audio.play("switch", 0.5);
        }

        // firing
        p.fire_cd -= dt;
        if input.fire && p.fire_cd <= 0.0 {
            let w = p.weapon;
            let def = &WEAPONS[w];
            if def.clip > 0 && p.ammo[w] == 0 {
                p.owned[w] = false;
                p.weapon = PULSE;
                audio.play("empty", 0.5);
            } else {
                p.fire_cd = def.delay;
                if def.clip > 0 {
                    p.ammo[w] -= 1;
                    if p.ammo[w] == 0 {
                        p.owned[w] = false;
                    }
                }
                self.fire(w, audio);
                let p = &mut self.player;
                if def.clip > 0 && p.ammo[w] == 0 {
                    p.weapon = PULSE;
                    p.fire_cd = p.fire_cd.max(0.25);
                }
            }
        }
    }

    pub fn kill_player(&mut self, audio: &mut Audio) {
        if !self.player.alive {
            return;
        }
        let p = &mut self.player;
        p.alive = false;
        p.hp = 0.0;
        p.death_t = 0.0;
        p.vel = vec2(-p.facing * 80.0, -160.0);
        audio.play("death", 0.9);
        audio.music(Track::Elegy);
        let c = p.center();
        self.fx.ichor(c, vec2(0.0, -1.0), 50, Color::from_rgba(170, 20, 30, 255), 220.0);
        self.fx.explosion(c, 30.0, Color::from_rgba(255, 80, 60, 255));
        self.fx.flash(Color::new(1.0, 0.1, 0.1, 1.0), 0.8);
        self.fx.shake(10.0);
    }

    pub(crate) fn hurt_player(&mut self, dmg: f32, audio: &mut Audio) {
        let p = &mut self.player;
        if !p.alive || p.iframes > 0.0 || self.powers[Power::ElderWard as usize] > 0.0 {
            return;
        }
        p.hp -= dmg;
        p.iframes = 0.4;
        p.hurt_flash = 1.0;
        audio.play("hurt", 0.8);
        self.fx.shake(3.0);
        self.fx.flash(Color::new(1.0, 0.0, 0.0, 1.0), 0.35);
        let c = p.center();
        self.fx.ichor(c, vec2(0.0, -1.0), 8, Color::from_rgba(170, 20, 30, 255), 140.0);
        if self.player.hp <= 0.0 {
            self.kill_player(audio);
        }
    }

    // -----------------------------------------------------------------------
    fn fire(&mut self, w: usize, audio: &mut Audio) {
        let def = &WEAPONS[w];
        let mult = if self.power(Power::Overcharge) { 3.0 } else { 1.0 };
        let p = &self.player;
        let sh = p.shoulder();
        let dir = p.aim_dir;
        let ang = dir.y.atan2(dir.x);
        let muzzle = sh + dir * (def.muzzle - 4.0);
        let perp = vec2(-dir.y, dir.x);
        let pvel = p.vel;
        let facing = p.facing;
        self.stats.shots += 1;
        audio.play_throttled(def.sound, if def.shot == Shot::Flame { 0.35 } else { 0.55 }, if def.shot == Shot::Flame { 0.09 } else { 0.03 });
        self.fx.shake(def.shake);
        self.player.recoil = (def.shake * 0.8 + 1.0).min(5.0);
        self.player.muzzle_flash = 0.05;

        let mk = |pos: Vec2, vel: Vec2, shot: Shot, life: f32| Bullet {
            pos,
            vel,
            shot,
            damage: def.damage * mult,
            splash: def.splash * mult,
            radius: def.radius,
            life,
            age: 0.0,
            speed: def.speed,
            target: None,
            hit: Vec::new(),
            dead: false,
            landed: false,
        };

        match def.shot {
            Shot::Bullet | Shot::Pellet | Shot::Smart => {
                for i in 0..def.count {
                    let a = ang + gen_range(-def.spread, def.spread);
                    let v = vec2(a.cos(), a.sin()) * def.speed * if def.shot == Shot::Pellet { gen_range(0.8, 1.1) } else { 1.0 };
                    let off = if def.count == 2 { perp * (if i == 0 { -2.0 } else { 2.0 }) } else { Vec2::ZERO };
                    let life = if def.shot == Shot::Pellet { gen_range(0.32, 0.5) } else { 1.4 };
                    self.bullets.push(mk(muzzle + off, v, def.shot, life));
                }
                if def.shot != Shot::Pellet {
                    self.fx.shell(sh + dir * 10.0, facing);
                } else {
                    self.fx.shell(sh + dir * 10.0, facing);
                    for _ in 0..5 {
                        self.fx.smoke(muzzle, dir * 40.0, 3.0, 0.4, Color::new(0.5, 0.5, 0.55, 0.4));
                    }
                }
                self.fx.glow(muzzle, 26.0, 0.05, Color::new(1.0, 0.8, 0.4, 1.0));
            }
            Shot::Rocket | Shot::Rpg | Shot::Seeker => {
                for i in 0..def.count {
                    let a = if def.count > 1 { ang + (i as f32 - (def.count - 1) as f32 * 0.5) * def.spread } else { ang };
                    let d = vec2(a.cos(), a.sin());
                    let v = if def.shot == Shot::Rpg { d * 70.0 } else { d * def.speed };
                    let mut b = mk(muzzle, v, def.shot, 4.0);
                    if def.shot == Shot::Seeker {
                        b.target = self.nearest_enemy(self.player.shoulder() + dir * 200.0, 2000.0);
                    }
                    self.bullets.push(b);
                }
                for _ in 0..6 {
                    self.fx.smoke(sh - dir * 6.0, -dir * gen_range(40.0, 120.0), 4.0, 0.6, Color::new(0.6, 0.6, 0.6, 0.45));
                }
                self.fx.glow(muzzle, 36.0, 0.08, Color::new(1.0, 0.7, 0.3, 1.0));
            }
            Shot::Grenade => {
                let v = dir * def.speed + pvel * 0.3;
                self.bullets.push(mk(muzzle, v, Shot::Grenade, 1.5));
                self.fx.smoke(muzzle, dir * 30.0, 3.0, 0.4, Color::new(0.6, 0.5, 0.7, 0.4));
            }
            Shot::Mine => {
                let pos = self.player.pos + vec2(facing * 6.0, -6.0);
                self.bullets.push(mk(pos, vec2(facing * 40.0, -80.0), Shot::Mine, 30.0));
            }
            Shot::Flame => {
                for _ in 0..2 {
                    let a = ang + gen_range(-def.spread, def.spread);
                    let v = vec2(a.cos(), a.sin()) * def.speed * gen_range(0.75, 1.1) + pvel * 0.5;
                    self.bullets.push(mk(muzzle, v, Shot::Flame, gen_range(0.38, 0.5)));
                }
            }
            Shot::Rail => self.fire_rail(muzzle, dir, def.damage * mult, w == NULL_LANCE, audio),
            Shot::Singularity => {
                self.bullets.push(mk(muzzle, dir * def.speed, Shot::Singularity, 7.0));
            }
            Shot::Harpoon => {
                self.bullets.push(mk(muzzle, dir * def.speed, Shot::Harpoon, 1.1));
            }
        }
    }

    fn fire_rail(&mut self, from: Vec2, dir: Vec2, dmg: f32, null: bool, audio: &mut Audio) {
        let to = from + dir * 1500.0;
        let col = if null { Color::from_rgba(255, 60, 160, 255) } else { Color::from_rgba(90, 255, 170, 255) };
        self.fx.beams.push(crate::fx::Beam { a: from, b: to, life: 0.45, max: 0.45, color: col, width: if null { 4.0 } else { 3.0 } });
        // spiral sparks along the beam
        for i in 0..40 {
            let t = i as f32 / 40.0 * 600.0;
            let pp = from + dir * t + vec2(-dir.y, dir.x) * (t * 0.08).sin() * 4.0;
            self.fx.add(Particle { pos: pp, vel: rand_dir() * 20.0, life: 0.5, max: 0.5, size: 1.0, color: col, kind: PKind::Spark, grav: 0.0, drag: 2.0 });
        }
        let mut hits = Vec::new();
        for (i, e) in self.enemies.iter().enumerate() {
            if e.dead || e.emerge < 0.4 {
                continue;
            }
            let rel = e.pos - from;
            let t = rel.dot(dir);
            if t < 0.0 {
                continue;
            }
            let closest = from + dir * t;
            if closest.distance(e.pos) < e.radius() + 3.0 {
                hits.push(i);
            }
        }
        for i in hits {
            let d = if null && self.enemies[i].kind == Kind::Leviathan { 280.0 } else { dmg };
            let pos = self.enemies[i].pos;
            self.fx.spark(pos, dir, 12, col, 300.0);
            self.stats.hits += 1;
            self.damage_enemy(i, d, dir, audio);
        }
    }

    pub(crate) fn nearest_enemy(&self, from: Vec2, max: f32) -> Option<u32> {
        let mut best = None;
        let mut bd = max;
        for e in &self.enemies {
            if e.dead || e.emerge < 0.5 || e.kind == Kind::Spawnling {
                continue;
            }
            let d = e.pos.distance(from);
            if d < bd {
                bd = d;
                best = Some(e.id);
            }
        }
        best
    }

    pub(crate) fn enemy_by_id(&self, id: u32) -> Option<usize> {
        self.enemies.iter().position(|e| e.id == id && !e.dead)
    }

    // -----------------------------------------------------------------------
    fn update_bullets(&mut self, dt: f32, audio: &mut Audio) {
        let mut explosions: Vec<(Vec2, f32, f32, bool)> = Vec::new(); // pos, radius, dmg, singularity
        let mut direct: Vec<(usize, f32, Vec2)> = Vec::new();
        let mut harpooned: Vec<usize> = Vec::new();
        let mut new_pillars: Vec<Pillar> = Vec::new();
        let mut bounce_sfx = false;
        let player_pos = self.player.center();

        for bi in 0..self.bullets.len() {
            let (shot, target) = (self.bullets[bi].shot, self.bullets[bi].target);
            // homing needs read access to enemies
            let mut steer: Option<Vec2> = None;
            match shot {
                Shot::Seeker => {
                    let tid = match target {
                        Some(t) if self.enemy_by_id(t).is_some() => Some(t),
                        _ => self.nearest_enemy(self.bullets[bi].pos, 900.0),
                    };
                    self.bullets[bi].target = tid;
                    if let Some(ix) = tid.and_then(|t| self.enemy_by_id(t)) {
                        steer = Some(self.enemies[ix].pos);
                    }
                }
                Shot::Smart => {
                    let b = &self.bullets[bi];
                    let dir = b.vel.normalize_or_zero();
                    let mut best = None;
                    let mut bd = 260.0;
                    for e in &self.enemies {
                        if e.dead || e.emerge < 0.5 {
                            continue;
                        }
                        let to = e.pos - b.pos;
                        let d = to.length();
                        if d < bd && to.normalize_or_zero().dot(dir) > 0.3 {
                            bd = d;
                            best = Some(e.pos);
                        }
                    }
                    steer = best;
                }
                _ => {}
            }

            let level = &self.level;
            let b = &mut self.bullets[bi];
            b.age += dt;
            b.life -= dt;
            match b.shot {
                Shot::Rpg => {
                    if b.age > 0.28 {
                        let d = b.vel.normalize_or_zero();
                        let sp = (b.vel.length() + 2600.0 * dt).min(b.speed);
                        b.vel = d * sp;
                        if gen_range(0, 2) == 0 {
                            self.fx.smoke(b.pos, -b.vel * 0.05, 2.5, 0.5, Color::new(0.6, 0.6, 0.6, 0.35));
                        }
                    }
                }
                Shot::Rocket | Shot::Seeker => {
                    if let Some(t) = steer {
                        let want = (t - b.pos).normalize_or_zero();
                        let cur = b.vel.normalize_or_zero();
                        let turn = 4.5 * dt;
                        let nd = (cur + (want - cur) * turn.min(1.0)).normalize_or_zero();
                        b.vel = nd * b.speed;
                    }
                    if gen_range(0, 2) == 0 {
                        self.fx.smoke(b.pos, -b.vel * 0.05, 2.5, 0.5, Color::new(0.6, 0.6, 0.6, 0.35));
                    }
                }
                Shot::Smart => {
                    if let Some(t) = steer {
                        let want = (t - b.pos).normalize_or_zero();
                        let cur = b.vel.normalize_or_zero();
                        let nd = (cur + (want - cur) * (10.0 * dt).min(1.0)).normalize_or_zero();
                        b.vel = nd * b.speed;
                    }
                }
                Shot::Grenade => {
                    b.vel.y += 760.0 * dt;
                }
                Shot::Mine => {
                    if !b.landed {
                        b.vel.y += 900.0 * dt;
                    } else if b.age > 0.5 {
                        b.dead = true;
                        new_pillars.push(Pillar { x: b.pos.x, y: b.pos.y + 4.0, life: 3.0, dps: b.damage });
                    }
                }
                Shot::Flame => {
                    b.vel *= 1.0 / (1.0 + 2.6 * dt);
                    b.vel.y -= 90.0 * dt;
                }
                Shot::Singularity => {
                    // drags particles in
                    if gen_range(0, 2) == 0 {
                        let d = rand_dir();
                        self.fx.add(Particle { pos: b.pos + d * 30.0, vel: -d * 60.0, life: 0.45, max: 0.45, size: 1.0, color: Color::from_rgba(200, 120, 255, 255), kind: PKind::Spark, grav: 0.0, drag: 0.0 });
                    }
                }
                _ => {}
            }
            let old = b.pos;
            let mut np = b.pos + b.vel * dt;

            // world collision
            match b.shot {
                Shot::Grenade => {
                    if level.solid_at(np) || level.platforms.iter().any(|pl| pl.contains(np) && b.vel.y > 0.0) {
                        let hx = level.solid_at(vec2(np.x, old.y));
                        if hx {
                            b.vel.x *= -0.6;
                        } else {
                            b.vel.y *= -0.55;
                            b.vel.x *= 0.8;
                        }
                        np = old;
                        if b.vel.length() > 60.0 {
                            bounce_sfx = true;
                        }
                    }
                }
                Shot::Mine => {
                    if !b.landed {
                        let floor = level.floor_below(np.x, old.y);
                        if np.y >= floor - 3.0 {
                            np.y = floor - 3.0;
                            b.landed = true;
                            b.age = 0.0;
                            b.vel = Vec2::ZERO;
                        }
                        if level.solid_at(vec2(np.x, np.y - 1.0)) {
                            np.x = old.x;
                            b.vel.x = 0.0;
                        }
                    }
                }
                Shot::Flame => {
                    if level.solid_at(np) {
                        b.life = b.life.min(0.05);
                        np = old;
                    }
                }
                Shot::Rail => {}
                _ => {
                    if level.solid_at(np) {
                        b.dead = true;
                        if b.radius > 0.0 {
                            explosions.push((old, b.radius, b.splash, b.shot == Shot::Singularity));
                        } else {
                            self.fx.spark(old, -b.vel.normalize_or_zero(), 4, Color::from_rgba(255, 220, 140, 255), 150.0);
                        }
                    }
                }
            }
            b.pos = np;
            if b.pos.x < -120.0 || b.pos.x > WORLD_W + 120.0 || b.pos.y < -400.0 || b.pos.y > VH + 60.0 {
                b.dead = true;
            }
            if b.life <= 0.0 && !b.dead {
                b.dead = true;
                if matches!(b.shot, Shot::Grenade | Shot::Rocket | Shot::Rpg | Shot::Seeker | Shot::Singularity) {
                    explosions.push((b.pos, b.radius, b.splash, b.shot == Shot::Singularity));
                }
            }
            if b.dead {
                continue;
            }

            // enemy collision
            let br = match b.shot {
                Shot::Flame => 4.0 + b.age * 14.0,
                Shot::Singularity => 10.0,
                Shot::Rocket | Shot::Rpg | Shot::Seeker | Shot::Grenade => 4.0,
                Shot::Mine => 0.0,
                _ => 2.0,
            };
            if b.shot == Shot::Mine {
                continue;
            }
            for (ei, e) in self.enemies.iter().enumerate() {
                if e.dead || e.emerge < 0.5 || e.harpooned.is_some() {
                    continue;
                }
                if e.pos.distance(b.pos) > e.radius() + br {
                    continue;
                }
                match b.shot {
                    Shot::Flame => {
                        if b.hit.contains(&e.id) {
                            continue;
                        }
                        b.hit.push(e.id);
                        direct.push((ei, b.damage, b.vel.normalize_or_zero()));
                    }
                    Shot::Harpoon => {
                        b.dead = true;
                        if e.kind == Kind::Leviathan {
                            direct.push((ei, b.damage, b.vel.normalize_or_zero()));
                        } else {
                            harpooned.push(ei);
                        }
                        break;
                    }
                    Shot::Singularity => {
                        b.dead = true;
                        explosions.push((b.pos, b.radius, b.splash, true));
                        break;
                    }
                    Shot::Grenade | Shot::Rocket | Shot::Rpg | Shot::Seeker => {
                        b.dead = true;
                        direct.push((ei, b.damage, b.vel.normalize_or_zero()));
                        explosions.push((b.pos, b.radius, b.splash, false));
                        break;
                    }
                    _ => {
                        b.dead = true;
                        direct.push((ei, b.damage, b.vel.normalize_or_zero()));
                        break;
                    }
                }
            }
        }
        if bounce_sfx {
            audio.play("bounce", 0.4);
        }

        // apply direct hits
        for (ei, dmg, dir) in direct {
            if ei < self.enemies.len() && !self.enemies[ei].dead {
                self.stats.hits += 1;
                let ep = self.enemies[ei].pos;
                let col = info(self.enemies[ei].kind).ichor;
                self.fx.ichor(ep - dir * 6.0, -dir, 3, col, 120.0);
                self.fx.spark(ep - dir * 8.0, -dir, 2, Color::from_rgba(255, 240, 200, 255), 160.0);
                audio.play_throttled("hit", 0.35, 0.06);
                // Heli Attack's "hyperjump push": hitting a horror from above shoves you upward
                let p = &mut self.player;
                if !p.on_ground && p.pos.y < ep.y - 8.0 && dir.y > 0.2 {
                    p.vel.y = (p.vel.y - 55.0).max(-430.0);
                }
                self.damage_enemy(ei, dmg, dir, audio);
            }
        }
        for ei in harpooned {
            if !self.enemies[ei].dead && self.enemies[ei].harpooned.is_none() {
                self.enemies[ei].harpooned = Some(0.0);
                self.enemies[ei].vel = vec2(0.0, 60.0);
                let id = self.enemies[ei].id;
                self.harpoon_lines.push((self.player.shoulder(), id));
                audio.play("hit", 0.7);
                self.stats.hits += 1;
            }
        }
        for (pos, radius, dmg, sing) in explosions {
            if sing {
                self.singularity(pos, radius, audio);
            } else {
                self.explode(pos, radius, dmg, audio);
            }
        }
        for p in new_pillars {
            audio.play("firepillar", 0.6);
            self.pillars.push(p);
        }
        let _ = player_pos;
        self.bullets.retain(|b| !b.dead);
    }

    pub(crate) fn explode(&mut self, pos: Vec2, radius: f32, dmg: f32, audio: &mut Audio) {
        audio.play("explosion", 0.6);
        self.fx.explosion(pos, radius, Color::from_rgba(255, 170, 80, 255));
        let mut hits = Vec::new();
        for (i, e) in self.enemies.iter().enumerate() {
            if e.dead || e.emerge < 0.3 {
                continue;
            }
            let d = e.pos.distance(pos) - e.radius();
            if d < radius {
                let k = 1.0 - (d.max(0.0) / radius) * 0.6;
                hits.push((i, dmg * k, (e.pos - pos).normalize_or_zero()));
            }
        }
        for (i, d, dir) in hits {
            self.damage_enemy(i, d, dir, audio);
        }
        // explosions never hurt the marine (true to the original) - they just shove you
        let pc = self.player.center();
        let d = pc.distance(pos);
        if d < radius * 1.3 && self.player.alive {
            let push = (pc - pos).normalize_or_zero() * (1.0 - d / (radius * 1.3)) * 260.0;
            self.player.vel += push;
        }
    }

    fn singularity(&mut self, pos: Vec2, radius: f32, audio: &mut Audio) {
        audio.play("singularity", 0.9);
        self.fx.flash(Color::new(0.7, 0.3, 1.0, 1.0), 1.0);
        self.fx.shake(14.0);
        self.fx.glow(pos, radius * 6.0, 0.8, Color::new(0.6, 0.2, 1.0, 1.0));
        self.fx.glow(pos, radius * 3.0, 0.5, Color::new(1.0, 1.0, 1.0, 1.0));
        for k in 0..3 {
            self.fx.rings.push(crate::fx::Ring { pos, r: 2.0, max_r: radius * (1.6 + k as f32 * 0.8), life: 0.6 + k as f32 * 0.2, max: 0.6 + k as f32 * 0.2, color: Color::from_rgba(200, 120, 255, 255) });
        }
        for _ in 0..120 {
            let d = rand_dir();
            self.fx.add(Particle { pos: pos + d * gen_range(0.0, 10.0), vel: d * gen_range(50.0, 600.0), life: gen_range(0.4, 1.2), max: 1.2, size: gen_range(1.0, 4.0), color: Color::from_rgba(0, 0, 0, 255), kind: PKind::Shard, grav: 0.0, drag: 2.0 });
        }
        let mut hits = Vec::new();
        for (i, e) in self.enemies.iter().enumerate() {
            if e.dead {
                continue;
            }
            if e.pos.distance(pos) - e.radius() < radius {
                hits.push(i);
            }
        }
        for i in hits {
            let d = if self.enemies[i].kind == Kind::Leviathan { 900.0 } else { 99999.0 };
            self.damage_enemy(i, d, Vec2::ZERO, audio);
        }
        self.eshots.retain(|s| s.pos.distance(pos) > radius * 1.5);
    }

    pub(crate) fn damage_enemy(&mut self, i: usize, dmg: f32, _dir: Vec2, audio: &mut Audio) {
        let e = &mut self.enemies[i];
        if e.dead {
            return;
        }
        let real = dmg.min(e.hp.max(0.0));
        e.hp -= dmg;
        e.flash = 0.08;
        self.stats.damage += real;
        self.score += real as u64;
        if e.hp <= 0.0 {
            self.kill_enemy(i, audio);
        }
    }

    pub(crate) fn kill_enemy(&mut self, i: usize, audio: &mut Audio) {
        let (kind, pos, was_harpooned) = {
            let e = &mut self.enemies[i];
            if e.dead {
                return;
            }
            e.dead = true;
            (e.kind, e.pos, e.harpooned.is_some())
        };
        let inf = info(kind);
        let mult = 1.0 + self.threat as f32 * 0.1;
        self.score += (inf.score as f32 * mult) as u64;
        self.fx.ichor(pos, vec2(0.0, -1.0), (inf.radius * 2.0) as usize + 10, inf.ichor, 260.0);
        self.fx.glow(pos, inf.radius * 5.0, 0.35, inf.ichor);
        for _ in 0..(inf.radius as usize) {
            let d = rand_dir();
            self.fx.smoke(pos + d * inf.radius * 0.5, d * 40.0, gen_range(4.0, 9.0), gen_range(0.6, 1.2), Color::new(inf.ichor.r * 0.3, inf.ichor.g * 0.3, inf.ichor.b * 0.3, 0.6));
        }
        self.fx.rings.push(crate::fx::Ring { pos, r: 2.0, max_r: inf.radius * 2.2, life: 0.4, max: 0.4, color: inf.ichor });
        self.fx.shake(inf.radius / 6.0);
        audio.play("splat", 0.8);

        if kind == Kind::Spawnling {
            return;
        }
        match kind {
            Kind::Gazer => audio.play("gazer", 0.6),
            Kind::Maw => audio.play("maw", 0.6),
            Kind::Seraph => audio.play("seraph", 0.6),
            _ => {}
        }
        self.kills += 1;
        self.kills_since_crate += 1;

        if kind == Kind::Leviathan {
            self.bosses_killed += 1;
            self.next_boss_at = self.kills + 45;
            audio.play("boss", 1.0);
            audio.music(Track::Ichor);
            self.fx.flash(WHITE, 1.0);
            for k in 0..8 {
                let off = rand_dir() * gen_range(10.0, 60.0);
                self.fx.explosion(pos + off, 30.0 + k as f32 * 4.0, Color::from_rgba(255, 80, 160, 255));
            }
            self.banner("THE CROWN IS BROKEN", "IT WILL REMEMBER YOU", Color::from_rgba(255, 120, 200, 255));
            for k in 0..3 {
                let x = (pos.x - 80.0 + k as f32 * 80.0).clamp(30.0, WORLD_W - 30.0);
                self.spawn_crate(x, Some(Content::Mystery));
            }
            // the rest of the swarm dies with it
            for j in 0..self.enemies.len() {
                if !self.enemies[j].dead && self.enemies[j].kind == Kind::Spawnling {
                    self.enemies[j].dead = true;
                }
            }
        }

        // aerial kill reward (Heli Attack 2: killing from a HyperJump hands you a weapon)
        let p = &self.player;
        let aerial = p.alive && !p.on_ground && (p.hyper || self.powers[Power::JumpJets as usize] > 0.0);
        if aerial && kind != Kind::Leviathan {
            self.stats.aerial_kills += 1;
            let pool = [1usize, 2, 3, 4, 5, 6, 7, 9, 13];
            let w = pool[gen_range(0, pool.len())];
            let amt = (WEAPONS[w].clip / 5).max(1);
            let p = &mut self.player;
            p.owned[w] = true;
            p.ammo[w] += amt;
            let c = p.center();
            self.fx.popup(c + vec2(0.0, -24.0), format!("AERIAL KILL  +{}", WEAPONS[w].name), Color::from_rgba(140, 255, 200, 255), 1.0);
            audio.play("pickup", 0.6);
        }
        if was_harpooned {
            self.fx.popup(pos, "HARPOONED", Color::from_rgba(255, 200, 120, 255), 1.0);
        }

        // threat level
        let new_threat = self.kills / 10;
        if new_threat > self.threat {
            self.threat = new_threat;
            let t = format!("THREAT LEVEL {}", self.threat + 1);
            self.banner(&t, "THE SKY IS GETTING THINNER", Color::from_rgba(255, 90, 90, 255));
            audio.play("rift", 0.7);
        }
        // every 3 kills: a supply drop
        if self.kills_since_crate >= 3 {
            self.kills_since_crate = 0;
            let x = (self.player.pos.x + gen_range(-220.0, 220.0)).clamp(30.0, WORLD_W - 30.0);
            self.spawn_crate(x, None);
        }
        // medkits at 15, 30, 60, 120...
        if self.kills >= self.next_medkit {
            self.next_medkit *= 2;
            let x = (self.player.pos.x + gen_range(-160.0, 160.0)).clamp(30.0, WORLD_W - 30.0);
            self.spawn_crate(x, Some(Content::Medkit));
        }
        // boss
        if self.kills >= self.next_boss_at && !self.boss_pending && !self.boss_alive() {
            self.boss_pending = true;
            self.next_boss_at = u32::MAX;
            self.banner("THE HOLLOW CROWN DESCENDS", "KILL IT OR BE KNOWN BY IT", Color::from_rgba(255, 50, 120, 255));
            audio.music(Track::Static);
            audio.play("boss", 0.9);
            let pos = vec2((self.cam_x + VW * 0.5).clamp(200.0, WORLD_W - 200.0), 70.0);
            let hp_scale = 1.0 + self.bosses_killed as f32 * 0.4 + self.threat as f32 * 0.05;
            self.rifts.push(RiftPortal { pos, t: -1.0, kind: Kind::Leviathan, hp_scale, spawned: false });
        }
    }

    pub fn boss_alive(&self) -> bool {
        self.enemies.iter().any(|e| !e.dead && e.kind == Kind::Leviathan)
    }

    pub(crate) fn spawn_crate(&mut self, x: f32, content: Option<Content>) {
        let content = content.unwrap_or_else(|| {
            let r = gen_range(0.0, 1.0);
            if self.kills >= 6 && r < 0.17 {
                let ps = [Power::Overcharge, Power::VoidSight, Power::JumpJets, Power::ElderWard, Power::TimeRift];
                Content::Power(ps[gen_range(0, ps.len())])
            } else if r < 0.30 {
                Content::Mystery
            } else {
                Content::Weapon(random_weapon())
            }
        });
        self.crates.push(Crate { pos: vec2(x, -20.0), vel: vec2(0.0, 45.0), content, landed: false, life: 16.0, t: 0.0 });
    }

    // -----------------------------------------------------------------------
    fn update_crates(&mut self, dt: f32, audio: &mut Audio) {
        let pr = self.player.rect();
        let alive = self.player.alive;
        let mut picked = Vec::new();
        for (i, c) in self.crates.iter_mut().enumerate() {
            c.t += dt;
            if !c.landed {
                c.pos.x += (c.t * 1.7).sin() * 12.0 * dt;
                c.pos += c.vel * dt;
                let floor = self.level.floor_below(c.pos.x, c.pos.y - 8.0);
                if c.pos.y >= floor {
                    c.pos.y = floor;
                    c.landed = true;
                }
            } else {
                c.life -= dt;
            }
            let r = Rect::new(c.pos.x - 8.0, c.pos.y - 16.0, 16.0, 16.0);
            if alive && r.overlaps(&pr) {
                picked.push(i);
            }
        }
        for &i in picked.iter().rev() {
            let c = self.crates.remove(i);
            self.collect(c.content, c.pos, audio);
        }
        self.crates.retain(|c| c.life > 0.0);
    }

    fn collect(&mut self, content: Content, pos: Vec2, audio: &mut Audio) {
        let content = if content == Content::Mystery {
            if gen_range(0, 4) == 0 && self.kills >= 3 {
                let ps = [Power::Overcharge, Power::VoidSight, Power::JumpJets, Power::ElderWard, Power::TimeRift];
                Content::Power(ps[gen_range(0, ps.len())])
            } else {
                Content::Weapon(random_weapon())
            }
        } else {
            content
        };
        self.fx.glow(pos + vec2(0.0, -8.0), 60.0, 0.3, Color::new(1.0, 1.0, 0.8, 1.0));
        match content {
            Content::Weapon(w) => {
                let p = &mut self.player;
                p.owned[w] = true;
                p.ammo[w] += WEAPONS[w].clip;
                if p.weapon != NULL_LANCE {
                    p.weapon = w;
                }
                p.fire_cd = p.fire_cd.max(0.15);
                audio.play("pickup", 0.7);
                audio.play("switch", 0.4);
                self.fx.popup(pos + vec2(0.0, -20.0), WEAPONS[w].name, Color::from_rgba(255, 230, 140, 255), 1.0);
            }
            Content::Medkit => {
                self.player.hp = (self.player.hp + 45.0).min(MAX_HP);
                audio.play("medkit", 0.8);
                self.fx.popup(pos + vec2(0.0, -20.0), "MEDKIT +45", Color::from_rgba(255, 120, 120, 255), 1.0);
            }
            Content::Power(pw) => {
                self.powers[pw as usize] = POWER_TIME;
                audio.play("powerup", 0.8);
                let (_, name, sub) = POWERS[pw as usize];
                self.banner(name, sub, Color::from_rgba(255, 240, 160, 255));
                match pw {
                    Power::VoidSight => {
                        let p = &mut self.player;
                        if p.weapon != NULL_LANCE {
                            p.pre_void_weapon = p.weapon;
                        }
                        p.owned[NULL_LANCE] = true;
                        p.weapon = NULL_LANCE;
                    }
                    Power::TimeRift => audio.play("timerift", 0.7),
                    _ => {}
                }
            }
            Content::Mystery => {}
        }
    }

    // -----------------------------------------------------------------------
    fn update_pillars(&mut self, dt: f32, audio: &mut Audio) {
        let mut hits = Vec::new();
        for p in self.pillars.iter_mut() {
            p.life -= dt;
            let top = 40.0;
            for _ in 0..3 {
                let y = gen_range(top, p.y);
                self.fx.add(Particle { pos: vec2(p.x + gen_range(-7.0, 7.0), y), vel: vec2(gen_range(-20.0, 20.0), -gen_range(80.0, 200.0)), life: 0.4, max: 0.4, size: gen_range(1.0, 3.0), color: Color::new(1.0, gen_range(0.3, 0.8), 0.1, 1.0), kind: PKind::Spark, grav: -100.0, drag: 1.0 });
            }
            for (i, e) in self.enemies.iter().enumerate() {
                if e.dead || e.emerge < 0.5 {
                    continue;
                }
                if (e.pos.x - p.x).abs() < e.radius() + 10.0 && e.pos.y > top - e.radius() && e.pos.y < p.y + 10.0 {
                    hits.push((i, p.dps * dt));
                }
            }
        }
        self.pillars.retain(|p| p.life > 0.0);
        for (i, d) in hits {
            self.damage_enemy(i, d, Vec2::ZERO, audio);
        }
    }
}

/// Strict AABB overlap (touching edges do not count - macroquad's `Rect::overlaps` is inclusive).
fn hits(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w - 0.001 && a.x + a.w > b.x + 0.001 && a.y < b.y + b.h - 0.001 && a.y + a.h > b.y + 0.001
}

pub fn random_weapon() -> usize {
    // weighted like the original: common guns often, exotics rarely
    let table: [(usize, u32); 13] = [(1, 10), (2, 10), (3, 7), (4, 8), (5, 8), (6, 8), (7, 6), (8, 5), (9, 7), (10, 4), (11, 2), (12, 2), (13, 7)];
    let total: u32 = table.iter().map(|t| t.1).sum();
    let mut r = gen_range(0, total);
    for (w, wt) in table {
        if r < wt {
            return w;
        }
        r -= wt;
    }
    1
}
