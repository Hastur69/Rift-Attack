//! The arena: LZ GRAVEN, a colony landing pad under a split sky. One wide level like Heli Attack 2's jungle clearing, with cargo
//! blocks for cover and one-way catwalks to fight from.

use crate::assets::Gfx;
use macroquad::prelude::*;

pub const WORLD_W: f32 = 1280.0;
pub const VW: f32 = 640.0;
pub const VH: f32 = 360.0;
pub const GROUND_Y: f32 = 318.0;

pub struct Level {
    pub solids: Vec<Rect>,
    pub platforms: Vec<Rect>,
}

impl Level {
    pub fn new() -> Level {
        let solids = vec![
            Rect::new(-200.0, GROUND_Y, WORLD_W + 400.0, 200.0),
            Rect::new(196.0, GROUND_Y - 32.0, 64.0, 32.0),
            Rect::new(560.0, GROUND_Y - 64.0, 32.0, 64.0),
            Rect::new(592.0, GROUND_Y - 32.0, 32.0, 32.0),
            Rect::new(896.0, GROUND_Y - 32.0, 96.0, 32.0),
            Rect::new(1104.0, GROUND_Y - 64.0, 64.0, 64.0),
        ];
        let platforms = vec![
            Rect::new(48.0, 236.0, 160.0, 8.0),
            Rect::new(352.0, 214.0, 192.0, 8.0),
            Rect::new(688.0, 236.0, 160.0, 8.0),
            Rect::new(960.0, 206.0, 192.0, 8.0),
            Rect::new(512.0, 150.0, 128.0, 8.0),
            Rect::new(176.0, 162.0, 96.0, 8.0),
            Rect::new(1056.0, 140.0, 96.0, 8.0),
        ];
        Level { solids, platforms }
    }

    pub fn solid_at(&self, p: Vec2) -> bool {
        self.solids.iter().any(|r| r.contains(p))
    }

    /// Height of the floor (solid or platform) directly under x, starting at y.
    pub fn floor_below(&self, x: f32, y: f32) -> f32 {
        let mut best = GROUND_Y;
        for r in self.solids.iter().chain(self.platforms.iter()) {
            if x >= r.x && x <= r.x + r.w && r.y >= y && r.y < best {
                best = r.y;
            }
        }
        best
    }

    pub fn draw_backdrop(&self, g: &Gfx, cam_x: f32, t: f32, lightning: f32) {
        // sky is fixed; slight drift so the tear seems alive
        let drift = (t * 0.7).sin() * 2.0;
        draw_texture(&g.sky, drift - 4.0, 0.0, WHITE);
        draw_texture_ex(&g.sky, drift - 4.0 + 640.0, 0.0, WHITE, DrawTextureParams { flip_x: true, ..Default::default() });
        if lightning > 0.0 {
            draw_rectangle(0.0, 0.0, VW, VH, Color::new(0.75, 0.5, 1.0, lightning * 0.35));
        }
        // far layer, parallax 0.25
        let fx = -(cam_x * 0.25) % 1280.0;
        for k in 0..2 {
            draw_texture(&g.far, fx + k as f32 * 1280.0, VH - 220.0 - 20.0, WHITE);
        }
        // fog band
        for i in 0..6 {
            let a = 0.05 + i as f32 * 0.018;
            draw_rectangle(0.0, 200.0 + i as f32 * 14.0, VW, 14.0, Color::new(0.35, 0.12, 0.4, a));
        }
        // mid layer, parallax 0.55
        let mx = -(cam_x * 0.55) % 1280.0;
        for k in 0..2 {
            draw_texture(&g.mid, mx + k as f32 * 1280.0, GROUND_Y - 260.0 + 12.0, WHITE);
        }
        draw_rectangle(0.0, GROUND_Y - 60.0, VW, 60.0, Color::new(0.1, 0.03, 0.12, 0.35));
    }

    /// Drawn in world space (camera already offset).
    pub fn draw_world(&self, g: &Gfx) {
        // ground
        let mut x = -64.0;
        while x < WORLD_W + 64.0 {
            draw_texture(&g.ground, x, GROUND_Y, WHITE);
            x += 64.0;
        }
        // cargo blocks
        for r in self.solids.iter().skip(1) {
            let mut yy = r.y;
            while yy < r.y + r.h {
                let mut xx = r.x;
                while xx < r.x + r.w {
                    draw_texture(&g.block, xx, yy, WHITE);
                    xx += 32.0;
                }
                yy += 32.0;
            }
        }
        // catwalks + supports
        for p in &self.platforms {
            let floor = self.floor_below(p.x + 6.0, p.y + 9.0);
            for sx in [p.x + 6.0, p.x + p.w - 10.0] {
                draw_rectangle(sx, p.y + 8.0, 4.0, floor - p.y - 8.0, Color::from_rgba(28, 22, 34, 255));
                draw_rectangle(sx + 1.0, p.y + 8.0, 1.0, floor - p.y - 8.0, Color::from_rgba(60, 52, 70, 255));
            }
            let mut xx = p.x;
            while xx < p.x + p.w {
                draw_texture_ex(&g.catwalk, xx, p.y, WHITE, DrawTextureParams {
                    source: Some(Rect::new(0.0, 0.0, (p.x + p.w - xx).min(32.0), 8.0)),
                    ..Default::default()
                });
                xx += 32.0;
            }
            // warning lamp
            draw_rectangle(p.x + p.w - 3.0, p.y - 3.0, 2.0, 3.0, Color::from_rgba(255, 60, 40, 255));
        }
        // edge-of-world pylons
        for x in [-6.0, WORLD_W - 2.0] {
            draw_rectangle(x, 0.0, 8.0, GROUND_Y, Color::from_rgba(20, 10, 26, 255));
            for k in 0..12 {
                draw_rectangle(x + 2.0, 20.0 + k as f32 * 25.0, 4.0, 3.0, Color::from_rgba(255, 170, 40, 200));
            }
        }
    }
}
