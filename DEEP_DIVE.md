# Heli Attack 2 — A Deep Dive

*Background research for RIFT ATTACK: Void Marines*

## What it was

Heli Attack 2 is a 2003 Flash run-and-gun by **squarecircleco.** (styled lowercase with a trailing period; the developer goes by "iopred" on GitHub). You play a lone soldier in a jungle clearing, and the entire game is one question: how many helicopters can you bring down before they bring you down? There is no story beyond the premise that you were shot down behind enemy lines, one level, and an endless supply of "helis".

It was a sequel to the original **Heli Attack** (2001), which had a single heli type, no sound apart from the studio sting, and only five weapons. The author has said Heli Attack 1 was never broadly released — it only ever lived as a link on his own site, and its source has been lost.

## Why it mattered

Heli Attack 2 was originally meant as a polish pass on the first game and turned into a major leap: proper art, a soundtrack and sound effects, thirteen weapons, and time-bending power-ups. It spread across Miniclip, Newgrounds and every school-computer "unblocked games" mirror of the 2000s, and it picked up two contemporary awards:

- **Miniclip Game of the Year 2003**
- **Flashkit.com Members' Choice — Most Addictive Game of 2003**

Its popularity also earned it a port to **Symbian** phones.

The sequel, **Heli Attack 3** (June 17, 2005), expanded to 16 levels, 32 weapons and new enemy types — moving away from pure helicopters — but HA2 remains the one people remember as *the* Heli Attack: a single screen of escalating chaos with perfect pick-up-and-play controls.

## How it played

**Controls.** Keyboard to move and jump, mouse to aim and fire, number keys or the wheel to swap weapons. Double-jump in mid-air. K is a suicide key — famously the *only* way your own weapons can kill you.

**The loop.**
- Helis arrive endlessly, one type, ramping in number and aggression. Reaching score thresholds triggers a "level up" that raises difficulty rather than your power.
- **Every 3 helis destroyed** drops a weapon or extra.
- **Health packs** appear at 15, 30, 60, 120 kills, doubling each time.
- **Explosives never hurt you**, but they do push you — a rule that turns rockets into movement tools.
- **HyperJump**: a much higher jump. Killing a heli during one awards a random weapon (at one-fifth of the normal ammo). Shooting a heli while you're above it pushes you upward, so skilled players could juggle themselves in the air over a target.

**Supply crates** were colour-coded by family:

| Crate | Contents |
|---|---|
| Yellow | Rocket weapons |
| Purple | Grenade weapons |
| Red | Fire weapons |
| Green | Shotgun weapons (incl. shotgun rockets) |
| Blue | MAC-10s |
| Grey | Grapple cannon, railgun or A-bomb |
| White, yellow arrow | Power-up |
| White, red cross | Health |
| Cycling | Random — changes as it falls |

**The arsenal** (13 weapons, kills-per-heli as fans documented them):

| Weapon | Notes |
|---|---|
| Machine gun / SMG | Infinite ammo, rapid fire, ~30 hits per heli |
| Akimbo MAC-10s | Two bullets per shot, 50 per pickup |
| Shotgun | 5-pellet cone, ~4 shots per kill, 14 per pickup |
| Shotgun Rockets | 3 rockets in a fan, slightly faster than normal rockets, 8 per pickup |
| Grenade Launcher | Parabolic — can't reach helis from the ground, 12 per pickup |
| RPG | Hovers for a moment, then accelerates very fast, 10 per pickup |
| Rocket Launcher | Slow and heavy, 3 per kill, 8 per pickup |
| Seeker Launcher | Homing, nearly guaranteed hits, 6 per pickup |
| Firemines | Dropped at your feet, erupt into a ~3 s pillar of fire that burns through cover |
| Flamethrower | 150 per pickup, almost no reload; can kill in a single well-timed HyperJump |
| Railgun | Instant beam that passes through cover, 3 per pickup |
| A-Bomb | One-shot kill, but crawls through the air — players HyperJumped to slam it into helis |
| Grapple Cannon | Hooks a heli and drags it down for an instant kill; longest reload in the game |

**Power-ups** (timed): **TriDamage** (triple damage), **Predator Mode** (invisible until hit, a one-shot-kill rail weapon, and an inverted-colour screen), **JetPack** (flight), **Invulnerability**, and **TimeRift** (everything slows except your movement and reloads). Fan wikis also describe a player **TimeDistort** ability that slows everything including you.

## Legacy and preservation

When Flash reached end-of-life in 2020, Heli Attack survived through the Ruffle emulator and fan archives. The author later published the **source code and assets for Heli Attack 2 & 3 on GitHub** (`iopred/heliattack`, GPL-3.0), crediting **Chris "DayDream" Hildenbrand** for the series' art and **David "Psykopig" Desi** for the Heli Attack 3 music.

## How RIFT ATTACK maps onto it

RIFT ATTACK borrows the *rules*, not the assets: every sprite, sound and note in this project was generated from scratch by the scripts in `tools/`.

| Heli Attack 2 | RIFT ATTACK: Void Marines |
|---|---|
| Downed soldier in a jungle | Lone Void Marine on colony landing pad LZ GRAVEN, under a torn sky |
| Helicopters | Things from the rift: **Gazer** (the "heli"), **Maw**, **Seraph**, and the boss **The Hollow Crown** |
| Machine gun (∞) | Pulse Carbine (∞) |
| Akimbo MAC-10s | Twin Machine Pistols |
| Shotgun | Riot Shotgun |
| Shotgun Rockets | Flechette Rockets |
| Grenade Launcher | Grenade Launcher |
| RPG | Breacher RPG (same hover-then-rip behaviour) |
| Rocket Launcher | Rocket Launcher |
| Seeker Launcher | Seeker Pods |
| Firemines | Thermal Mines |
| Flamethrower | Incinerator |
| Railgun | Rail Lance |
| A-Bomb | Singularity Bomb |
| Grapple Cannon | Harpoon Cannon |
| — | **Smart Rifle** (new: tracking rounds) |
| TriDamage | Overcharge |
| Predator Mode | Void Sight (invisible, Null Lance, inverted colours) |
| JetPack | Jump Jets |
| Invulnerability | Elder Ward |
| TimeRift | Time Rift |
| TimeDistort | Time Distort (SHIFT, rechargeable meter) |
| HyperJump | Hold S to charge, then jump; aerial kills award weapons at 1/5 ammo |
| Crate every 3 kills | Supply pod every 3 kills, same colour families |
| Health at 15/30/60/120 | Medkits at 15/30/60/120... |
| Explosions push but never hurt | Same |
| K = suicide | Same |

New in the reboot: three extra horror types with distinct attack patterns, a recurring boss with an eye-beam / rain / summon cycle, a motion tracker, threat levels, a post-processing pass (chromatic aberration, scanlines, warp at low health), and an original procedurally-synthesized witch house soundtrack.

## Sources

- [Heli Attack 2 — StrategyWiki](https://strategywiki.org/wiki/Heli_Attack_2)
- [Heli Attack 2/Weapons — StrategyWiki](https://strategywiki.org/wiki/Heli_Attack_2/Weapons)
- [Heli Attack 2 — Wikipedia (2007 archive)](https://wikipedia2007.classicistranieri.com/h/e/l/Heli_Attack_2_d0c8.html)
- [Heli Attack (series) — Wikipedia (2006 archive)](https://wikipedia2006.classicistranieri.com/en/h/e/l/Heli_Attack_%28series%29_cae5.html)
- [Heli Attack 3 — StrategyWiki](https://strategywiki.org/wiki/Heli_Attack_3)
- [Heli Attack 3 — EverybodyWiki](https://en.everybodywiki.com/Heli_Attack_3)
- [iopred/heliattack — GitHub](https://github.com/iopred/heliattack)
- [Heli Attack 2 — MobyGames](https://www.mobygames.com/game/heli-attack-2)
- [Heli Attack 1 — Kotaku game page](https://kotaku.com/games/heli-attack)
