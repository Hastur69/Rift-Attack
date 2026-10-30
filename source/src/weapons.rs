//! The arsenal. Thirteen weapons mirror Heli Attack 2's lineup one-for-one
//! (re-skinned for the Void Marines), plus the Smart Rifle and the Null Lance
//! that comes with the Void Sight power-up.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shot {
    /// straight bullet: (radius, r, g, b)
    Bullet,
    /// short-lived spread pellet
    Pellet,
    /// explosive rocket; `accel` > 0 accelerates
    Rocket,
    /// RPG: hangs in the air for a moment, then rips forward
    Rpg,
    /// homing rocket
    Seeker,
    /// lobbed, bouncing grenade
    Grenade,
    /// dropped mine that erupts into a pillar of fire
    Mine,
    /// flame jet particle (pierces)
    Flame,
    /// instant beam, passes through everything
    Rail,
    /// slow black-hole bomb, kills anything it touches
    Singularity,
    /// harpoon that drags a horror out of the sky
    Harpoon,
    /// homing bullets
    Smart,
}

/// Supply-pod colour (index into crates.png). Power & Medkit pods are
/// referenced by sprite index only.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrateKind {
    Rockets = 0,
    Grenades = 1,
    Fire = 2,
    Shotguns = 3,
    Pistols = 4,
    Exotic = 5,
    Power = 6,
    Medkit = 7,
}

pub struct WeaponDef {
    pub name: &'static str,
    pub shot: Shot,
    /// ammo per pickup (0 = infinite)
    pub clip: u32,
    pub delay: f32,
    pub damage: f32,
    pub count: u32,
    pub spread: f32,
    pub speed: f32,
    pub splash: f32,
    pub radius: f32,
    pub sound: &'static str,
    pub shake: f32,
    pub crate_kind: CrateKind,
    pub muzzle: f32,
    pub blurb: &'static str,
}

pub const PULSE: usize = 0;
pub const NULL_LANCE: usize = 14;
pub const NUM_WEAPONS: usize = 15;

pub const WEAPONS: [WeaponDef; NUM_WEAPONS] = [
    WeaponDef { name: "PULSE CARBINE", shot: Shot::Bullet, clip: 0, delay: 0.085, damage: 4.0, count: 1, spread: 0.035, speed: 900.0, splash: 0.0, radius: 0.0, sound: "pulse", shake: 0.6, crate_kind: CrateKind::Pistols, muzzle: 36.0,
        blurb: "Standard issue. Infinite caseless rounds. Chatters like a jackhammer." },
    WeaponDef { name: "TWIN MACHINE PISTOLS", shot: Shot::Bullet, clip: 50, delay: 0.09, damage: 3.6, count: 2, spread: 0.07, speed: 880.0, splash: 0.0, radius: 0.0, sound: "pistol", shake: 0.5, crate_kind: CrateKind::Pistols, muzzle: 22.0,
        blurb: "Akimbo. Two rounds per trigger pull." },
    WeaponDef { name: "RIOT SHOTGUN", shot: Shot::Pellet, clip: 14, delay: 0.55, damage: 6.5, count: 6, spread: 0.32, speed: 760.0, splash: 0.0, radius: 0.0, sound: "shotgun", shake: 4.0, crate_kind: CrateKind::Shotguns, muzzle: 34.0,
        blurb: "Six-pellet cone. Get close. Closer." },
    WeaponDef { name: "FLECHETTE ROCKETS", shot: Shot::Rocket, clip: 8, delay: 0.7, damage: 24.0, count: 3, spread: 0.22, speed: 470.0, splash: 16.0, radius: 26.0, sound: "rocket", shake: 3.0, crate_kind: CrateKind::Shotguns, muzzle: 30.0,
        blurb: "Three rockets in a fan." },
    WeaponDef { name: "GRENADE LAUNCHER", shot: Shot::Grenade, clip: 12, delay: 0.5, damage: 30.0, count: 1, spread: 0.0, speed: 470.0, splash: 30.0, radius: 42.0, sound: "grenade", shake: 1.5, crate_kind: CrateKind::Grenades, muzzle: 32.0,
        blurb: "Lobbed and bouncing. Jump before you fire." },
    WeaponDef { name: "BREACHER RPG", shot: Shot::Rpg, clip: 10, delay: 0.65, damage: 34.0, count: 1, spread: 0.0, speed: 1150.0, splash: 26.0, radius: 36.0, sound: "rocket", shake: 3.0, crate_kind: CrateKind::Rockets, muzzle: 38.0,
        blurb: "Hangs for a heartbeat, then tears forward." },
    WeaponDef { name: "ROCKET LAUNCHER", shot: Shot::Rocket, clip: 8, delay: 0.8, damage: 44.0, count: 1, spread: 0.0, speed: 320.0, splash: 30.0, radius: 46.0, sound: "rocket", shake: 3.5, crate_kind: CrateKind::Rockets, muzzle: 38.0,
        blurb: "Slow. Heavy. Lead your target." },
    WeaponDef { name: "SEEKER PODS", shot: Shot::Seeker, clip: 6, delay: 0.7, damage: 44.0, count: 1, spread: 0.0, speed: 360.0, splash: 26.0, radius: 40.0, sound: "rocket", shake: 2.5, crate_kind: CrateKind::Rockets, muzzle: 28.0,
        blurb: "Thermal lock. Point vaguely and pray." },
    WeaponDef { name: "THERMAL MINES", shot: Shot::Mine, clip: 3, delay: 0.8, damage: 46.0, count: 1, spread: 0.0, speed: 120.0, splash: 0.0, radius: 14.0, sound: "mine", shake: 0.0, crate_kind: CrateKind::Fire, muzzle: 14.0,
        blurb: "Drops at your feet. Erupts into a pillar of fire. Burns through cover." },
    WeaponDef { name: "INCINERATOR", shot: Shot::Flame, clip: 150, delay: 0.035, damage: 2.4, count: 1, spread: 0.16, speed: 330.0, splash: 0.0, radius: 0.0, sound: "flame", shake: 0.3, crate_kind: CrateKind::Fire, muzzle: 38.0,
        blurb: "Short range. Devastating if you can get above them." },
    WeaponDef { name: "RAIL LANCE", shot: Shot::Rail, clip: 3, delay: 1.0, damage: 70.0, count: 1, spread: 0.0, speed: 0.0, splash: 0.0, radius: 0.0, sound: "rail", shake: 6.0, crate_kind: CrateKind::Exotic, muzzle: 40.0,
        blurb: "Instant. Pierces flesh, steel and the walls of the world." },
    WeaponDef { name: "SINGULARITY BOMB", shot: Shot::Singularity, clip: 2, delay: 1.2, damage: 5000.0, count: 1, spread: 0.0, speed: 90.0, splash: 5000.0, radius: 78.0, sound: "grenade", shake: 2.0, crate_kind: CrateKind::Exotic, muzzle: 32.0,
        blurb: "Crawls through the air. Erases whatever it touches." },
    WeaponDef { name: "HARPOON CANNON", shot: Shot::Harpoon, clip: 2, delay: 1.6, damage: 400.0, count: 1, spread: 0.0, speed: 820.0, splash: 0.0, radius: 0.0, sound: "harpoon", shake: 2.0, crate_kind: CrateKind::Exotic, muzzle: 38.0,
        blurb: "Hooks a horror and drags it screaming into the deck." },
    WeaponDef { name: "SMART RIFLE", shot: Shot::Smart, clip: 90, delay: 0.07, damage: 4.4, count: 1, spread: 0.05, speed: 760.0, splash: 0.0, radius: 0.0, sound: "smart", shake: 0.6, crate_kind: CrateKind::Pistols, muzzle: 40.0,
        blurb: "Tracking rounds. The rifle does the aiming." },
    WeaponDef { name: "NULL LANCE", shot: Shot::Rail, clip: 0, delay: 0.42, damage: 9999.0, count: 1, spread: 0.0, speed: 0.0, splash: 0.0, radius: 0.0, sound: "rail", shake: 5.0, crate_kind: CrateKind::Exotic, muzzle: 42.0,
        blurb: "Void Sight only. One shot, one unmaking." },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Power {
    Overcharge = 0,
    VoidSight = 1,
    JumpJets = 2,
    ElderWard = 3,
    TimeRift = 4,
}

pub const POWERS: [(Power, &str, &str); 5] = [
    (Power::Overcharge, "OVERCHARGE", "Triple damage"),
    (Power::VoidSight, "VOID SIGHT", "Invisible. Null Lance. The world turns inside out"),
    (Power::JumpJets, "JUMP JETS", "Hold JUMP to fly"),
    (Power::ElderWard, "ELDER WARD", "Nothing can touch you"),
    (Power::TimeRift, "TIME RIFT", "The horrors crawl. You don't"),
];

pub const POWER_TIME: f32 = 12.0;
