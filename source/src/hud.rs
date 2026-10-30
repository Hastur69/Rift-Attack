//! Heads-up display, menus and overlay screens. Drawn at full window
//! resolution (crisp text) using the same 640x360 virtual coordinates as the world.

use crate::assets::{Fonts, Gfx};
use crate::enemies::{info, Kind};
use crate::game::{Game, MAX_HP};
use crate::level::{VH, VW};
use crate::weapons::*;
use macroquad::prelude::*;

pub struct View {
    pub scale: f32,
    pub ox: f32,
    pub oy: f32,
}

impl View {
    pub fn compute() -> View {
        let sw = screen_width();
        let sh = screen_height();
        let mut scale = (sw / VW).min(sh / VH);
        if scale >= 2.0 {
            scale = scale.floor();
        }
        let ox = ((sw - VW * scale) * 0.5).floor();
        let oy = ((sh - VH * scale) * 0.5).floor();
        View { scale, ox, oy }
    }

    /// camera mapping virtual coords onto the letterboxed screen area
    pub fn hud_camera(&self) -> Camera2D {
        // macroquad 0.4's from_display_rect is y-up when drawing straight to the
        // window, so flip it to get y-down screen coordinates.
        let mut c = Camera2D::from_display_rect(Rect::new(-self.ox / self.scale, -self.oy / self.scale, screen_width() / self.scale, screen_height() / self.scale));
        c.zoom.y = -c.zoom.y;
        c
    }

    pub fn mouse_virtual(&self) -> Vec2 {
        let (mx, my) = mouse_position();
        vec2((mx - self.ox) / self.scale, (my - self.oy) / self.scale)
    }
}

pub struct Ui<'a> {
    pub v: &'a View,
    pub f: &'a Fonts,
}

pub const GREEN: Color = Color::new(0.45, 1.0, 0.65, 1.0);
pub const DIM: Color = Color::new(0.45, 0.6, 0.5, 1.0);
pub const AMBER: Color = Color::new(1.0, 0.75, 0.3, 1.0);
pub const BLOOD: Color = Color::new(1.0, 0.22, 0.3, 1.0);

impl<'a> Ui<'a> {
    fn params(&self, size: f32, color: Color, bold: bool) -> TextParams<'a> {
        let px = (size * self.v.scale).round().max(6.0);
        TextParams {
            font: Some(if bold { &self.f.bold } else { &self.f.mono }),
            font_size: px as u16,
            font_scale: 1.0 / self.v.scale,
            color,
            ..Default::default()
        }
    }

    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text_ex(s, x, y, self.params(size, color, false));
    }

    pub fn bold(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text_ex(s, x, y, self.params(size, color, true));
    }

    pub fn width(&self, s: &str, size: f32, bold: bool) -> f32 {
        let px = (size * self.v.scale).round().max(6.0) as u16;
        measure_text(s, Some(if bold { &self.f.bold } else { &self.f.mono }), px, 1.0 / self.v.scale).width
    }

    pub fn center(&self, s: &str, cx: f32, y: f32, size: f32, color: Color, bold: bool) {
        let w = self.width(s, size, bold);
        if bold {
            self.bold(s, cx - w * 0.5, y, size, color);
        } else {
            self.text(s, cx - w * 0.5, y, size, color);
        }
    }

    pub fn right(&self, s: &str, rx: f32, y: f32, size: f32, color: Color, bold: bool) {
        let w = self.width(s, size, bold);
        if bold {
            self.bold(s, rx - w, y, size, color);
        } else {
            self.text(s, rx - w, y, size, color);
        }
    }

    /// text with a 1px drop shadow, for readability over the busy scene
    pub fn shadow_center(&self, s: &str, cx: f32, y: f32, size: f32, color: Color, bold: bool) {
        self.center(s, cx + 0.7, y + 0.7, size, Color::new(0.0, 0.0, 0.0, color.a * 0.8), bold);
        self.center(s, cx, y, size, color, bold);
    }
}

pub fn panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::new(0.02, 0.05, 0.04, 0.72));
    draw_rectangle_lines(x, y, w, h, 0.6, Color::new(0.35, 0.9, 0.55, 0.45));
    // corner ticks
    let c = Color::new(0.45, 1.0, 0.65, 0.9);
    for (cx, cy, dx, dy) in [(x, y, 1.0, 1.0), (x + w, y, -1.0, 1.0), (x, y + h, 1.0, -1.0), (x + w, y + h, -1.0, -1.0)] {
        draw_line(cx, cy, cx + 4.0 * dx, cy, 1.0, c);
        draw_line(cx, cy, cx, cy + 4.0 * dy, 1.0, c);
    }
}

pub fn weapon_icon(g: &Gfx, w: usize, x: f32, y: f32, scale: f32, col: Color) {
    draw_texture_ex(&g.weapons, x, y, col, DrawTextureParams {
        source: Some(Rect::new(0.0, w as f32 * 16.0, 48.0, 16.0)),
        dest_size: Some(vec2(48.0 * scale, 16.0 * scale)),
        ..Default::default()
    });
}

pub fn crosshair(p: Vec2, t: f32, col: Color) {
    let s = 5.0 + (t * 6.0).sin() * 0.6;
    for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        draw_line(p.x + dx * 2.5, p.y + dy * 2.5, p.x + dx * s, p.y + dy * s, 1.0, col);
    }
    draw_rectangle(p.x - 0.5, p.y - 0.5, 1.0, 1.0, col);
}

// ---------------------------------------------------------------------------
// In-game HUD
// ---------------------------------------------------------------------------
pub fn draw_hud(ui: &Ui, g: &Gfx, game: &Game, best_kills: u32) {
    let t = game.time;
    let p = &game.player;

    // ---- vitals + weapon (top-left, kept compact so horrors stay visible)
    panel(6.0, 6.0, 168.0, 30.0);
    ui.bold("VITALS", 10.0, 14.0, 6.0, DIM);
    let hpk = (p.hp / MAX_HP).clamp(0.0, 1.0);
    let segs = 20;
    for i in 0..segs {
        let on = (i as f32) < hpk * segs as f32;
        let c = if !on {
            Color::new(0.15, 0.2, 0.18, 0.8)
        } else if hpk < 0.3 {
            if ((t * 6.0) as i32) % 2 == 0 { BLOOD } else { Color::new(0.6, 0.1, 0.15, 1.0) }
        } else {
            GREEN
        };
        draw_rectangle(44.0 + i as f32 * 5.5, 8.5, 4.5, 6.0, c);
    }
    ui.right(&format!("{}", p.hp.max(0.0).ceil() as i32), 170.0, 14.0, 7.0, if hpk < 0.3 { BLOOD } else { GREEN }, true);
    ui.bold("DISTORT", 10.0, 23.0, 5.0, DIM);
    draw_rectangle(44.0, 19.5, 109.0, 3.0, Color::new(0.1, 0.12, 0.2, 1.0));
    let dc = if game.distorting { Color::new(0.6, 0.85, 1.0, 1.0) } else { Color::new(0.35, 0.55, 1.0, 1.0) };
    draw_rectangle(44.0, 19.5, 109.0 * p.distort, 3.0, dc);
    ui.text("SHIFT", 156.0, 23.0, 4.5, Color::new(0.4, 0.5, 0.6, 1.0));
    let w = &WEAPONS[p.weapon];
    let ammo = if w.clip == 0 { "∞".to_string() } else { format!("{}", p.ammo[p.weapon]) };
    ui.bold(w.name, 10.0, 32.5, 6.0, AMBER);
    ui.right(&ammo, 170.0, 33.0, 7.5, AMBER, true);
    if p.crouch_t > 0.35 && p.on_ground {
        ui.shadow_center("HYPERJUMP READY", p.pos.x - game.cam_x, p.pos.y - 44.0, 5.5, if ((t * 8.0) as i32) % 2 == 0 { GREEN } else { DIM }, true);
    }

    // ---- kills / score (top-right)
    panel(VW - 120.0, 6.0, 114.0, 30.0);
    ui.bold("HORRORS", VW - 116.0, 14.0, 5.5, DIM);
    ui.right(&format!("THREAT {}", game.threat + 1), VW - 10.0, 14.0, 5.5, BLOOD, true);
    ui.bold(&format!("{}", game.kills), VW - 116.0, 31.0, 15.0, Color::new(0.95, 0.95, 0.85, 1.0));
    ui.right(&format!("{}", game.score), VW - 10.0, 24.0, 6.0, GREEN, true);
    ui.right(&format!("BEST {}", best_kills.max(game.kills)), VW - 10.0, 32.0, 5.0, DIM, false);

    // ---- active powers
    let mut py = 46.0;
    for (i, (_, name, _)) in POWERS.iter().enumerate() {
        let left = game.powers[i];
        if left > 0.0 {
            panel(VW - 120.0, py - 6.0, 114.0, 12.0);
            let blink = left < 3.0 && ((t * 8.0) as i32) % 2 == 0;
            ui.bold(name, VW - 116.0, py + 2.5, 6.0, if blink { DIM } else { Color::new(1.0, 0.92, 0.5, 1.0) });
            draw_rectangle(VW - 54.0, py - 1.5, 44.0 * (left / POWER_TIME), 3.0, Color::new(1.0, 0.85, 0.3, 0.9));
            py += 15.0;
        }
    }

    // ---- inventory strip (bottom-left)
    let mut x = 6.0;
    let y = VH - 22.0;
    for wi in 0..NUM_WEAPONS {
        if !p.owned[wi] {
            continue;
        }
        let sel = wi == p.weapon;
        let bw = 30.0;
        draw_rectangle(x, y, bw, 16.0, if sel { Color::new(0.3, 0.6, 0.4, 0.55) } else { Color::new(0.02, 0.05, 0.04, 0.6) });
        if sel {
            draw_rectangle_lines(x, y, bw, 16.0, 0.8, GREEN);
        }
        draw_texture_ex(&g.weapons, x + 1.0, y + 2.0, WHITE, DrawTextureParams {
            source: Some(Rect::new(6.0, wi as f32 * 16.0, 40.0, 16.0)),
            dest_size: Some(vec2(25.0, 10.0)),
            ..Default::default()
        });
        let key = match wi {
            0..=8 => format!("{}", wi + 1),
            9 => "0".into(),
            _ => "".into(),
        };
        ui.text(&key, x + 1.5, y + 6.0, 4.5, DIM);
        if WEAPONS[wi].clip > 0 {
            ui.right(&format!("{}", p.ammo[wi]), x + bw - 1.5, y + 15.0, 4.5, AMBER, false);
        }
        x += bw + 2.0;
    }

    // ---- motion tracker (bottom-right)
    let origin = vec2(VW - 40.0, VH - 12.0);
    let rad = 34.0;
    {
        let (a0, a1) = (200f32.to_radians(), 340f32.to_radians());
        let n = 20;
        for i in 0..n {
            let x0 = a0 + (a1 - a0) * i as f32 / n as f32;
            let x1 = a0 + (a1 - a0) * (i + 1) as f32 / n as f32;
            draw_triangle(origin, origin + vec2(x0.cos(), x0.sin()) * rad, origin + vec2(x1.cos(), x1.sin()) * rad, Color::new(0.02, 0.12, 0.07, 0.75));
        }
        for k in 1..=3 {
            draw_arc_lines(origin, rad * k as f32 / 3.0, Color::new(0.3, 1.0, 0.6, if k == 3 { 0.8 } else { 0.3 }));
        }
        draw_line(origin.x, origin.y, origin.x, origin.y - rad, 0.5, Color::new(0.3, 1.0, 0.6, 0.3));
        for a in [a0, a1] {
            draw_line(origin.x, origin.y, origin.x + a.cos() * rad, origin.y + a.sin() * rad, 0.6, Color::new(0.3, 1.0, 0.6, 0.6));
        }
    }
    let sweep = game.tracker_sweep;
    draw_arc_lines(origin, sweep * rad, Color::new(0.4, 1.0, 0.6, 0.6 * (1.0 - sweep)));
    let mut nearest = f32::MAX;
    for e in &game.enemies {
        let rel = e.pos - p.center();
        let d = rel.length();
        nearest = nearest.min(d);
        let k = (d / 700.0).min(1.0);
        let dir = rel.normalize_or_zero();
        let bp = origin + vec2(dir.x, dir.y.min(-0.3)).normalize_or_zero() * k * rad;
        let size = if e.kind == Kind::Leviathan { 2.5 } else { 1.4 };
        let fade = 0.5 + 0.5 * (1.0 - ((sweep - k).abs() * 3.0).min(1.0));
        draw_circle(bp.x, bp.y, size, Color::new(1.0, 1.0, 1.0, fade));
    }
    let dist = if nearest == f32::MAX { "--".to_string() } else { format!("{}m", (nearest / 8.0) as i32) };
    ui.right(&dist, VW - 6.0, VH - 4.0, 6.0, GREEN, true);
    ui.bold("TRACKER", VW - 74.0, VH - 4.0, 4.5, DIM);

    // ---- boss bar
    if let Some(b) = game.enemies.iter().find(|e| e.kind == Kind::Leviathan) {
        let w = 270.0;
        let x = VW * 0.5 - w * 0.5 - 6.0;
        ui.shadow_center(info(Kind::Leviathan).name, x + w * 0.5, 15.0, 7.0, Color::new(1.0, 0.4, 0.6, 1.0), true);
        draw_rectangle(x - 1.0, 19.0, w + 2.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_rectangle(x, 20.0, w * (b.hp / b.max_hp).clamp(0.0, 1.0) * b.emerge, 4.0, Color::new(0.9, 0.15, 0.45, 1.0));
    }

    // ---- floating popups (world -> virtual)
    for pu in &game.fx.popups {
        let a = (pu.life / 0.5).min(1.0);
        let x = pu.pos.x - game.cam_x;
        ui.shadow_center(&pu.text, x, pu.pos.y, 6.5 * pu.size, Color::new(pu.color.r, pu.color.g, pu.color.b, a), true);
    }

    // ---- banner
    if let Some(b) = &game.banner {
        let a = (b.t / 0.5).min(1.0).min((3.5 - b.t) * 4.0).clamp(0.0, 1.0);
        let jitter = if b.t > 2.7 { ((t * 60.0).sin() * 2.0).round() } else { 0.0 };
        draw_rectangle(0.0, 118.0, VW, 40.0, Color::new(0.0, 0.0, 0.0, 0.45 * a));
        ui.shadow_center(&b.title, VW * 0.5 + jitter, 140.0, 16.0, Color::new(b.color.r, b.color.g, b.color.b, a), true);
        ui.shadow_center(&b.sub, VW * 0.5, 152.0, 6.5, Color::new(0.85, 0.85, 0.85, a), false);
    }

    // ---- heartbeat vignette at low health
    if p.alive && hpk < 0.3 {
        let beat = ((t * 7.0).sin() * 0.5 + 0.5) * (0.3 - hpk) * 1.6;
        for i in 0..8 {
            let k = i as f32 / 8.0;
            let c = Color::new(0.6, 0.0, 0.05, beat * (1.0 - k) * 0.25);
            draw_rectangle_lines(-i as f32 * 3.0, -i as f32 * 3.0, VW + i as f32 * 6.0, VH + i as f32 * 6.0, 40.0, c);
        }
    }
}

fn draw_arc_lines(o: Vec2, r: f32, c: Color) {
    if r < 1.0 {
        return;
    }
    let start = 200f32.to_radians();
    let end = 340f32.to_radians();
    let n = 16;
    for i in 0..n {
        let a0 = start + (end - start) * i as f32 / n as f32;
        let a1 = start + (end - start) * (i + 1) as f32 / n as f32;
        draw_line(o.x + a0.cos() * r, o.y + a0.sin() * r, o.x + a1.cos() * r, o.y + a1.sin() * r, 1.0, c);
    }
}

// ---------------------------------------------------------------------------
// Menus & screens
// ---------------------------------------------------------------------------
pub fn draw_title(ui: &Ui, g: &Gfx, t: f32, items: &[String], sel: usize, best_kills: u32, best_score: u64) {
    let pulse = 0.9 + (t * 1.7).sin() * 0.1;
    draw_texture_ex(&g.title, VW * 0.5 - 220.0, 34.0 + (t * 0.9).sin() * 2.0, Color::new(pulse, pulse, pulse, 1.0), DrawTextureParams {
        dest_size: Some(vec2(440.0, 110.0)),
        ..Default::default()
    });
    let y0 = 175.0;
    for (i, it) in items.iter().enumerate() {
        let y = y0 + i as f32 * 17.0;
        let s = i == sel;
        if s {
            let w = ui.width(it, 9.0, true) + 24.0;
            draw_rectangle(VW * 0.5 - w * 0.5, y - 10.0, w, 14.0, Color::new(0.45, 1.0, 0.65, 0.15));
            ui.center(">", VW * 0.5 - w * 0.5 + 6.0, y, 9.0, GREEN, true);
        }
        ui.shadow_center(it, VW * 0.5, y, 9.0, if s { GREEN } else { Color::new(0.75, 0.8, 0.78, 1.0) }, true);
    }
    if best_kills > 0 {
        ui.shadow_center(&format!("RECORD  {} HORRORS  /  {} PTS", best_kills, best_score), VW * 0.5, VH - 30.0, 6.5, AMBER, true);
    }
    ui.shadow_center("A cosmic-horror reboot of Heli Attack 2 (squarecircleco, 2003)", VW * 0.5, VH - 16.0, 5.5, DIM, false);
    ui.shadow_center("W/S + ENTER  or  MOUSE", VW * 0.5, VH - 6.0, 5.0, Color::new(0.4, 0.45, 0.45, 1.0), false);
}

pub fn draw_manual(ui: &Ui, g: &Gfx, page: usize) {
    panel(20.0, 14.0, VW - 40.0, VH - 28.0);
    let pages = 3;
    ui.bold(&format!("FIELD MANUAL  {}/{}", page + 1, pages), 30.0, 30.0, 9.0, GREEN);
    ui.right("A/D or ←/→  page    ESC  back", VW - 30.0, 30.0, 6.0, DIM, false);
    match page {
        0 => {
            let lines: [(&str, &str); 14] = [
                ("A / D", "move"),
                ("W / SPACE", "jump - press again in the air to double-jump"),
                ("hold S, then JUMP", "HYPERJUMP - a huge leap. Kills mid-hyperjump award a weapon"),
                ("S + JUMP (on catwalk)", "drop through"),
                ("MOUSE", "aim       LEFT BUTTON  fire (hold)"),
                ("WHEEL / Q / E", "cycle weapons    1-9, 0  select directly"),
                ("SHIFT (hold)", "TIME DISTORT - slow everything, meter recharges"),
                ("ESC / P", "pause      F11  fullscreen      M  mute music"),
                ("K", "the coward's way out"),
                ("", ""),
                ("RULES OF ENGAGEMENT", ""),
                ("", "Every 3 kills a supply pod drops. Medkits at 15, 30, 60, 120..."),
                ("", "Your own explosives can't hurt you. They will shove you around."),
                ("", "Shoot a horror while you're above it to stay airborne."),
            ];
            for (i, (k, v)) in lines.iter().enumerate() {
                let y = 52.0 + i as f32 * 19.0;
                ui.bold(k, 34.0, y, 7.0, AMBER);
                ui.text(v, 176.0, y, 7.0, Color::new(0.85, 0.88, 0.85, 1.0));
            }
        }
        1 => {
            for i in 0..NULL_LANCE + 1 {
                let col = i / 8;
                let row = i % 8;
                let x = 30.0 + col as f32 * 296.0;
                let y = 42.0 + row as f32 * 36.0;
                weapon_icon(g, i, x, y, 1.0, WHITE);
                let w = &WEAPONS[i];
                ui.bold(w.name, x + 52.0, y + 8.0, 6.5, AMBER);
                let ammo = if w.clip == 0 { "∞".to_string() } else { format!("x{}", w.clip) };
                ui.right(&ammo, x + 286.0, y + 8.0, 6.0, DIM, false);
                ui.text(w.blurb, x + 52.0, y + 18.0, 5.0, Color::new(0.8, 0.82, 0.8, 1.0));
            }
        }
        _ => {
            ui.bold("POWER-UPS  (white supply pods)", 34.0, 54.0, 8.0, Color::new(1.0, 0.92, 0.5, 1.0));
            for (i, (_, name, d)) in POWERS.iter().enumerate() {
                let y = 74.0 + i as f32 * 16.0;
                ui.bold(name, 40.0, y, 7.0, AMBER);
                ui.text(d, 150.0, y, 7.0, Color::new(0.85, 0.88, 0.85, 1.0));
            }
            ui.bold("KNOWN ENTITIES", 34.0, 172.0, 8.0, BLOOD);
            let ents = [
                (Kind::Gazer, "Hovers, watches, fires bursts of void bolts."),
                (Kind::Maw, "Spits arcs of acid. Births spawnlings that crawl at you."),
                (Kind::Seraph, "Too many eyes. Fans of light. Blinks across the sky."),
                (Kind::Leviathan, "Arrives every ~45 kills. Eye-beam. Ichor rain. Summons."),
            ];
            for (i, (k, d)) in ents.iter().enumerate() {
                let y = 192.0 + i as f32 * 26.0;
                let inf = info(*k);
                let tex = match k {
                    Kind::Gazer => &g.gazer,
                    Kind::Maw => &g.maw,
                    Kind::Seraph => &g.seraph,
                    _ => &g.leviathan,
                };
                let h = 24.0;
                let w = inf.fw * h / inf.fh;
                draw_texture_ex(tex, 40.0 + (30.0 - w) * 0.5, y - 16.0, WHITE, DrawTextureParams {
                    source: Some(Rect::new(0.0, 0.0, inf.fw, inf.fh)),
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                });
                ui.bold(inf.name, 80.0, y - 2.0, 7.0, AMBER);
                ui.text(d, 200.0, y - 2.0, 6.5, Color::new(0.85, 0.88, 0.85, 1.0));
            }
        }
    }
}

pub fn draw_pause(ui: &Ui, items: &[String], sel: usize) {
    draw_rectangle(0.0, 0.0, VW, VH, Color::new(0.0, 0.0, 0.0, 0.55));
    ui.shadow_center("PAUSED", VW * 0.5, 120.0, 20.0, GREEN, true);
    for (i, it) in items.iter().enumerate() {
        let y = 160.0 + i as f32 * 17.0;
        let s = i == sel;
        ui.shadow_center(it, VW * 0.5, y, 9.0, if s { GREEN } else { Color::new(0.75, 0.8, 0.78, 1.0) }, true);
        if s {
            let w = ui.width(it, 9.0, true);
            ui.center(">", VW * 0.5 - w * 0.5 - 10.0, y, 9.0, GREEN, true);
        }
    }
}

pub fn draw_game_over(ui: &Ui, game: &Game, best_kills: u32, best_score: u64, new_record: bool, t: f32, epitaph: &str) {
    let a = (t / 1.0).min(1.0);
    draw_rectangle(0.0, 0.0, VW, VH, Color::new(0.04, 0.0, 0.02, 0.65 * a));
    ui.shadow_center("MARINE DOWN", VW * 0.5, 92.0, 26.0, Color::new(1.0, 0.25, 0.3, a), true);
    ui.shadow_center(epitaph, VW * 0.5, 110.0, 7.0, Color::new(0.8, 0.75, 0.8, a), false);
    panel(VW * 0.5 - 140.0, 124.0, 280.0, 132.0);
    let rows = [
        ("HORRORS KILLED", format!("{}", game.kills)),
        ("SCORE", format!("{}", game.score)),
        ("TIME SURVIVED", format!("{}:{:02}", (game.time as u32) / 60, (game.time as u32) % 60)),
        ("ACCURACY", format!("{:.0}%", if game.stats.shots > 0 { (game.stats.hits as f32 / game.stats.shots as f32 * 100.0).min(100.0) } else { 0.0 })),
        ("AERIAL KILLS", format!("{}", game.stats.aerial_kills)),
        ("CROWNS BROKEN", format!("{}", game.bosses_killed)),
        ("RECORD", format!("{} / {}", best_kills, best_score)),
    ];
    for (i, (k, v)) in rows.iter().enumerate() {
        let y = 142.0 + i as f32 * 16.0;
        ui.bold(k, VW * 0.5 - 128.0, y, 7.5, Color::new(0.6, 0.75, 0.65, a));
        ui.right(v, VW * 0.5 + 128.0, y, 7.5, Color::new(0.95, 0.95, 0.85, a), true);
    }
    if new_record && ((t * 4.0) as i32) % 2 == 0 {
        ui.shadow_center("NEW RECORD", VW * 0.5, 272.0, 11.0, AMBER, true);
    }
    if t > 1.2 {
        ui.shadow_center("ENTER / CLICK  redeploy        ESC  title", VW * 0.5, 300.0, 7.5, GREEN, true);
    }
}
