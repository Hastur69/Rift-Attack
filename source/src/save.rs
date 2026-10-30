//! Persistent high scores and settings.
//! Windows: %APPDATA%\RiftAttack\save.cfg   Elsewhere: ~/.rift_attack/save.cfg

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct SaveData {
    pub best_kills: u32,
    pub best_score: u64,
    pub best_time: f32,
    pub music_vol: f32,
    pub sfx_vol: f32,
    pub fullscreen: bool,
    pub screen_fx: bool,
    pub shake: bool,
}

impl Default for SaveData {
    fn default() -> Self {
        SaveData {
            best_kills: 0,
            best_score: 0,
            best_time: 0.0,
            music_vol: 0.7,
            sfx_vol: 0.8,
            fullscreen: false,
            screen_fx: true,
            shake: true,
        }
    }
}

fn path() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("RiftAttack").join("save.cfg"))
}

impl SaveData {
    pub fn load() -> SaveData {
        let mut s = SaveData::default();
        if std::env::var("RIFT_NOSAVE").is_ok() {
            return s;
        }
        let Some(p) = path() else { return s };
        let Ok(text) = std::fs::read_to_string(p) else { return s };
        for line in text.lines() {
            let mut it = line.splitn(2, '=');
            let (Some(k), Some(v)) = (it.next(), it.next()) else { continue };
            let v = v.trim();
            match k.trim() {
                "best_kills" => s.best_kills = v.parse().unwrap_or(0),
                "best_score" => s.best_score = v.parse().unwrap_or(0),
                "best_time" => s.best_time = v.parse().unwrap_or(0.0),
                "music_vol" => s.music_vol = v.parse::<f32>().unwrap_or(0.7).clamp(0.0, 1.0),
                "sfx_vol" => s.sfx_vol = v.parse::<f32>().unwrap_or(0.8).clamp(0.0, 1.0),
                "fullscreen" => s.fullscreen = v == "true",
                "screen_fx" => s.screen_fx = v != "false",
                "shake" => s.shake = v != "false",
                _ => {}
            }
        }
        s
    }

    pub fn store(&self) {
        if std::env::var("RIFT_NOSAVE").is_ok() {
            return;
        }
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let text = format!(
            "# RIFT ATTACK save file\nbest_kills={}\nbest_score={}\nbest_time={:.1}\nmusic_vol={:.2}\nsfx_vol={:.2}\nfullscreen={}\nscreen_fx={}\nshake={}\n",
            self.best_kills, self.best_score, self.best_time, self.music_vol, self.sfx_vol, self.fullscreen,
            self.screen_fx, self.shake
        );
        let _ = std::fs::write(p, text);
    }
}
