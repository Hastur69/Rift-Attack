#!/usr/bin/env python3
"""
RIFT ATTACK - procedural art generator.

Every sprite, background and icon in assets/gfx is produced by this script,
so the whole look of the game can be re-rolled or tweaked from code.

    python3 tools/gen_art.py            (writes into ./assets/gfx and ./res)

Requires: numpy, Pillow
"""
import math
import os
import random
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont, ImageChops

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
GFX = os.path.join(ROOT, "assets", "gfx")
RES = os.path.join(ROOT, "res")
os.makedirs(GFX, exist_ok=True)
os.makedirs(RES, exist_ok=True)

FONT_BOLD = os.path.join(ROOT, "assets", "fonts", "DejaVuSansMono-Bold.ttf")

rng = random.Random(1986)
np.random.seed(1986)


def hexc(h, a=255):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


def save(img, name):
    path = os.path.join(GFX, name)
    img.save(path)
    print("  wrote", os.path.relpath(path, ROOT), img.size)


# --------------------------------------------------------------------------
# Value noise helpers
# --------------------------------------------------------------------------
def value_noise(w, h, scale, seed=0, tile_x=False):
    r = np.random.RandomState(seed)
    gw = int(math.ceil(w / scale)) + 2
    gh = int(math.ceil(h / scale)) + 2
    grid = r.rand(gh, gw)
    if tile_x:
        n = int(round(w / scale))
        grid[:, n:] = grid[:, : gw - n]
    ys = np.arange(h) / scale
    xs = np.arange(w) / scale
    x0 = np.floor(xs).astype(int)
    y0 = np.floor(ys).astype(int)
    fx = xs - x0
    fy = ys - y0
    fx = fx * fx * (3 - 2 * fx)
    fy = fy * fy * (3 - 2 * fy)
    a = grid[y0[:, None], x0[None, :]]
    b = grid[y0[:, None], x0[None, :] + 1]
    c = grid[y0[:, None] + 1, x0[None, :]]
    d = grid[y0[:, None] + 1, x0[None, :] + 1]
    top = a + (b - a) * fx[None, :]
    bot = c + (d - c) * fx[None, :]
    return top + (bot - top) * fy[:, None]


def fbm(w, h, scale, octaves=5, seed=0, tile_x=False):
    out = np.zeros((h, w))
    amp = 1.0
    tot = 0.0
    for o in range(octaves):
        out += amp * value_noise(w, h, max(scale / (2 ** o), 1.0), seed + o * 17, tile_x)
        tot += amp
        amp *= 0.5
    return out / tot


# --------------------------------------------------------------------------
# THE MARINE
#   Body sheet: frames 32x40, feet at y=39, hips at (16,27).
#   Shoulder pivot (for the weapon arm) at (16,18) in every frame.
# --------------------------------------------------------------------------
OLIVE = hexc("#4f5a24")
OLIVE_D = hexc("#323a16")
OLIVE_L = hexc("#7a8a3a")
CAMO_A = hexc("#55583a")
CAMO_B = hexc("#3d4029")
CAMO_C = hexc("#6b6b45")
BOOT = hexc("#17150f")
BOOT_L = hexc("#35302a")
SKIN = hexc("#c4946b")
SKIN_D = hexc("#8e6545")
WEB = hexc("#262614")
METAL = hexc("#6f7570")
METAL_D = hexc("#3b3f3d")
LAMP = hexc("#ffe58a")
VISOR = hexc("#141a12")

FW, FH = 32, 40
HIP = (16, 27)


def thick_line(d, p0, p1, w, col):
    d.line([p0, p1], fill=col, width=w)
    r = w / 2.0 - 0.5
    for (x, y) in (p0, p1):
        d.ellipse([x - r, y - r, x + r, y + r], fill=col)


def leg(d, hip, thigh_deg, knee_deg, front):
    """Draw one leg. angles in degrees from straight-down, + = forward(right)."""
    tl, sl = 7.0, 7.5
    a1 = math.radians(thigh_deg)
    kx = hip[0] + math.sin(a1) * tl
    ky = hip[1] + math.cos(a1) * tl
    a2 = math.radians(thigh_deg - knee_deg)
    fx = kx + math.sin(a2) * sl
    fy = ky + math.cos(a2) * sl
    c1 = CAMO_A if front else CAMO_B
    c2 = CAMO_C if front else CAMO_B
    thick_line(d, hip, (kx, ky), 4, c1)
    thick_line(d, (kx, ky), (fx, fy), 4, c1)
    # knee pad
    d.rectangle([kx - 1.5, ky - 1.5, kx + 1.5, ky + 1.5], fill=OLIVE if front else OLIVE_D)
    # boot
    d.rectangle([fx - 2, fy - 2, fx + 3, fy + 1], fill=BOOT if front else BOOT)
    d.line([(fx - 2, fy + 1), (fx + 3, fy + 1)], fill=BOOT_L if front else BOOT)
    # camo specks
    if front:
        mx, my = (hip[0] + kx) / 2, (hip[1] + ky) / 2
        d.point((mx, my), fill=c2)


def torso(d, hip, lean=0, bob=0, crouch=False):
    hx, hy = hip
    hy += bob
    top = hy - (9 if crouch else 12)
    off = lean
    # back pack / battery
    d.rectangle([hx - 7 + off, top + 2, hx - 4 + off, hy - 3], fill=OLIVE_D)
    d.rectangle([hx - 7 + off, top + 3, hx - 6 + off, top + 6], fill=METAL_D)
    # fatigue under-shirt
    d.rectangle([hx - 4 + off, top + 1, hx + 4 + off, hy - 1], fill=CAMO_B)
    # chest armor plate
    d.rectangle([hx - 4 + off, top + 1, hx + 4 + off, top + 8], fill=OLIVE)
    d.line([(hx - 4 + off, top + 1), (hx + 4 + off, top + 1)], fill=OLIVE_L)
    d.line([(hx + 4 + off, top + 1), (hx + 4 + off, top + 8)], fill=OLIVE_L)
    d.line([(hx - 4 + off, top + 8), (hx + 4 + off, top + 8)], fill=OLIVE_D)
    # little stencilled squad marking
    d.point((hx + 2 + off, top + 4), fill=hexc("#c9c08a"))
    d.point((hx + 3 + off, top + 4), fill=hexc("#c9c08a"))
    # shoulder pad
    d.rectangle([hx - 1 + off, top - 0, hx + 3 + off, top + 3], fill=OLIVE_L)
    d.point((hx + 3 + off, top + 3), fill=OLIVE)
    # webbing belt + pouches
    d.rectangle([hx - 4 + off, hy - 3, hx + 4 + off, hy - 1], fill=WEB)
    d.rectangle([hx + 1 + off, hy - 4, hx + 3 + off, hy - 2], fill=OLIVE_D)
    d.rectangle([hx - 3 + off, hy - 4, hx - 1 + off, hy - 2], fill=OLIVE_D)
    # neck + head
    head_y = top - 7
    hxo = hx + off
    d.rectangle([hxo - 1, top - 1, hxo + 2, top], fill=SKIN_D)
    # face
    d.rectangle([hxo - 2, head_y + 2, hxo + 3, head_y + 6], fill=SKIN)
    d.point((hxo + 3, head_y + 6), fill=SKIN_D)
    d.line([(hxo - 2, head_y + 6), (hxo + 2, head_y + 6)], fill=SKIN_D)
    # eye shadow / visor strip
    d.line([(hxo + 1, head_y + 3), (hxo + 3, head_y + 3)], fill=VISOR)
    # helmet
    d.rectangle([hxo - 3, head_y - 1, hxo + 3, head_y + 1], fill=OLIVE)
    d.rectangle([hxo - 4, head_y, hxo - 2, head_y + 5], fill=OLIVE)
    d.line([(hxo - 2, head_y - 1), (hxo + 2, head_y - 1)], fill=OLIVE_L)
    d.line([(hxo - 3, head_y + 1), (hxo + 4, head_y + 1)], fill=OLIVE_D)
    # helmet cam + lamp (side mounted)
    d.rectangle([hxo - 5, head_y + 1, hxo - 4, head_y + 3], fill=METAL_D)
    d.point((hxo + 4, head_y + 1), fill=LAMP)
    # chin strap
    d.point((hxo - 1, head_y + 5), fill=WEB)
    return top


def marine_frame(kind, t):
    img = Image.new("RGBA", (FW, FH), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    hip = HIP
    if kind == "idle":
        bob = 1 if t >= 0.5 else 0
        leg(d, (hip[0] - 1, hip[1] + bob), -6, 4, False)
        leg(d, (hip[0] + 1, hip[1] + bob), 8, 6, True)
        torso(d, hip, 0, bob)
    elif kind == "run":
        ph = t * 2 * math.pi
        bob = int(round(abs(math.sin(ph)) * -1.2))
        a = math.sin(ph) * 38
        b = math.sin(ph + math.pi) * 38
        ka = 25 + 30 * max(0, math.sin(ph + 1.7))
        kb = 25 + 30 * max(0, math.sin(ph + math.pi + 1.7))
        leg(d, (hip[0] - 1, hip[1] + bob), b, kb, False)
        leg(d, (hip[0] + 1, hip[1] + bob), a, ka, True)
        torso(d, hip, 1, bob)
    elif kind == "jump":
        leg(d, (hip[0] - 1, hip[1] - 1), -20, 45, False)
        leg(d, (hip[0] + 1, hip[1] - 1), 35, 70, True)
        torso(d, hip, 0, -1)
    elif kind == "fall":
        leg(d, (hip[0] - 1, hip[1]), -12, 10, False)
        leg(d, (hip[0] + 1, hip[1]), 18, 25, True)
        torso(d, hip, 0, 0)
    elif kind == "crouch":
        h2 = (hip[0], hip[1] + 5)
        leg(d, (h2[0] - 1, h2[1]), -60, -95, False)
        leg(d, (h2[0] + 1, h2[1]), 70, 130, True)
        torso(d, h2, 0, 0, crouch=False)
    elif kind == "dead":
        # lying on the deck, drawn rotated afterwards
        leg(d, (hip[0] - 1, hip[1]), -4, 2, False)
        leg(d, (hip[0] + 1, hip[1]), 6, 4, True)
        torso(d, hip, 0, 0)
        img = img.rotate(-90 * min(1.0, t), resample=Image.NEAREST, center=(16, 38))
    return img


def gen_marine():
    frames = []
    frames += [marine_frame("idle", 0.0), marine_frame("idle", 0.6)]           # 0-1
    frames += [marine_frame("run", i / 8.0) for i in range(8)]                 # 2-9
    frames += [marine_frame("jump", 0), marine_frame("fall", 0)]               # 10-11
    frames += [marine_frame("crouch", 0)]                                      # 12
    frames += [marine_frame("dead", i / 3.0) for i in range(4)]                # 13-16
    sheet = Image.new("RGBA", (FW * len(frames), FH), (0, 0, 0, 0))
    for i, f in enumerate(frames):
        sheet.paste(f, (i * FW, 0), f)
    # 1px dark outline improves readability against the busy background
    sheet = outline(sheet, (8, 10, 6, 255))
    save(sheet, "marine.png")


def outline(img, col):
    a = np.array(img)
    alpha = a[:, :, 3] > 0
    grow = np.zeros_like(alpha)
    grow[1:, :] |= alpha[:-1, :]
    grow[:-1, :] |= alpha[1:, :]
    grow[:, 1:] |= alpha[:, :-1]
    grow[:, :-1] |= alpha[:, 1:]
    edge = grow & ~alpha
    a[edge] = col
    return Image.fromarray(a)


# --------------------------------------------------------------------------
# WEAPONS  (each 48x16, pivot = shoulder at (4,8), pointing right)
# --------------------------------------------------------------------------
GW, GH = 48, 16
WEAPON_NAMES = [
    "pulse_carbine", "twin_pistols", "riot_shotgun", "flechette_rockets",
    "grenade_launcher", "breacher_rpg", "rocket_launcher", "seeker_pods",
    "thermal_mines", "incinerator", "rail_lance", "singularity_bomb",
    "harpoon_cannon", "smart_rifle", "null_lance",
]


def arm(d, hand=(12, 9)):
    thick_line(d, (4, 8), hand, 3, OLIVE)
    d.point((5, 7), fill=OLIVE_L)
    d.rectangle([hand[0] - 1, hand[1] - 1, hand[0] + 1, hand[1] + 1], fill=hexc("#2a2a20"))


def gun(name):
    img = Image.new("RGBA", (GW, GH), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    G = hexc("#2b2e2c")
    G2 = hexc("#454a47")
    G3 = hexc("#5f6662")
    if name == "pulse_carbine":
        arm(d)
        d.rectangle([9, 6, 30, 9], fill=G)          # receiver
        d.rectangle([12, 5, 26, 6], fill=G2)        # top cover
        d.rectangle([30, 7, 37, 8], fill=G)         # barrel shroud
        d.rectangle([22, 10, 31, 11], fill=G2)      # underbarrel launcher
        d.rectangle([14, 10, 16, 13], fill=G)       # mag
        d.rectangle([5, 7, 9, 10], fill=G2)         # stock
        d.point((28, 7), fill=hexc("#ff4a2a"))      # ammo counter
        d.point((29, 7), fill=hexc("#ff4a2a"))
        d.line([(31, 7), (36, 7)], fill=G3)
    elif name == "twin_pistols":
        arm(d, (12, 8))
        d.rectangle([10, 6, 20, 8], fill=G)
        d.rectangle([11, 9, 13, 12], fill=G2)
        d.rectangle([12, 9, 22, 10], fill=G2)   # second pistol, offset
        d.rectangle([13, 11, 15, 14], fill=G)
        d.line([(10, 6), (19, 6)], fill=G3)
    elif name == "riot_shotgun":
        arm(d)
        d.rectangle([9, 6, 34, 8], fill=hexc("#3a2f22"))
        d.rectangle([14, 9, 30, 10], fill=G2)       # tube
        d.rectangle([18, 9, 24, 11], fill=hexc("#5a4632"))  # pump
        d.rectangle([4, 7, 9, 10], fill=hexc("#4a3a28"))
        d.line([(10, 6), (34, 6)], fill=G3)
    elif name == "flechette_rockets":
        arm(d)
        d.rectangle([8, 4, 30, 11], fill=hexc("#5c5a2a"))
        for y in (5, 7, 9):
            d.line([(26, y), (31, y)], fill=G)
        d.rectangle([12, 12, 14, 14], fill=G)
        d.line([(8, 4), (30, 4)], fill=hexc("#8c8a44"))
    elif name == "grenade_launcher":
        arm(d)
        d.rectangle([9, 6, 28, 10], fill=hexc("#3b2d4a"))
        d.ellipse([13, 4, 21, 12], fill=hexc("#54406a"))   # drum
        d.ellipse([15, 6, 19, 10], fill=G)
        d.rectangle([28, 6, 33, 9], fill=G)
        d.rectangle([5, 7, 9, 10], fill=G2)
    elif name == "breacher_rpg":
        arm(d)
        d.rectangle([2, 6, 34, 9], fill=hexc("#5a5a22"))
        d.polygon([(34, 4), (40, 7), (40, 8), (34, 11)], fill=hexc("#7a7a2c"))
        d.rectangle([14, 10, 16, 13], fill=G)
        d.line([(2, 6), (34, 6)], fill=hexc("#8c8c44"))
    elif name == "rocket_launcher":
        arm(d)
        d.rectangle([0, 5, 36, 10], fill=hexc("#6b6324"))
        d.rectangle([36, 4, 38, 11], fill=G)
        d.rectangle([0, 4, 2, 11], fill=G)
        d.rectangle([16, 2, 20, 4], fill=G2)  # sight
        d.rectangle([14, 11, 16, 14], fill=G)
        d.line([(0, 5), (36, 5)], fill=hexc("#9a9038"))
    elif name == "seeker_pods":
        arm(d)
        d.rectangle([6, 3, 28, 12], fill=hexc("#6a6224"))
        for (x, y) in ((24, 4), (24, 9), (19, 4), (19, 9)):
            d.ellipse([x, y - 1, x + 3, y + 2], fill=G)
        d.point((10, 5), fill=hexc("#ff3b3b"))
        d.line([(6, 3), (28, 3)], fill=hexc("#9a9038"))
    elif name == "thermal_mines":
        arm(d, (11, 9))
        d.ellipse([10, 5, 20, 13], fill=hexc("#6a2a1a"))
        d.ellipse([13, 7, 17, 11], fill=hexc("#ff7a2a"))
        d.point((15, 9), fill=hexc("#ffee99"))
    elif name == "incinerator":
        arm(d)
        d.rectangle([9, 6, 32, 9], fill=hexc("#5a2a1c"))
        d.rectangle([13, 10, 21, 13], fill=hexc("#7a3a1c"))   # fuel tank
        d.rectangle([32, 5, 36, 10], fill=G)
        d.point((37, 9), fill=hexc("#5ab0ff"))                 # pilot light
        d.line([(9, 6), (32, 6)], fill=hexc("#8a4a2c"))
    elif name == "rail_lance":
        arm(d)
        d.rectangle([9, 5, 40, 6], fill=G2)
        d.rectangle([9, 9, 40, 10], fill=G2)
        d.rectangle([9, 6, 22, 9], fill=G)
        for x in range(24, 40, 3):
            d.line([(x, 7), (x, 8)], fill=hexc("#5affb0"))
        d.rectangle([4, 7, 9, 10], fill=G)
    elif name == "singularity_bomb":
        arm(d)
        d.rectangle([6, 4, 30, 11], fill=hexc("#3a3a40"))
        d.ellipse([24, 3, 34, 12], fill=hexc("#1a1a22"))
        d.ellipse([27, 6, 31, 10], fill=hexc("#b07aff"))
        d.line([(6, 4), (30, 4)], fill=hexc("#6a6a74"))
        d.text((10, 4), "", fill=(0, 0, 0, 0))
    elif name == "harpoon_cannon":
        arm(d)
        d.rectangle([9, 5, 30, 10], fill=hexc("#4a4a4a"))
        d.polygon([(30, 6), (38, 7), (38, 8), (30, 9)], fill=hexc("#c0c0c0"))
        d.ellipse([12, 9, 18, 14], fill=hexc("#7a5a2a"))  # rope spool
        d.line([(9, 5), (30, 5)], fill=hexc("#7a7a7a"))
    elif name == "smart_rifle":
        arm(d)
        d.rectangle([9, 5, 34, 10], fill=hexc("#2f3a3a"))
        d.rectangle([34, 6, 40, 8], fill=G)
        d.rectangle([14, 2, 20, 4], fill=G2)          # tracking unit
        d.point((19, 3), fill=hexc("#ff3030"))
        d.rectangle([4, 7, 9, 11], fill=G)
        d.rectangle([20, 11, 26, 13], fill=G2)
        d.line([(9, 5), (34, 5)], fill=hexc("#4a5a5a"))
    elif name == "null_lance":
        arm(d)
        d.rectangle([9, 5, 42, 10], fill=hexc("#0a0a0a"))
        for x in range(12, 42, 2):
            d.point((x, 7), fill=hexc("#ffffff"))
            d.point((x + 1, 8), fill=hexc("#ff3070"))
    return outline(img, (8, 8, 8, 200))


def gen_weapons():
    sheet = Image.new("RGBA", (GW, GH * len(WEAPON_NAMES)), (0, 0, 0, 0))
    for i, n in enumerate(WEAPON_NAMES):
        g = gun(n)
        sheet.paste(g, (0, i * GH), g)
    save(sheet, "weapons.png")


# --------------------------------------------------------------------------
# HORRORS
# --------------------------------------------------------------------------
def soft_disc(size, cx, cy, r, inner, outer, power=1.0):
    yy, xx = np.mgrid[0:size[1], 0:size[0]]
    d = np.sqrt((xx - cx) ** 2 + (yy - cy) ** 2) / r
    t = np.clip(d, 0, 1) ** power
    col = np.zeros((size[1], size[0], 4))
    for k in range(3):
        col[:, :, k] = inner[k] * (1 - t) + outer[k] * t
    col[:, :, 3] = (d <= 1.0) * 255
    return col


def flesh_texture(w, h, seed, base, dark, vein):
    n = fbm(w, h, 10, 4, seed)
    v = np.abs(fbm(w, h, 14, 3, seed + 5) - 0.5)
    veins = np.clip(1 - v * 18, 0, 1) ** 2
    out = np.zeros((h, w, 3))
    for k in range(3):
        out[:, :, k] = dark[k] + (base[k] - dark[k]) * n
        out[:, :, k] = out[:, :, k] * (1 - veins * 0.7) + vein[k] * veins * 0.7
    return out


def blob_mask(w, h, cx, cy, rx, ry, wobble, seed, phase=0.0):
    yy, xx = np.mgrid[0:h, 0:w]
    ang = np.arctan2(yy - cy, xx - cx)
    r = rng_state = np.random.RandomState(seed)
    harmonics = [(k, r.rand() * wobble / k, r.rand() * 6.28) for k in range(2, 7)]
    rr = np.ones_like(ang)
    for (k, a, p) in harmonics:
        rr += a * np.sin(k * ang + p + phase * (1 if k % 2 else -1))
    d = np.sqrt(((xx - cx) / rx) ** 2 + ((yy - cy) / ry) ** 2)
    return d <= rr, d / rr


def shade(arr, mask, dn, light=(-0.5, -0.7)):
    """arr: HxWx3 float, apply spherical shading from normalized distance dn."""
    h, w = mask.shape
    yy, xx = np.mgrid[0:h, 0:w]
    sh = 1.0 - np.clip(dn, 0, 1) ** 2.5 * 0.75
    return arr * sh[:, :, None]


def tentacle(d, base, length, segs, phase, amp, width0, col, col2, curl=1.0, down=True):
    pts = []
    x, y = base
    ang = math.pi / 2 if down else -math.pi / 2
    for i in range(segs + 1):
        t = i / segs
        pts.append((x, y, t))
        ang += math.sin(phase + t * 5.0) * amp * 0.35 * curl
        step = length / segs
        x += math.cos(ang) * step
        y += math.sin(ang) * step
    for i in range(len(pts) - 1):
        x0, y0, t0 = pts[i]
        x1, y1, _ = pts[i + 1]
        w = max(1, int(round(width0 * (1 - t0) + 0.6)))
        c = col if i % 2 == 0 else col2
        d.line([(x0, y0), (x1, y1)], fill=c, width=w)
        r = w / 2
        d.ellipse([x1 - r, y1 - r, x1 + r, y1 + r], fill=c)
    # suckers
    for i in range(1, len(pts) - 1, 2):
        x0, y0, t0 = pts[i]
        d.point((x0 + 1, y0), fill=hexc("#e7b7c8"))
    return pts[-1]


def np_to_img(arr, mask):
    h, w = mask.shape
    out = np.zeros((h, w, 4), dtype=np.uint8)
    out[:, :, :3] = np.clip(arr, 0, 255).astype(np.uint8)
    out[:, :, 3] = mask.astype(np.uint8) * 255
    return Image.fromarray(out)


def eye(d, cx, cy, r, sclera=hexc("#e8e2c4"), iris=hexc("#b8ff3a"), pupil=True, lid=None):
    d.ellipse([cx - r, cy - r * 0.8, cx + r, cy + r * 0.8], fill=hexc("#3a0f22"))
    d.ellipse([cx - r + 1, cy - r * 0.8 + 1, cx + r - 1, cy + r * 0.8 - 1], fill=sclera)
    ir = r * 0.55
    d.ellipse([cx - ir, cy - ir, cx + ir, cy + ir], fill=iris)
    if pupil:
        d.ellipse([cx - 1, cy - ir * 0.8, cx + 1, cy + ir * 0.8], fill=hexc("#050505"))
    # vein
    d.line([(cx - r + 1, cy), (cx - ir, cy - 1)], fill=hexc("#c0304a"))


def gen_gazer():
    """Watcher-class horror: a floating eye-mass with a crown of hanging tentacles.
    The giant central eye has NO pupil baked in - the game draws a pupil that tracks the player.
    64x72, 8 animation frames."""
    W, H = 64, 72
    frames = []
    for f in range(8):
        ph = f / 8 * 2 * math.pi
        img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        # back tentacles
        for i in range(5):
            bx = 14 + i * 9
            tentacle(d, (bx, 36), 28 + (i % 2) * 6, 9, ph + i * 1.3, 1.2, 4, hexc("#4a1838"), hexc("#3a1230"))
        mask, dn = blob_mask(W, H, 32, 26, 22, 20, 0.10, 11, ph * 0.5)
        tex = flesh_texture(W, H, 3, (150, 52, 108), (54, 14, 46), (220, 70, 80))
        body = np_to_img(shade(tex, mask, dn), mask)
        img.alpha_composite(body)
        d = ImageDraw.Draw(img)
        # front tentacles
        for i in range(4):
            bx = 18 + i * 9
            tentacle(d, (bx, 40), 22 + (i % 2) * 8, 8, ph + i * 2.1 + 1.0, 1.4, 4, hexc("#7a2a5a"), hexc("#622048"))
        # small eyes
        for (ex, ey, er) in ((15, 20, 3.2), (47, 18, 3.6), (22, 9, 2.6), (43, 34, 2.8), (12, 32, 2.2)):
            blink = (f + int(ex)) % 8 == 0
            if blink:
                d.line([(ex - er, ey), (ex + er, ey)], fill=hexc("#2a0a20"), width=2)
            else:
                eye(d, ex, ey, er, iris=hexc("#ff4040"))
        # big central eye socket (pupil drawn in-game)
        d.ellipse([20, 15, 44, 37], fill=hexc("#2a0618"))
        d.ellipse([22, 17, 42, 35], fill=hexc("#f0ead0"))
        d.ellipse([25, 19, 39, 33], fill=hexc("#c8ff40"))
        d.ellipse([27, 21, 37, 31], fill=hexc("#88d020"))
        d.arc([22, 17, 42, 35], 200, 340, fill=hexc("#ffffff"))
        frames.append(outline(img, (20, 4, 16, 255)))
    sheet = Image.new("RGBA", (W * 8, H), (0, 0, 0, 0))
    for i, fr in enumerate(frames):
        sheet.paste(fr, (i * W, 0), fr)
    save(sheet, "gazer.png")


def gen_maw():
    """Maw-class horror: a bloated flying sac split by a vertical toothed mouth. 88x80, 8 frames."""
    W, H = 88, 80
    frames = []
    for f in range(8):
        ph = f / 8 * 2 * math.pi
        open_amt = (math.sin(ph) * 0.5 + 0.5)
        img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        for i in range(7):
            bx = 14 + i * 10
            tentacle(d, (bx, 48), 22 + (i * 7) % 12, 8, ph + i * 0.9, 1.0, 5, hexc("#2d3b1e"), hexc("#243018"))
        mask, dn = blob_mask(W, H, 44, 32, 34, 27, 0.08, 23, ph * 0.3)
        tex = flesh_texture(W, H, 8, (108, 128, 64), (30, 40, 20), (170, 60, 50))
        body = np_to_img(shade(tex, mask, dn), mask)
        img.alpha_composite(body)
        d = ImageDraw.Draw(img)
        # ribs
        for k in range(-3, 4):
            if k == 0:
                continue
            x = 44 + k * 8
            d.arc([x - 6, 10, x + 6, 54], 260 if k < 0 else 280, 100 if k < 0 else 80, fill=hexc("#cfc7a0"))
        # the mouth
        mw = 4 + open_amt * 11
        d.ellipse([44 - mw, 12, 44 + mw, 54], fill=hexc("#2a0006"))
        d.ellipse([44 - mw * 0.6, 16, 44 + mw * 0.6, 50], fill=hexc("#6a0a18"))
        for k in range(9):
            y = 15 + k * 4.4
            tw = 3
            d.polygon([(44 - mw, y), (44 - mw + tw + 1, y + 1.5), (44 - mw, y + 3)], fill=hexc("#f2ecd2"))
            d.polygon([(44 + mw, y + 1.5), (44 + mw - tw - 1, y + 3), (44 + mw, y + 4.5)], fill=hexc("#f2ecd2"))
        # throat glow
        d.ellipse([44 - 2, 28, 44 + 2, 38], fill=hexc("#ff7a2a"))
        # cluster of eyes on top
        for (ex, ey, er) in ((26, 14, 3), (62, 14, 3), (34, 7, 2.4), (54, 7, 2.4), (18, 28, 2.5), (70, 28, 2.5)):
            eye(d, ex, ey, er, iris=hexc("#ffc23a"))
        frames.append(outline(img, (10, 14, 6, 255)))
    sheet = Image.new("RGBA", (W * 8, H), (0, 0, 0, 0))
    for i, fr in enumerate(frames):
        sheet.paste(fr, (i * W, 0), fr)
    save(sheet, "maw.png")


def gen_seraph():
    """Seraph-class horror: interlocking rings covered in eyes, with tendril wings. 80x80, 8 frames."""
    W, H = 80, 80
    frames = []
    for f in range(8):
        ph = f / 8 * 2 * math.pi
        img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        cx, cy = 40, 38
        # tendril wings
        for side in (-1, 1):
            for k in range(5):
                a0 = -math.pi / 2 + side * (0.6 + k * 0.32) + math.sin(ph + k) * 0.08
                x, y = cx, cy
                pts = [(x, y)]
                for s in range(10):
                    a = a0 + side * s * 0.06 + math.sin(ph * 2 + s * 0.5 + k) * 0.05
                    x += math.cos(a) * 3.4
                    y += math.sin(a) * 3.4
                    pts.append((x, y))
                for i in range(len(pts) - 1):
                    w = 3 if i < 4 else (2 if i < 8 else 1)
                    c = hexc("#e8d8b0") if i % 3 else hexc("#b89c70")
                    d.line([pts[i], pts[i + 1]], fill=c, width=w)
        # rings (ellipses rotating)
        for (rx, ry, rot, col, n_eyes) in ((30, 10, ph, hexc("#c9a548"), 7), (10, 30, -ph * 0.7, hexc("#a8862c"), 6), (24, 24, ph * 0.4, hexc("#7a6020"), 5)):
            pts = []
            for i in range(64):
                a = i / 64 * 2 * math.pi
                x = math.cos(a) * rx
                y = math.sin(a) * ry
                c, s = math.cos(rot * 0.3), math.sin(rot * 0.3)
                pts.append((cx + x * c - y * s, cy + x * s + y * c))
            d.line(pts + [pts[0]], fill=col, width=3)
            d.line(pts + [pts[0]], fill=hexc("#f0d878"), width=1)
            for e in range(n_eyes):
                a = e / n_eyes * 2 * math.pi + rot
                x = math.cos(a) * rx
                y = math.sin(a) * ry
                c, s = math.cos(rot * 0.3), math.sin(rot * 0.3)
                eye(d, cx + x * c - y * s, cy + x * s + y * c, 2.6, iris=hexc("#5ad0ff"))
        # core
        d.ellipse([cx - 8, cy - 8, cx + 8, cy + 8], fill=hexc("#fff4c0"))
        d.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], fill=hexc("#ffcf40"))
        frames.append(outline(img, (30, 20, 6, 255)))
    sheet = Image.new("RGBA", (W * 8, H), (0, 0, 0, 0))
    for i, fr in enumerate(frames):
        sheet.paste(fr, (i * W, 0), fr)
    save(sheet, "seraph.png")


def gen_leviathan():
    """Boss: THE HOLLOW CROWN. Enormous head with a crown of horns, a tentacle beard
    and a dozen eyes. 192x160, 8 frames. Central eye pupil drawn in-game."""
    W, H = 192, 160
    frames = []
    for f in range(8):
        ph = f / 8 * 2 * math.pi
        img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        # beard of tentacles
        for i in range(11):
            bx = 40 + i * 11
            tentacle(d, (bx, 92), 44 + (i * 13) % 26, 12, ph + i * 0.7, 0.9, 8, hexc("#1c1430"), hexc("#140e24"))
        mask, dn = blob_mask(W, H, 96, 70, 64, 48, 0.06, 41, ph * 0.2)
        tex = flesh_texture(W, H, 21, (80, 60, 120), (16, 10, 30), (140, 40, 160))
        body = np_to_img(shade(tex, mask, dn), mask)
        img.alpha_composite(body)
        d = ImageDraw.Draw(img)
        # crown of horns
        for i in range(9):
            a = math.pi + (i + 1) / 10 * math.pi
            bx = 96 + math.cos(a) * 54
            by = 70 + math.sin(a) * 40
            tx = 96 + math.cos(a) * 82
            ty = 70 + math.sin(a) * 66 - 6
            d.polygon([(bx - 5, by), (bx + 5, by), (tx, ty)], fill=hexc("#d9d0b0"))
            d.line([(bx, by), (tx, ty)], fill=hexc("#8a8068"))
        # front tentacles
        for i in range(7):
            bx = 58 + i * 13
            tentacle(d, (bx, 104), 30 + (i * 7) % 16, 10, ph + i * 1.7 + 2, 1.2, 7, hexc("#3a2a5a"), hexc("#2c2046"))
        # eyes
        for (ex, ey, er) in ((52, 52, 6), (140, 52, 6), (66, 34, 4.5), (126, 34, 4.5), (44, 76, 4), (148, 76, 4),
                             (80, 22, 3.5), (112, 22, 3.5), (96, 18, 3), (70, 92, 3.5), (122, 92, 3.5)):
            blink = (f * 3 + int(ex)) % 13 == 0
            if blink:
                d.line([(ex - er, ey), (ex + er, ey)], fill=hexc("#120818"), width=2)
            else:
                eye(d, ex, ey, er, iris=hexc("#ff3050"))
        # maw under the main eye
        d.ellipse([72, 84, 120, 104], fill=hexc("#120008"))
        for k in range(8):
            x = 76 + k * 5.5
            d.polygon([(x, 85), (x + 3, 85), (x + 1.5, 92)], fill=hexc("#efe6c8"))
            d.polygon([(x + 1, 103), (x + 4, 103), (x + 2.5, 97)], fill=hexc("#efe6c8"))
        # the great eye socket
        d.ellipse([70, 44, 122, 82], fill=hexc("#0a0010"))
        d.ellipse([74, 47, 118, 79], fill=hexc("#f2e8d0"))
        d.ellipse([82, 50, 110, 76], fill=hexc("#ff2a6a"))
        d.ellipse([86, 54, 106, 72], fill=hexc("#b0103a"))
        frames.append(outline(img, (8, 2, 14, 255)))
    sheet = Image.new("RGBA", (W * 8, H), (0, 0, 0, 0))
    for i, fr in enumerate(frames):
        sheet.paste(fr, (i * W, 0), fr)
    save(sheet, "leviathan.png")


def gen_spawnling():
    """Small crawling thing dropped by Maws. 16x12, 4 frames."""
    W, H = 16, 12
    frames = []
    for f in range(4):
        ph = f / 4 * 2 * math.pi
        img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        for k in range(4):
            x = 3 + k * 3
            off = math.sin(ph + k * 1.5) * 1.5
            d.line([(x, 6), (x - 1 + off, 11)], fill=hexc("#2a3a18"), width=1)
        d.ellipse([2, 2, 14, 9], fill=hexc("#5c7a2a"))
        d.ellipse([4, 3, 10, 7], fill=hexc("#7a9a3a"))
        d.point((11, 4), fill=hexc("#ffdd33"))
        d.point((12, 5), fill=hexc("#ffdd33"))
        frames.append(outline(img, (10, 14, 4, 255)))
    sheet = Image.new("RGBA", (W * 4, H), (0, 0, 0, 0))
    for i, fr in enumerate(frames):
        sheet.paste(fr, (i * W, 0), fr)
    save(sheet, "spawnling.png")


# --------------------------------------------------------------------------
# RIFT portal (64x64, 8 frames, meant for additive blending)
# --------------------------------------------------------------------------
def gen_rift():
    W = 64
    frames = []
    yy, xx = np.mgrid[0:W, 0:W]
    cx = cy = W / 2 - 0.5
    dx, dy = xx - cx, yy - cy
    r = np.sqrt(dx * dx + dy * dy) / (W / 2)
    a = np.arctan2(dy, dx)
    for f in range(8):
        ph = f / 8 * 2 * math.pi
        swirl = np.sin(a * 3 + r * 12 - ph * 2) * 0.5 + 0.5
        ring = np.exp(-((r - 0.62) ** 2) / 0.02)
        core = np.clip(1 - r * 1.8, 0, 1)
        inten = np.clip(ring * (0.6 + 0.6 * swirl) + core * 0.8 * swirl, 0, 1)
        inten *= (r < 1)
        out = np.zeros((W, W, 4), dtype=np.uint8)
        out[:, :, 0] = np.clip(170 * inten + 80 * ring, 0, 255)
        out[:, :, 1] = np.clip(40 * inten + 200 * core * swirl, 0, 255)
        out[:, :, 2] = np.clip(255 * inten, 0, 255)
        out[:, :, 3] = np.clip(255 * inten, 0, 255)
        frames.append(Image.fromarray(out))
    sheet = Image.new("RGBA", (W * 8, W), (0, 0, 0, 0))
    for i, fr in enumerate(frames):
        sheet.paste(fr, (i * W, 0))
    save(sheet, "rift.png")


# --------------------------------------------------------------------------
# CRATES & PICKUPS (16x16 each) + parachute (24x18)
# order: 0 yellow(rockets) 1 purple(grenades) 2 red(fire) 3 green(shotguns)
#        4 blue(pistols/smart) 5 grey(exotic) 6 white-arrow(power) 7 medkit
# --------------------------------------------------------------------------
def gen_crates():
    cols = ["#d8b422", "#8a4ad0", "#d83a22", "#3aa83a", "#2a7ad8", "#8a8e92", "#e8e8e8", "#f0f0f0"]
    sheet = Image.new("RGBA", (16 * 8 + 24, 18), (0, 0, 0, 0))
    for i, c in enumerate(cols):
        img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        base = hexc(c)
        dark = tuple(int(v * 0.55) for v in base[:3]) + (255,)
        light = tuple(min(255, int(v * 1.25 + 20)) for v in base[:3]) + (255,)
        d.rectangle([1, 2, 14, 15], fill=hexc("#2a2c28"))
        d.rectangle([2, 3, 13, 14], fill=base)
        d.line([(2, 3), (13, 3)], fill=light)
        d.line([(2, 14), (13, 14)], fill=dark)
        d.rectangle([2, 8, 13, 9], fill=dark)   # band
        d.rectangle([6, 1, 9, 3], fill=hexc("#555852"))  # lifting ring
        if i == 6:
            d.polygon([(7.5, 4), (12, 9), (9, 9), (9, 13), (6, 13), (6, 9), (3, 9)], fill=hexc("#ffcc22"))
        if i == 7:
            d.rectangle([6, 4, 9, 13], fill=hexc("#e02020"))
            d.rectangle([3, 7, 12, 10], fill=hexc("#e02020"))
        else:
            # hazard stripes on the left
            for y in range(4, 14, 3):
                d.line([(2, y), (3, y + 1)], fill=hexc("#1a1a1a"))
        sheet.paste(img, (i * 16, 2), img)
    # parachute drone-chute
    p = Image.new("RGBA", (24, 18), (0, 0, 0, 0))
    d = ImageDraw.Draw(p)
    d.pieslice([0, 0, 23, 16], 180, 360, fill=hexc("#c8c0a0"))
    for k in range(1, 4):
        d.line([(k * 6, 1), (k * 6, 8)], fill=hexc("#8a8268"))
    d.line([(1, 8), (12, 17)], fill=hexc("#5a5a5a"))
    d.line([(22, 8), (12, 17)], fill=hexc("#5a5a5a"))
    d.line([(12, 8), (12, 17)], fill=hexc("#5a5a5a"))
    sheet.paste(p, (128, 0), p)
    save(sheet, "crates.png")


# --------------------------------------------------------------------------
# BACKGROUNDS
#   World is 1280 px wide, screen is 640x360.
# --------------------------------------------------------------------------
def gen_sky():
    W, H = 640, 360
    yy = np.linspace(0, 1, H)[:, None]
    top = np.array([6, 2, 14])
    mid = np.array([32, 8, 46])
    bot = np.array([70, 18, 52])
    g = np.where(yy < 0.6, top + (mid - top) * (yy / 0.6), mid + (bot - mid) * ((yy - 0.6) / 0.4))
    sky = np.repeat(g[:, None, :], W, axis=1).reshape(H, W, 3).astype(float)
    # nebula
    n1 = fbm(W, H, 140, 6, 101)
    n2 = fbm(W, H, 90, 6, 202)
    neb = np.clip((n1 - 0.45) * 2.5, 0, 1) ** 1.6
    neb2 = np.clip((n2 - 0.5) * 3.0, 0, 1) ** 2
    sky += neb[:, :, None] * np.array([120, 30, 140]) * 0.8
    sky += neb2[:, :, None] * np.array([20, 80, 120]) * 0.7
    # stars
    r = np.random.RandomState(7)
    for _ in range(700):
        x, y = r.randint(0, W), r.randint(0, int(H * 0.8))
        b = r.rand() ** 3
        sky[y, x] += 255 * b
        if b > 0.6 and x > 0 and y > 0 and x < W - 1 and y < H - 1:
            sky[y - 1, x] += 90 * b
            sky[y + 1, x] += 90 * b
            sky[y, x - 1] += 90 * b
            sky[y, x + 1] += 90 * b
    # dying moon, partially eclipsed by something
    my, mx, mr = 70, 470, 34
    Y, X = np.mgrid[0:H, 0:W]
    dm = np.sqrt((X - mx) ** 2 + (Y - my) ** 2)
    moon = dm < mr
    crater = fbm(W, H, 9, 3, 55)
    moonc = np.array([190, 170, 160]) * (0.7 + 0.4 * crater[:, :, None])
    shadow = np.sqrt((X - mx + 14) ** 2 + (Y - my + 6) ** 2) < mr * 0.95
    sky[moon] = moonc[moon]
    sky[moon & shadow] = sky[moon & shadow] * 0.12 + np.array([20, 0, 20])
    halo = np.exp(-((dm - mr) / 18.0) ** 2) * (dm > mr)
    sky += halo[:, :, None] * np.array([90, 40, 70])
    # the great tear in the sky
    tear = np.zeros((H, W))
    pts = []
    x, y = 120.0, 10.0
    r2 = random.Random(3)
    while y < 210:
        pts.append((x, y))
        x += r2.uniform(-14, 18)
        y += r2.uniform(6, 14)
    img = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(img)
    for i in range(len(pts) - 1):
        w = int(9 * math.sin(math.pi * i / len(pts)) + 1)
        d.line([pts[i], pts[i + 1]], fill=255, width=w)
        # branches
        if r2.random() < 0.5:
            bx, by = pts[i]
            for k in range(r2.randint(2, 5)):
                nx = bx + r2.uniform(-22, 22)
                ny = by + r2.uniform(-8, 16)
                d.line([(bx, by), (nx, ny)], fill=200, width=1)
                bx, by = nx, ny
    core = np.array(img).astype(float) / 255.0
    glow = np.array(img.filter(ImageFilter.GaussianBlur(14))).astype(float) / 255.0
    glow2 = np.array(img.filter(ImageFilter.GaussianBlur(40))).astype(float) / 255.0
    sky += glow2[:, :, None] * np.array([160, 30, 200]) * 1.6
    sky += glow[:, :, None] * np.array([220, 80, 255]) * 1.4
    sky = sky * (1 - core[:, :, None]) + core[:, :, None] * np.array([255, 235, 255])
    out = Image.fromarray(np.clip(sky, 0, 255).astype(np.uint8), "RGB").convert("RGBA")
    save(out, "bg_sky.png")


def gen_far():
    """Far silhouettes: atmosphere processor, colony towers. 1280x200, tileable, alpha."""
    W, H = 1280, 220
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    col = hexc("#1a0c24")
    hl = hexc("#2c1638")
    r = random.Random(11)
    # rolling hills
    pts = [(0, H)]
    for x in range(0, W + 1, 8):
        y = 170 + 14 * math.sin(x / W * 2 * math.pi * 3) + 8 * math.sin(x / W * 2 * math.pi * 7 + 1)
        pts.append((x, y))
    pts.append((W, H))
    d.polygon(pts, fill=col)
    # the processor - a giant stepped cone with stacks
    px = 820
    d.polygon([(px - 120, 175), (px - 70, 60), (px + 70, 60), (px + 120, 175)], fill=col)
    d.rectangle([px - 55, 20, px + 55, 62], fill=col)
    for k in range(5):
        sx = px - 46 + k * 23
        d.rectangle([sx, -10 + (k % 2) * 12, sx + 10, 22], fill=col)
    for k in range(6):
        y = 70 + k * 17
        w = 70 + k * 9
        d.line([(px - w, y), (px + w, y)], fill=hl)
    # towers
    for i in range(14):
        tx = r.randint(0, W)
        th = r.randint(40, 110)
        tw = r.randint(10, 26)
        d.rectangle([tx, 175 - th, tx + tw, 180], fill=col)
        d.line([(tx + tw // 2, 175 - th), (tx + tw // 2, 175 - th - r.randint(6, 24))], fill=col, width=2)
    # little lights
    for i in range(140):
        x = r.randint(0, W - 1)
        y = r.randint(40, 185)
        if img.getpixel((x, y))[3] > 0:
            c = r.choice([hexc("#ffb040"), hexc("#ff3a3a"), hexc("#9affd0"), hexc("#ffb040")])
            d.point((x, y), fill=c)
    save(img, "bg_far.png")


def gen_mid():
    """Mid layer: industrial wreckage, pipes, crashed dropship. 1280x260 alpha, tileable."""
    W, H = 1280, 260
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    col = hexc("#160a1c")
    col2 = hexc("#221028")
    r = random.Random(23)
    # pipes
    for k in range(3):
        y = 150 + k * 18
        d.rectangle([0, y, W, y + 6], fill=col2)
        for x in range(0, W, 64):
            d.rectangle([x + k * 13, y - 2, x + k * 13 + 5, y + 8], fill=col)
    # gantries
    for i in range(6):
        gx = 60 + i * 210 + r.randint(-30, 30)
        gh = r.randint(90, 170)
        d.rectangle([gx, H - gh, gx + 8, H], fill=col)
        d.rectangle([gx + 40, H - gh + 10, gx + 48, H], fill=col)
        for y in range(H - gh, H, 16):
            d.line([(gx + 8, y), (gx + 40, y + 16)], fill=col, width=2)
            d.line([(gx + 40, y), (gx + 8, y + 16)], fill=col, width=2)
        d.rectangle([gx - 10, H - gh - 6, gx + 58, H - gh], fill=col)
        d.point((gx + 24, H - gh - 7), fill=hexc("#ff2020"))
    # crashed dropship hull
    sx = 380
    d.polygon([(sx, H - 20), (sx + 40, H - 70), (sx + 190, H - 92), (sx + 240, H - 60), (sx + 220, H - 10)], fill=col)
    d.polygon([(sx + 150, H - 90), (sx + 200, H - 140), (sx + 214, H - 136), (sx + 190, H - 88)], fill=col)
    d.line([(sx + 50, H - 64), (sx + 190, H - 84)], fill=col2, width=2)
    # fires on the wreck
    for k in range(4):
        fx = sx + 60 + k * 40
        d.point((fx, H - 74 + k), fill=hexc("#ff7020"))
    # fence
    for x in range(0, W, 6):
        d.line([(x, H - 40), (x, H)], fill=col)
    d.line([(0, H - 40), (W, H - 40)], fill=col, width=2)
    d.rectangle([0, H - 14, W, H], fill=col)
    save(img, "bg_mid.png")


def gen_ground():
    """Deck plating: 64x64 tile for ground, plus a 32x32 container/crate block tile and catwalk."""
    T = 64
    base = fbm(T, T, 8, 3, 300, tile_x=True)
    img = np.zeros((T, T, 4), dtype=np.uint8)
    c0 = np.array([38, 36, 44])
    c1 = np.array([62, 58, 70])
    col = c0 + (c1 - c0) * base[:, :, None]
    # plate seams
    col[0:1, :] *= 1.6
    col[31:32, :] *= 0.5
    col[:, 0:1] *= 0.5
    col[:, 32:33] *= 1.4
    # rivets
    for (x, y) in ((4, 4), (27, 4), (36, 4), (59, 4), (4, 27), (27, 27), (36, 27), (59, 27)):
        col[y, x] = [110, 104, 116]
        col[y + 1, x + 1] = [20, 20, 24]
    # grime
    g = fbm(T, T, 16, 3, 301, tile_x=True)
    col *= (0.75 + 0.35 * g[:, :, None])
    img[:, :, :3] = np.clip(col, 0, 255)
    img[:, :, 3] = 255
    tile = Image.fromarray(img)
    # hazard edge strip on top
    d = ImageDraw.Draw(tile)
    d.rectangle([0, 0, T, 3], fill=hexc("#1a1a1a"))
    for x in range(-4, T, 8):
        d.polygon([(x, 3), (x + 4, 0), (x + 8, 0), (x + 4, 3)], fill=hexc("#c8a020"))
    d.line([(0, 4), (T, 4)], fill=hexc("#0c0c10"))
    save(tile, "tile_ground.png")

    # cargo block 32x32
    B = 32
    blk = Image.new("RGBA", (B, B), hexc("#3a4a52"))
    d = ImageDraw.Draw(blk)
    for x in range(2, B, 4):
        d.line([(x, 2), (x, B - 3)], fill=hexc("#2a363c"))
    d.rectangle([0, 0, B - 1, B - 1], outline=hexc("#1a2226"))
    d.rectangle([1, 1, B - 2, 1], fill=hexc("#5a6e78"))
    d.rectangle([3, 12, 13, 18], fill=hexc("#c8b080"))
    d.point((5, 14), fill=hexc("#303030"))
    d.point((7, 14), fill=hexc("#303030"))
    d.point((9, 14), fill=hexc("#303030"))
    save(blk, "tile_block.png")

    # catwalk grating 32x8
    cw = Image.new("RGBA", (32, 8), (0, 0, 0, 0))
    d = ImageDraw.Draw(cw)
    d.rectangle([0, 0, 31, 2], fill=hexc("#6a6470"))
    d.line([(0, 0), (31, 0)], fill=hexc("#9a94a0"))
    for x in range(0, 32, 4):
        d.line([(x, 3), (x + 3, 7)], fill=hexc("#3a3640"))
    d.line([(0, 7), (31, 7)], fill=hexc("#3a3640"))
    save(cw, "tile_catwalk.png")


# --------------------------------------------------------------------------
# UI bits: soft glow, motion tracker, title logo, icon
# --------------------------------------------------------------------------
def gen_glow():
    S = 64
    yy, xx = np.mgrid[0:S, 0:S]
    d = np.sqrt((xx - S / 2 + 0.5) ** 2 + (yy - S / 2 + 0.5) ** 2) / (S / 2)
    a = np.clip(1 - d, 0, 1) ** 2.2
    out = np.zeros((S, S, 4), dtype=np.uint8)
    out[:, :, :3] = 255
    out[:, :, 3] = (a * 255).astype(np.uint8)
    save(Image.fromarray(out), "glow.png")


def glitch_text(text, size, fill, w, h, seed):
    font = ImageFont.truetype(FONT_BOLD, size)
    base = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(base)
    bbox = d.textbbox((0, 0), text, font=font)
    tx = (w - (bbox[2] - bbox[0])) // 2 - bbox[0]
    ty = (h - (bbox[3] - bbox[1])) // 2 - bbox[1]
    d.text((tx, ty), text, font=font, fill=fill)
    return base


def gen_title():
    W, H = 600, 150
    main = glitch_text("RIFT ATTACK", 84, (235, 225, 255, 255), W, 110, 1)
    red = glitch_text("RIFT ATTACK", 84, (255, 30, 80, 200), W, 110, 1)
    cyan = glitch_text("RIFT ATTACK", 84, (40, 220, 255, 200), W, 110, 1)
    out = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    glow = main.filter(ImageFilter.GaussianBlur(8))
    ga = np.array(glow)
    ga[:, :, 0] = 200
    ga[:, :, 1] = 40
    ga[:, :, 2] = 255
    out.alpha_composite(Image.fromarray(ga))
    out.alpha_composite(red, (-4, 0))
    out.alpha_composite(cyan, (4, 0))
    out.alpha_composite(main)
    # horizontal slice glitches
    arr = np.array(out)
    r = np.random.RandomState(4)
    for _ in range(9):
        y = r.randint(10, 100)
        hh = r.randint(2, 6)
        sh = r.randint(-14, 14)
        arr[y:y + hh] = np.roll(arr[y:y + hh], sh, axis=1)
    out = Image.fromarray(arr)
    sub = glitch_text("V O I D   M A R I N E S", 22, (200, 255, 180, 255), W, 40, 2)
    out.alpha_composite(sub, (0, 108))
    save(out, "title.png")


def gen_icon():
    S = 256
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([8, 8, S - 8, S - 8], radius=48, fill=hexc("#120618"))
    # rift crack
    glow = Image.new("L", (S, S), 0)
    gd = ImageDraw.Draw(glow)
    pts = [(70, 30), (110, 80), (96, 120), (150, 160), (130, 200), (180, 236)]
    gd.line(pts, fill=255, width=26)
    gl = glow.filter(ImageFilter.GaussianBlur(18))
    col = Image.new("RGBA", (S, S), hexc("#c040ff"))
    img.paste(col, (0, 0), gl)
    d = ImageDraw.Draw(img)
    d.line(pts, fill=hexc("#ffe8ff"), width=6)
    # the eye
    d.ellipse([58, 78, 198, 178], fill=hexc("#f0e8d0"), outline=hexc("#2a0618"), width=8)
    d.ellipse([92, 92, 164, 164], fill=hexc("#b8ff3a"))
    d.ellipse([118, 96, 138, 160], fill=hexc("#050505"))
    d.ellipse([140, 104, 152, 116], fill=hexc("#ffffff"))
    img.save(os.path.join(RES, "icon.ico"), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    img.save(os.path.join(GFX, "icon.png"))
    print("  wrote res/icon.ico + assets/gfx/icon.png")


def gen_contact_sheet():
    """Preview of every generated sprite, for the README."""
    files = ["marine.png", "weapons.png", "gazer.png", "maw.png", "seraph.png", "leviathan.png", "spawnling.png",
             "rift.png", "crates.png"]
    imgs = [Image.open(os.path.join(GFX, f)) for f in files]
    W = max(min(i.width, 1536) for i in imgs) + 20
    H = sum(i.height + 10 for i in imgs) + 10
    sheet = Image.new("RGBA", (W, H), (24, 14, 32, 255))
    y = 10
    for im in imgs:
        sheet.alpha_composite(im.crop((0, 0, min(im.width, 1536), im.height)), (10, y))
        y += im.height + 10
    sheet = sheet.resize((sheet.width * 2, sheet.height * 2), Image.NEAREST)
    sheet.save(os.path.join(ROOT, "tools", "sprite_preview.png"))


if __name__ == "__main__":
    print("RIFT ATTACK art generator")
    gen_marine()
    gen_weapons()
    gen_gazer()
    gen_maw()
    gen_seraph()
    gen_leviathan()
    gen_spawnling()
    gen_rift()
    gen_crates()
    gen_sky()
    gen_far()
    gen_mid()
    gen_ground()
    gen_glow()
    gen_title()
    gen_icon()
    gen_contact_sheet()
    print("done.")
