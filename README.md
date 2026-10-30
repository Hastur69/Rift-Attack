# RIFT ATTACK: Void Marines

*A cosmic-horror reboot of the Heli Attack 2 formula, written in Rust.*

The sky over colony landing pad LZ GRAVEN has split open. One marine is left. Things with too many eyes are coming through. Nobody is coming to get you.

Like squarecircleco's 2003 Flash classic, it's one arena, an endless enemy supply and an ever-growing arsenal: every 3 kills drops a supply pod, medkits fall at 15/30/60/120 kills, your own explosives can't hurt you, and HyperJump kills hand you new guns. The helicopters have been replaced by inter-dimensional horrors, the soldier is a heavily-armoured void marine, and the soundtrack is hard witch house.

## Play

Run **`RiftAttack.exe`** (Windows 10/11, 64-bit). It's a single self-contained file: every sprite, sound and track is embedded, and no installer or runtime is needed. Windows SmartScreen may warn about an unsigned executable; choose *More info → Run anyway*.

| Input | Action |
|---|---|
| A / D | Move |
| W / Space | Jump (press again mid-air to double-jump) |
| Hold S, then Jump | **HyperJump** (kills during one award a weapon) |
| S + Jump on a catwalk | Drop through |
| Mouse | Aim |
| Left button | Fire (hold) |
| Wheel, Q / E | Cycle weapons |
| 1–9, 0 | Select weapon |
| Shift (hold) | **Time Distort** (slows everything; meter recharges) |
| Esc / P | Pause |
| F11 | Fullscreen |
| M | Mute music |
| K | Suicide (a Heli Attack tradition) |

High scores and settings are saved to `%APPDATA%\RiftAttack\save.cfg`.

## What's inside

- **15 weapons**: the 13 from Heli Attack 2, re-themed (Pulse Carbine, Twin Machine Pistols, Riot Shotgun, Flechette Rockets, Grenade Launcher, Breacher RPG, Rocket Launcher, Seeker Pods, Thermal Mines, Incinerator, Rail Lance, Singularity Bomb, Harpoon Cannon), plus the Smart Rifle and the Void Sight's Null Lance.
- **5 power-ups**: Overcharge, Void Sight, Jump Jets, Elder Ward, Time Rift.
- **4 horrors + a boss**: Gazers, Maws (which birth spawnlings), Seraphs, and The Hollow Crown, which descends every ~45 kills.
- A motion tracker, threat levels, a post-processing pass (chromatic aberration, scanlines, grain, reality-warp at low health, colour inversion in Void Sight), and an attract-mode demo on the title screen.
- **Original witch house soundtrack**, synthesized in code:
  - *Rift Hymn* — title (66 BPM)
  - *Ichor Drill* — combat (74 BPM, half-time trap drums, distorted 808s)
  - *Black Static* — boss and threat 7+ (82 BPM)
  - *Elegy for the Squad* — game over

See **DEEP_DIVE.md** for the research on Heli Attack 2 and a one-to-one map of how every mechanic was adapted.

## Folder layout

```
RiftAttack.exe          the game
README.md               this file
DEEP_DIVE.md            Heli Attack 2 research + design mapping
source/
  Cargo.toml, build.rs  Rust project (macroquad 0.4)
  src/                  game source (~4,000 lines)
  assets/gfx            all sprites, backgrounds, logo (PNG)
  assets/sfx            36 sound effects (WAV)
  assets/music          4 witch house tracks (OGG Vorbis)
  assets/fonts          DejaVu Sans Mono (+ licence)
  res/                  Windows icon + version resource
  tools/gen_art.py      regenerates every sprite from code
  tools/gen_audio.py    regenerates every track and sound effect from code
```

## Building from source

Requires Rust 1.80+.

**On Windows** (MSVC or GNU toolchain):

```
cargo build --release
```

The exe lands in `target\release\RiftAttack.exe`. The icon is embedded when `rc.exe` (Visual Studio) or `windres` (MinGW) is on PATH; otherwise the build still succeeds without it.

**Cross-compiling from Linux:**

```
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

**Native Linux build** (for development): `cargo build --release` needs the ALSA dev package (`libasound2-dev`). Use `cargo build --release --no-default-features` for a silent build without it.

**Regenerating the assets** (Python 3 with numpy, scipy and Pillow; ffmpeg with libvorbis):

```
python3 tools/gen_art.py
python3 tools/gen_audio.py            # or: gen_audio.py sfx | hymn | ichor | static | elegy
```

### Debug switches (environment variables)

| Variable | Effect |
|---|---|
| `RIFT_STATE=play` | Skip the title screen |
| `RIFT_AUTOPLAY=1` | A bot plays the game |
| `RIFT_GIVE_ALL=1` | Start with every weapon |
| `RIFT_KILLS=39` | Start at a kill count (39 → the boss arrives on the next kill) |
| `RIFT_MUTE=1` | No audio |
| `RIFT_NOSAVE=1` | Don't read or write the save file |
| `RIFT_SHOTS=dir`, `RIFT_SHOT_EVERY=n`, `RIFT_FRAMES=n` | Screenshot every *n* frames / quit after *n* frames |

## Credits and licences

- Game code, sprites, sound effects and music: original work generated for this project, released under the MIT licence (see `LICENSE`).
- Inspired by **Heli Attack 2** by squarecircleco. (2003). No code, art or audio from the original is used.
- The marine's look is a nod to the armoured space-marine archetype of late-80s sci-fi; names and designs here are original.
- Font: **DejaVu Sans Mono** (Bitstream Vera licence, see `assets/fonts/LICENSE-DejaVu.txt`).
- Engine: [macroquad](https://github.com/not-fl3/macroquad) (MIT / Apache-2.0).
