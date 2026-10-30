//! Music + sound effects. With the `audio` cargo feature disabled every call
//! becomes a no-op, so the game still builds on machines without sound libs.

#[cfg(feature = "audio")]
use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Hymn = 0,
    Ichor = 1,
    Static = 2,
    Elegy = 3,
}

pub const SFX_NAMES: &[&str] = &[
    "pulse", "pistol", "shotgun", "rocket", "grenade", "bounce", "explosion", "singularity", "rail", "flame",
    "mine", "firepillar", "harpoon", "smart", "empty", "switch", "pickup", "powerup", "medkit", "hurt",
    "death", "jump", "land", "hyperjump", "tracker", "rift", "gazer", "maw", "seraph", "boss", "hit", "splat",
    "voidbolt", "timerift", "ui_move", "ui_select",
];

#[cfg(feature = "audio")]
macro_rules! sfx_bytes {
    ($($n:literal),*) => { &[ $( include_bytes!(concat!("../assets/sfx/", $n, ".wav")) as &[u8] ),* ] };
}

#[cfg(feature = "audio")]
const SFX_DATA: &[&[u8]] = sfx_bytes!(
    "pulse", "pistol", "shotgun", "rocket", "grenade", "bounce", "explosion", "singularity", "rail", "flame",
    "mine", "firepillar", "harpoon", "smart", "empty", "switch", "pickup", "powerup", "medkit", "hurt",
    "death", "jump", "land", "hyperjump", "tracker", "rift", "gazer", "maw", "seraph", "boss", "hit", "splat",
    "voidbolt", "timerift", "ui_move", "ui_select"
);

#[cfg(feature = "audio")]
const MUSIC_DATA: [&[u8]; 4] = [
    include_bytes!("../assets/music/rift_hymn.ogg"),
    include_bytes!("../assets/music/ichor_drill.ogg"),
    include_bytes!("../assets/music/black_static.ogg"),
    include_bytes!("../assets/music/elegy.ogg"),
];

pub struct Audio {
    #[cfg(feature = "audio")]
    sfx: Vec<Sound>,
    #[cfg(feature = "audio")]
    music: Vec<Option<Sound>>,
    last_played: Vec<f64>,
    pub current: Option<Track>,
    fading: Option<(Track, f32)>,
    fade_in: f32,
    pub music_vol: f32,
    pub sfx_vol: f32,
    pub enabled: bool,
}

impl Audio {
    pub fn new(music_vol: f32, sfx_vol: f32) -> Audio {
        Audio {
            #[cfg(feature = "audio")]
            sfx: Vec::new(),
            #[cfg(feature = "audio")]
            music: vec![None, None, None, None],
            last_played: vec![-1.0; SFX_NAMES.len()],
            current: None,
            fading: None,
            fade_in: 1.0,
            music_vol,
            sfx_vol,
            enabled: std::env::var("RIFT_MUTE").is_err(),
        }
    }

    /// Loads all short sound effects.
    pub async fn load_sfx(&mut self) {
        #[cfg(feature = "audio")]
        {
            if !self.enabled {
                return;
            }
            for bytes in SFX_DATA {
                match load_sound_from_bytes(bytes).await {
                    Ok(s) => self.sfx.push(s),
                    Err(_) => {
                        self.enabled = false;
                        return;
                    }
                }
            }
        }
    }

    /// Decodes one music track. Called between frames from the loading screen.
    pub async fn load_track(&mut self, _idx: usize) {
        #[cfg(feature = "audio")]
        {
            if !self.enabled {
                return;
            }
            if let Ok(s) = load_sound_from_bytes(MUSIC_DATA[_idx]).await {
                self.music[_idx] = Some(s);
            }
        }
    }

    fn index(name: &str) -> Option<usize> {
        SFX_NAMES.iter().position(|n| *n == name)
    }

    pub fn play(&mut self, name: &str, vol: f32) {
        self.play_throttled(name, vol, 0.035);
    }

    /// Plays a sound unless the same sound started less than `min_gap` seconds ago.
    pub fn play_throttled(&mut self, name: &str, vol: f32, min_gap: f64) {
        let Some(i) = Self::index(name) else { return };
        let now = macroquad::time::get_time();
        if now - self.last_played[i] < min_gap {
            return;
        }
        self.last_played[i] = now;
        #[cfg(feature = "audio")]
        {
            if !self.enabled || self.sfx_vol <= 0.0 {
                return;
            }
            if let Some(s) = self.sfx.get(i) {
                play_sound(s, PlaySoundParams { looped: false, volume: (vol * self.sfx_vol).clamp(0.0, 1.0) });
            }
        }
        let _ = vol;
    }

    /// Switches music, fading the previous track out.
    pub fn music(&mut self, t: Track) {
        if self.current == Some(t) {
            return;
        }
        if let Some(old) = self.current {
            self.fading = Some((old, 1.0));
        }
        self.current = Some(t);
        self.fade_in = 0.0;
        #[cfg(feature = "audio")]
        {
            if !self.enabled {
                return;
            }
            if let Some(Some(s)) = self.music.get(t as usize) {
                play_sound(s, PlaySoundParams { looped: true, volume: 0.0 });
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some((old, v)) = self.fading {
            let nv = v - dt * 1.2;
            #[cfg(feature = "audio")]
            if let Some(Some(s)) = self.music.get(old as usize) {
                if nv <= 0.0 {
                    stop_sound(s);
                } else {
                    set_sound_volume(s, nv * self.music_vol);
                }
            }
            self.fading = if nv <= 0.0 { None } else { Some((old, nv)) };
            let _ = old;
        }
        self.fade_in = (self.fade_in + dt * 0.8).min(1.0);
        #[cfg(feature = "audio")]
        if let Some(t) = self.current {
            if let Some(Some(s)) = self.music.get(t as usize) {
                set_sound_volume(s, self.fade_in * self.music_vol);
            }
        }
    }
}
