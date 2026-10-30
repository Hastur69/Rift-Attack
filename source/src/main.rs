//! RIFT ATTACK: VOID MARINES
//!
//! A cosmic-horror reboot of squarecircleco's Heli Attack 2 (2003).
//! One marine. One landing zone. An endless sky full of things that should not be.

#![windows_subsystem = "windows"]

mod ai;
mod assets;
mod audio;
mod enemies;
mod fx;
mod game;
mod hud;
mod level;
mod render;
mod save;
mod weapons;

use assets::{Fonts, Gfx, Materials};
use audio::{Audio, Track};
use game::{Game, Input};
use hud::{Ui, View};
use level::{VH, VW, WORLD_W};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use save::SaveData;
use weapons::NUM_WEAPONS;

fn window_conf() -> Conf {
    let save = SaveData::load();
    Conf {
        window_title: "RIFT ATTACK: Void Marines".to_owned(),
        window_width: 1280,
        window_height: 720,
        fullscreen: save.fullscreen,
        window_resizable: true,
        high_dpi: false,
        sample_count: 1,
        icon: assets::window_icon(),
        ..Default::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Title,
    Manual(usize),
    Playing,
    Paused,
    GameOver,
}

const EPITAPHS: [&str; 8] = [
    "The rift remembers every name it swallows.",
    "Squad status: one. Then none.",
    "You held the line. The line did not hold you.",
    "Something up there learned your face.",
    "The sky closed over the LZ like an eyelid.",
    "Dog tags recovered: 0.",
    "No extraction was ever scheduled.",
    "They will sing about you, in a language with no mouths.",
];

// ---------------------------------------------------------------------------
// Autoplay bot - drives the title-screen demo and automated testing
// ---------------------------------------------------------------------------
struct Bot {
    target_x: f32,
    retarget: f32,
    jump_t: f32,
    switch_t: f32,
    crouch: f32,
    want_hyper: bool,
}

impl Bot {
    fn new() -> Bot {
        Bot { target_x: WORLD_W * 0.5, retarget: 0.0, jump_t: 1.0, switch_t: 3.0, crouch: 0.0, want_hyper: false }
    }

    fn input(&mut self, g: &Game, dt: f32) -> Input {
        let mut i = Input::default();
        let p = &g.player;
        let me = p.center();
        // target: closest horror, leading slightly
        let mut best: Option<(f32, Vec2)> = None;
        for e in &g.enemies {
            if e.emerge < 0.6 {
                continue;
            }
            let d = e.pos.distance(me);
            if best.map_or(true, |b| d < b.0) {
                best = Some((d, e.pos + e.vel * 0.12));
            }
        }
        i.aim = best.map(|b| b.1).unwrap_or(me + vec2(p.facing * 100.0, -60.0));
        i.fire = best.is_some();
        // movement: chase crates, otherwise keep a fighting distance
        self.retarget -= dt;
        if let Some(c) = g.crates.iter().filter(|c| c.landed).min_by(|a, b| (a.pos.x - me.x).abs().total_cmp(&(b.pos.x - me.x).abs())) {
            self.target_x = c.pos.x;
        } else if self.retarget <= 0.0 {
            self.retarget = gen_range(1.2, 3.0);
            let anchor = best.map(|b| b.1.x).unwrap_or(WORLD_W * 0.5);
            self.target_x = (anchor + gen_range(-180.0, 180.0)).clamp(40.0, WORLD_W - 40.0);
        }
        if (self.target_x - p.pos.x).abs() > 8.0 {
            i.right = self.target_x > p.pos.x;
            i.left = !i.right;
        }
        // jumping & hyperjumps
        self.jump_t -= dt;
        if self.want_hyper {
            i.down = true;
            i.left = false;
            i.right = false;
            self.crouch += dt;
            if self.crouch > 0.45 {
                i.jump_pressed = true;
                i.jump_held = true;
                self.want_hyper = false;
                self.crouch = 0.0;
            }
        } else if self.jump_t <= 0.0 {
            self.jump_t = gen_range(0.6, 2.2);
            if p.on_ground && gen_range(0, 4) == 0 {
                self.want_hyper = true;
            } else {
                i.jump_pressed = true;
            }
        }
        if !p.on_ground && p.vel.y < 0.0 {
            i.jump_held = true;
        }
        if !p.on_ground && p.vel.y > 120.0 && gen_range(0, 30) == 0 {
            i.jump_pressed = true;
        }
        // shuffle the arsenal
        self.switch_t -= dt;
        if self.switch_t <= 0.0 {
            self.switch_t = gen_range(2.5, 5.0);
            let owned: Vec<usize> = (0..NUM_WEAPONS).filter(|w| p.owned[*w]).collect();
            i.select = Some(owned[gen_range(0, owned.len())]);
        }
        i.distort = p.distort > 0.8 && g.eshots.len() > 8;
        i
    }
}

fn read_input(view: &View, cam_x: f32) -> Input {
    let mut i = Input::default();
    i.left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    i.right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
    i.jump_pressed = is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up);
    i.jump_held = is_key_down(KeyCode::W) || is_key_down(KeyCode::Space) || is_key_down(KeyCode::Up);
    i.down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);
    i.fire = is_mouse_button_down(MouseButton::Left);
    i.aim = view.mouse_virtual() + vec2(cam_x, 0.0);
    let wheel = mouse_wheel().1;
    i.next = is_key_pressed(KeyCode::E) || wheel < 0.0;
    i.prev = is_key_pressed(KeyCode::Q) || wheel > 0.0;
    let keys = [
        KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5,
        KeyCode::Key6, KeyCode::Key7, KeyCode::Key8, KeyCode::Key9, KeyCode::Key0,
    ];
    for (n, k) in keys.iter().enumerate() {
        if is_key_pressed(*k) {
            i.select = Some(n);
        }
    }
    i.distort = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    i.suicide = is_key_pressed(KeyCode::K);
    i
}

fn bar(v: f32) -> String {
    let n = (v * 10.0).round() as usize;
    format!("{}{}", "|".repeat(n), ".".repeat(10 - n))
}

fn title_items(s: &SaveData) -> Vec<String> {
    vec![
        "DEPLOY".into(),
        "FIELD MANUAL".into(),
        format!("MUSIC   {}", bar(s.music_vol)),
        format!("SFX     {}", bar(s.sfx_vol)),
        format!("SCREEN FX     {}", if s.screen_fx { "ON" } else { "OFF" }),
        format!("SCREEN SHAKE  {}", if s.shake { "ON" } else { "OFF" }),
        format!("FULLSCREEN    {}", if s.fullscreen { "ON" } else { "OFF" }),
        "QUIT".into(),
    ]
}

fn env_num(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|v| v.parse().ok())
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand((macroquad::miniquad::date::now() * 1000.0) as u64);
    let mut save = SaveData::load();

    // ---- debug / test hooks (environment variables) ----------------------
    let shot_dir = std::env::var("RIFT_SHOTS").ok();
    let shot_every = env_num("RIFT_SHOT_EVERY").unwrap_or(120);
    let max_frames = env_num("RIFT_FRAMES");
    let autoplay = std::env::var("RIFT_AUTOPLAY").is_ok();
    let start_state = std::env::var("RIFT_STATE").unwrap_or_default();

    // ---- loading ---------------------------------------------------------
    let fonts = Fonts::load();
    let gfx = Gfx::load();
    let mats = Materials::load();
    let rt = render_target(VW as u32, VH as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut audio = Audio::new(save.music_vol, save.sfx_vol);
    for step in 0..6 {
        let view = View::compute();
        clear_background(BLACK);
        set_camera(&view.hud_camera());
        let ui = Ui { v: &view, f: &fonts };
        let msg = ["CALIBRATING TRACKER", "WARMING THE CARBINE", "TUNING INTO THE RIFT", "THE CHOIR IS ASSEMBLING", "SOMETHING IS LISTENING", "DEPLOYING"][step];
        ui.center(msg, VW * 0.5, VH * 0.5, 9.0, hud::GREEN, true);
        draw_rectangle(VW * 0.5 - 80.0, VH * 0.5 + 10.0, 160.0 * (step as f32 + 1.0) / 6.0, 3.0, hud::GREEN);
        next_frame().await;
        match step {
            0 => audio.load_sfx().await,
            1..=4 => audio.load_track(step - 1).await,
            _ => {}
        }
    }

    let mut screen = Screen::Title;
    let mut sel = 0usize;
    let mut game = Game::new();
    game.fx.shake_enabled = save.shake;
    let mut demo = Game::new();
    demo.banner = None;
    let mut demo_audio = Audio::new(0.0, 0.0);
    demo_audio.enabled = false;
    let mut bot = Bot::new();
    let mut demo_bot = Bot::new();
    let mut pause_sel = 0usize;
    let mut over_t = 0.0f32;
    let mut new_record = false;
    let mut epitaph = EPITAPHS[0];
    let mut t = 0.0f32;
    let mut frame: u64 = 0;
    let mut music_backup = save.music_vol;
    let mut manual_from_pause = false;

    match start_state.as_str() {
        "play" => screen = Screen::Playing,
        "manual0" => screen = Screen::Manual(0),
        "manual1" => screen = Screen::Manual(1),
        "manual2" => screen = Screen::Manual(2),
        _ => {}
    }
    audio.music(if screen == Screen::Playing { Track::Ichor } else { Track::Hymn });

    loop {
        let dt = get_frame_time().min(1.0 / 30.0);
        t += dt;
        frame += 1;
        let view = View::compute();

        // ---- global keys -------------------------------------------------
        if is_key_pressed(KeyCode::F11) || (is_key_pressed(KeyCode::Enter) && is_key_down(KeyCode::LeftAlt)) {
            save.fullscreen = !save.fullscreen;
            set_fullscreen(save.fullscreen);
            save.store();
        }
        if is_key_pressed(KeyCode::M) {
            if save.music_vol > 0.0 {
                music_backup = save.music_vol;
                save.music_vol = 0.0;
            } else {
                save.music_vol = if music_backup > 0.0 { music_backup } else { 0.7 };
            }
            audio.music_vol = save.music_vol;
            save.store();
        }

        // ---- update ------------------------------------------------------
        match screen {
            Screen::Title | Screen::Manual(_) => {
                let di = demo_bot.input(&demo, dt);
                demo.update(dt, &di, &mut demo_audio);
                if demo.over_t > 3.0 {
                    demo = Game::new();
                    demo.banner = None;
                }
                if let Screen::Manual(page) = screen {
                    if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Right) {
                        screen = if manual_from_pause { Screen::Paused } else { Screen::Title };
                        audio.play("ui_move", 0.5);
                    }
                    if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                        screen = Screen::Manual((page + 1) % 3);
                        audio.play("ui_move", 0.5);
                    }
                    if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                        screen = Screen::Manual((page + 2) % 3);
                        audio.play("ui_move", 0.5);
                    }
                } else {
                    let items = title_items(&save);
                    let n = items.len();
                    if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                        sel = (sel + n - 1) % n;
                        audio.play("ui_move", 0.5);
                    }
                    if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                        sel = (sel + 1) % n;
                        audio.play("ui_move", 0.5);
                    }
                    // mouse hover
                    let mv = view.mouse_virtual();
                    let mut clicked = false;
                    let md = mouse_delta_position();
                    for k in 0..n {
                        let y = 175.0 + k as f32 * 17.0;
                        if mv.y > y - 11.0 && mv.y < y + 4.0 && (mv.x - VW * 0.5).abs() < 90.0 {
                            if md.length() > 0.0 && sel != k {
                                sel = k;
                                audio.play("ui_move", 0.3);
                            }
                            if is_mouse_button_pressed(MouseButton::Left) {
                                sel = k;
                                clicked = true;
                            }
                        }
                    }
                    let left = is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left);
                    let right = is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right);
                    let enter = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || clicked;
                    let adj = if right || (enter && sel >= 2 && sel <= 3) { 0.1 } else if left { -0.1 } else { 0.0 };
                    match sel {
                        0 if enter => {
                            audio.play("ui_select", 0.7);
                            game = Game::new();
                            game.fx.shake_enabled = save.shake;
                            bot = Bot::new();
                            screen = Screen::Playing;
                            audio.music(Track::Ichor);
                        }
                        1 if enter => {
                            audio.play("ui_select", 0.7);
                            manual_from_pause = false;
                            screen = Screen::Manual(0);
                        }
                        2 | 3 if adj != 0.0 => {
                            let v = if sel == 2 { &mut save.music_vol } else { &mut save.sfx_vol };
                            let mut nv = ((*v + adj) * 10.0).round() / 10.0;
                            if nv > 1.01 {
                                nv = 0.0;
                            }
                            *v = nv.clamp(0.0, 1.0);
                            audio.music_vol = save.music_vol;
                            audio.sfx_vol = save.sfx_vol;
                            audio.play("ui_move", 0.7);
                            save.store();
                        }
                        4 if enter || left || right => {
                            save.screen_fx = !save.screen_fx;
                            save.store();
                            audio.play("ui_move", 0.5);
                        }
                        5 if enter || left || right => {
                            save.shake = !save.shake;
                            game.fx.shake_enabled = save.shake;
                            save.store();
                            audio.play("ui_move", 0.5);
                        }
                        6 if enter || left || right => {
                            save.fullscreen = !save.fullscreen;
                            set_fullscreen(save.fullscreen);
                            save.store();
                        }
                        7 if enter => break,
                        _ => {}
                    }
                    if is_key_pressed(KeyCode::Escape) && sel != n - 1 {
                        sel = n - 1;
                    }
                }
            }
            Screen::Playing => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    screen = Screen::Paused;
                    pause_sel = 0;
                    audio.play("ui_move", 0.5);
                } else {
                    let input = if autoplay { bot.input(&game, dt) } else { read_input(&view, game.cam_x) };
                    game.update(dt, &input, &mut audio);
                    if game.player.alive && !game.boss_alive() && !game.boss_pending {
                        audio.music(if game.threat >= 6 { Track::Static } else { Track::Ichor });
                    }
                    if game.over_t > 2.6 {
                        screen = Screen::GameOver;
                        over_t = 0.0;
                        epitaph = EPITAPHS[gen_range(0, EPITAPHS.len())];
                        new_record = game.kills > save.best_kills || game.score > save.best_score;
                        save.best_kills = save.best_kills.max(game.kills);
                        save.best_score = save.best_score.max(game.score);
                        save.best_time = save.best_time.max(game.time);
                        save.store();
                    }
                }
            }
            Screen::Paused => {
                let items = ["RESUME", "FIELD MANUAL", "ABANDON POST"];
                if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                    pause_sel = (pause_sel + 2) % 3;
                }
                if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                    pause_sel = (pause_sel + 1) % 3;
                }
                let mv = view.mouse_virtual();
                let mut clicked = false;
                for k in 0..3 {
                    let y = 160.0 + k as f32 * 17.0;
                    if mv.y > y - 11.0 && mv.y < y + 4.0 && (mv.x - VW * 0.5).abs() < 80.0 {
                        if mouse_delta_position().length() > 0.0 {
                            pause_sel = k;
                        }
                        if is_mouse_button_pressed(MouseButton::Left) {
                            pause_sel = k;
                            clicked = true;
                        }
                    }
                }
                let _ = items;
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    screen = Screen::Playing;
                } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || clicked {
                    audio.play("ui_select", 0.6);
                    match pause_sel {
                        0 => screen = Screen::Playing,
                        1 => {
                            manual_from_pause = true;
                            screen = Screen::Manual(0);
                        }
                        _ => {
                            screen = Screen::Title;
                            audio.music(Track::Hymn);
                        }
                    }
                }
            }
            Screen::GameOver => {
                over_t += dt;
                game.update(dt, &Input { aim: game.player.center(), ..Default::default() }, &mut audio);
                if over_t > 1.2 {
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) || (autoplay && over_t > 4.0) {
                        game = Game::new();
                        game.fx.shake_enabled = save.shake;
                        bot = Bot::new();
                        screen = Screen::Playing;
                        audio.play("ui_select", 0.7);
                        audio.music(Track::Ichor);
                    } else if is_key_pressed(KeyCode::Escape) {
                        screen = Screen::Title;
                        audio.music(Track::Hymn);
                    }
                }
            }
        }
        audio.update(dt);

        // ---- render: world -> low-res target -----------------------------
        let in_menu = matches!(screen, Screen::Title | Screen::Manual(_));
        let showing_demo = matches!(screen, Screen::Title) || (matches!(screen, Screen::Manual(_)) && !manual_from_pause);
        let world = if showing_demo { &demo } else { &game };

        let mut bg_cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VW, VH));
        bg_cam.render_target = Some(rt.clone());
        set_camera(&bg_cam);
        clear_background(BLACK);
        world.draw_backdrop(&gfx);
        let shake = world.fx.shake_offset(t);
        let mut wcam = Camera2D::from_display_rect(Rect::new(world.cam_x.round() + shake.x.round(), shake.y.round(), VW, VH));
        wcam.render_target = Some(rt.clone());
        set_camera(&wcam);
        world.draw_world(&gfx, &mats);
        set_camera(&bg_cam);
        world.draw_edge_markers();
        if world.fx.flash > 0.0 {
            let c = world.fx.flash_col;
            draw_rectangle(0.0, 0.0, VW, VH, Color::new(c.r, c.g, c.b, world.fx.flash * 0.45));
        }
        if in_menu {
            draw_rectangle(0.0, 0.0, VW, VH, Color::new(0.02, 0.0, 0.04, 0.55));
        }

        // ---- post-process to screen -------------------------------------
        set_default_camera();
        clear_background(BLACK);
        let low_hp = if world.player.alive { (1.0 - world.player.hp / 50.0).clamp(0.0, 1.0) } else { 1.0 };
        let fx_on = save.screen_fx;
        mats.post.set_uniform("time", t);
        mats.post.set_uniform("aberr", if fx_on { world.fx.trauma * 1.5 + low_hp * 0.6 + world.fx.flash * 0.8 } else { 0.0 });
        mats.post.set_uniform("invert", if world.power(weapons::Power::VoidSight) { 1.0f32 } else { 0.0 });
        mats.post.set_uniform("rift", if world.power(weapons::Power::TimeRift) || world.distorting { 1.0f32 } else { 0.0 });
        mats.post.set_uniform("warp", if fx_on { low_hp * 0.8 + if world.distorting { 0.6 } else { 0.0 } } else { 0.0 });
        mats.post.set_uniform("grain", if fx_on { 0.06f32 } else { 0.0 });
        mats.post.set_uniform("redout", if !world.player.alive { (world.over_t * 0.6).min(1.0) } else { 0.0 });
        mats.post.set_uniform("res", vec2(VW, VH));
        gl_use_material(&mats.post);
        draw_texture_ex(&rt.texture, view.ox, view.oy, WHITE, DrawTextureParams {
            dest_size: Some(vec2(VW * view.scale, VH * view.scale)),
            // render targets come back bottom-up
            flip_y: true,
            ..Default::default()
        });
        gl_use_default_material();

        // ---- HUD / menus -------------------------------------------------
        set_camera(&view.hud_camera());
        let ui = Ui { v: &view, f: &fonts };
        match screen {
            Screen::Title => {
                hud::draw_title(&ui, &gfx, t, &title_items(&save), sel, save.best_kills, save.best_score);
                hud::crosshair(view.mouse_virtual(), t, hud::GREEN);
            }
            Screen::Manual(p) => hud::draw_manual(&ui, &gfx, p),
            Screen::Playing => {
                hud::draw_hud(&ui, &gfx, &game, save.best_kills);
                let c = if game.power(weapons::Power::VoidSight) { Color::new(1.0, 0.3, 0.6, 1.0) } else { hud::GREEN };
                let aim = if autoplay { game.player.shoulder() + game.player.aim_dir * 80.0 - vec2(game.cam_x, 0.0) } else { view.mouse_virtual() };
                hud::crosshair(aim, t, c);
            }
            Screen::Paused => {
                hud::draw_hud(&ui, &gfx, &game, save.best_kills);
                hud::draw_pause(&ui, &["RESUME".into(), "FIELD MANUAL".into(), "ABANDON POST".into()], pause_sel);
                hud::crosshair(view.mouse_virtual(), t, hud::GREEN);
            }
            Screen::GameOver => {
                hud::draw_game_over(&ui, &game, save.best_kills, save.best_score, new_record, over_t, epitaph);
            }
        }
        show_mouse(false);

        // ---- test hooks --------------------------------------------------
        if let Some(dir) = &shot_dir {
            if frame % shot_every == 0 {
                let img = get_screen_data();
                img.export_png(&format!("{}/shot_{:05}.png", dir, frame));
            }
        }
        if let Some(m) = max_frames {
            if frame >= m {
                break;
            }
        }
        next_frame().await;
    }
    save.store();
}
