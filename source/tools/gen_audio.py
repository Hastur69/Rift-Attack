#!/usr/bin/env python3
"""
RIFT ATTACK - procedural witch house soundtrack + sound effects.

No samples are used: every kick, hat, choir vowel and scream is synthesized
from oscillators and noise right here, then mixed, reverbed and encoded.

    python3 tools/gen_audio.py        (writes assets/music/*.ogg, assets/sfx/*.wav)

Requires: numpy, scipy, ffmpeg (with libvorbis) on PATH
"""
import os
import subprocess
import wave

import numpy as np
from scipy.signal import butter, sosfilt, fftconvolve

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MUS = os.path.join(ROOT, "assets", "music")
SFX = os.path.join(ROOT, "assets", "sfx")
os.makedirs(MUS, exist_ok=True)
os.makedirs(SFX, exist_ok=True)

SR = 44100
R = np.random.RandomState(666)


# ==========================================================================
# DSP primitives
# ==========================================================================
def t_axis(dur):
    return np.arange(int(dur * SR)) / SR


def midi(n):
    return 440.0 * 2 ** ((n - 69) / 12.0)


def lp(x, f, order=2):
    f = min(max(f, 20), SR / 2 - 100)
    return sosfilt(butter(order, f, "low", fs=SR, output="sos"), x)


def hp(x, f, order=2):
    f = min(max(f, 20), SR / 2 - 100)
    return sosfilt(butter(order, f, "high", fs=SR, output="sos"), x)


def bp(x, lo, hi, order=2):
    return sosfilt(butter(order, [max(lo, 20), min(hi, SR / 2 - 100)], "band", fs=SR, output="sos"), x)


def saw_from_phase(ph):
    return 2.0 * (ph - np.floor(ph + 0.5))


def osc_saw(freq, dur, detune_cents=0.0, phase0=0.0):
    t = t_axis(dur)
    f = freq * 2 ** (detune_cents / 1200.0)
    return saw_from_phase(f * t + phase0)


def osc_sine_glide(f0, f1, dur, glide):
    n = int(dur * SR)
    t = np.arange(n) / SR
    f = f1 + (f0 - f1) * np.exp(-t / max(glide, 1e-4))
    ph = np.cumsum(f) / SR
    return np.sin(2 * np.pi * ph)


def env_adsr(n, a, d, s, r, total=None):
    a_n, d_n, r_n = int(a * SR), int(d * SR), int(r * SR)
    sus_n = max(0, n - a_n - d_n - r_n)
    e = np.concatenate([
        np.linspace(0, 1, max(a_n, 1)),
        np.linspace(1, s, max(d_n, 1)),
        np.full(sus_n, s),
        np.linspace(s, 0, max(r_n, 1)),
    ])
    if len(e) < n:
        e = np.pad(e, (0, n - len(e)))
    return e[:n]


def exp_env(n, tau):
    return np.exp(-np.arange(n) / SR / tau)


def softclip(x, drive=1.0):
    return np.tanh(x * drive) / np.tanh(drive)


def bitcrush(x, bits=8, down=2):
    q = 2 ** (bits - 1)
    y = np.round(x * q) / q
    if down > 1:
        y = np.repeat(y[::down], down)[: len(x)]
    return y


def make_ir(seconds, seed, damp=6000.0, predelay=0.02):
    n = int(seconds * SR)
    r = np.random.RandomState(seed)
    noise = r.randn(n)
    t = np.arange(n) / SR
    decay = np.exp(-t * 6.9 / seconds)
    ir = lp(noise, damp) * decay
    ir = np.concatenate([np.zeros(int(predelay * SR)), ir])
    return ir / np.sqrt(np.sum(ir ** 2))


IR_CACHE = {}


def reverb(x, seconds=3.0, wet=0.4, damp=6000):
    """Stereo reverb from mono or stereo input. Returns (n, 2)."""
    key = (seconds, damp)
    if key not in IR_CACHE:
        IR_CACHE[key] = (make_ir(seconds, 1, damp), make_ir(seconds, 2, damp))
    irl, irr = IR_CACHE[key]
    if x.ndim == 1:
        xl = xr = x
    else:
        xl, xr = x[:, 0], x[:, 1]
    n = len(xl)
    wl = fftconvolve(xl, irl)[:n]
    wr = fftconvolve(xr, irr)[:n]
    dry = np.stack([xl, xr], 1)
    return dry * (1 - wet) + np.stack([wl, wr], 1) * wet


def pan(x, p):
    """p in [-1,1] -> stereo"""
    l = np.cos((p + 1) * np.pi / 4)
    r = np.sin((p + 1) * np.pi / 4)
    return np.stack([x * l, x * r], 1)


def place(buf, sig, start):
    s = int(start * SR)
    if s >= len(buf):
        return
    e = min(len(buf), s + len(sig))
    buf[s:e] += sig[: e - s]


# ==========================================================================
# Instruments
# ==========================================================================
def kick808(freq=42, dur=1.4, drive=3.0, click=0.6):
    n = int(dur * SR)
    body = osc_sine_glide(freq * 4.2, freq, dur, 0.035)
    amp = exp_env(n, dur * 0.38)
    k = body * amp
    clk = R.randn(min(n, 400)) * np.linspace(1, 0, min(n, 400)) * click
    k[: len(clk)] += lp(clk, 3000)
    return softclip(k, drive) * 0.9


def snare(dur=0.5, tone=190, noise_amt=1.0):
    n = int(dur * SR)
    nz = bp(R.randn(n), 1200, 9000) * exp_env(n, 0.09) * noise_amt
    tn = osc_sine_glide(tone * 1.6, tone, dur, 0.02) * exp_env(n, 0.05)
    return softclip(nz * 0.7 + tn * 0.8, 1.6) * 0.8


def clap(dur=0.6):
    n = int(dur * SR)
    nz = bp(R.randn(n), 900, 6000)
    e = np.zeros(n)
    for k, off in enumerate((0, 0.011, 0.022, 0.034)):
        s = int(off * SR)
        e[s:] += exp_env(n - s, 0.012 if k < 3 else 0.16)
    return nz * e * 0.6


def hat(dur=0.06, open_=False):
    n = int((0.35 if open_ else dur) * SR)
    nz = hp(R.randn(n), 7000, 4)
    return nz * exp_env(n, 0.12 if open_ else 0.018) * 0.24


def supersaw(freq, dur, voices=7, spread=22, cutoff=1800, attack=0.4, release=1.2, seed=0):
    r = np.random.RandomState(seed)
    out = np.zeros(int(dur * SR))
    for v in range(voices):
        dc = (v - (voices - 1) / 2) / ((voices - 1) / 2) * spread
        out += osc_saw(freq, dur, dc, r.rand())
    out /= voices
    out = lp(out, cutoff, 2)
    return out * env_adsr(len(out), attack, 0.3, 0.8, release)


FORMANTS = {
    "oo": [(300, 1.0), (870, 0.4), (2240, 0.15)],
    "oh": [(500, 1.0), (800, 0.5), (2830, 0.15)],
    "ah": [(730, 1.0), (1090, 0.6), (2440, 0.25)],
    "ee": [(270, 1.0), (2290, 0.5), (3010, 0.3)],
}


def choir(freq, dur, vowel="oo", voices=4, vibrato=5.0, attack=0.5, release=1.5, seed=0):
    """Ghostly formant choir. The witch-house 'ooooh'."""
    r = np.random.RandomState(seed)
    n = int(dur * SR)
    t = np.arange(n) / SR
    src = np.zeros(n)
    for v in range(voices):
        det = (r.rand() - 0.5) * 18
        vib = 1 + 0.006 * np.sin(2 * np.pi * (vibrato + r.rand()) * t + r.rand() * 6)
        f = freq * 2 ** (det / 1200) * vib
        ph = np.cumsum(f) / SR + r.rand()
        src += saw_from_phase(ph)
    src /= voices
    src += r.randn(n) * 0.04  # breath
    out = np.zeros(n)
    for (fc, g) in FORMANTS[vowel]:
        out += bp(src, fc * 0.88, fc * 1.12, 2) * g
    out = out * env_adsr(n, attack, 0.2, 0.9, release)
    return out * 2.4


def bell(freq, dur=2.5, bright=2.0, crush=True):
    """Music-box/FM bell, slightly detuned and crushed - haunted-nursery vibe."""
    n = int(dur * SR)
    t = np.arange(n) / SR
    mod = np.sin(2 * np.pi * freq * 3.5 * t) * bright * exp_env(n, 0.3)
    car = np.sin(2 * np.pi * freq * t + mod)
    car2 = np.sin(2 * np.pi * freq * 1.003 * t + mod * 0.5)
    out = (car + car2 * 0.5) * exp_env(n, dur * 0.3)
    if crush:
        out = bitcrush(out, 6, 3)
        out = lp(out, 5000)
    return out * 0.5


def bass808(freq, dur, drive=4.0, glide_from=None):
    """Sustained distorted sub that slides. The witch-house chest punch."""
    n = int(dur * SR)
    f0 = glide_from if glide_from else freq * 1.0
    body = osc_sine_glide(f0, freq, dur, 0.06)
    amp = env_adsr(n, 0.005, 0.25, 0.65, 0.25)
    x = body * amp
    x = softclip(x * 1.0 + 0.25 * np.sign(x) * x * x, drive)
    return lp(x, 900) * 0.8


def stab(freq, dur=0.6, cutoff=2600):
    out = np.zeros(int(dur * SR))
    for dc in (-14, -5, 0, 6, 13):
        out += osc_saw(freq, dur, dc, R.rand())
        out += osc_saw(freq * 2, dur, dc * 1.3, R.rand()) * 0.4
    out = lp(out / 6, cutoff)
    return softclip(out * exp_env(len(out), 0.22), 2.0) * 0.6


def siren(f_lo, f_hi, dur, rate=0.5):
    n = int(dur * SR)
    t = np.arange(n) / SR
    f = f_lo + (f_hi - f_lo) * (0.5 + 0.5 * np.sin(2 * np.pi * rate * t))
    ph = np.cumsum(f) / SR
    x = saw_from_phase(ph) * 0.6 + np.sin(2 * np.pi * ph * 0.5) * 0.4
    return lp(x, 2500) * env_adsr(n, 1.0, 0.1, 1, 1.0) * 0.4


def riser(dur, rev=False):
    n = int(dur * SR)
    t = np.linspace(0, 1, n)
    nz = R.randn(n)
    out = np.zeros(n)
    chunks = 32
    for c in range(chunks):
        s, e = c * n // chunks, (c + 1) * n // chunks
        fc = 400 + 7000 * (c / chunks) ** 2
        out[s:e] = bp(nz[s:e], fc * 0.7, fc * 1.3)
    out *= t ** 2
    return (out[::-1] if rev else out) * 0.5


def reverse_swell(sig, seconds=2.0):
    """reverb a sound, then reverse it: the classic ghostly suck-in."""
    tail = np.concatenate([sig, np.zeros(int(seconds * SR))])
    w = reverb(tail, seconds, 1.0)[:, 0]
    return w[::-1]


def screwed(x, factor=0.8):
    """Chopped & screwed: pitch AND time down by resampling."""
    idx = np.arange(0, len(x) - 1, factor)
    i0 = idx.astype(int)
    fr = idx - i0
    return x[i0] * (1 - fr) + x[i0 + 1] * fr


# ==========================================================================
# Song builder
# ==========================================================================
class Song:
    def __init__(self, bpm, bars, tail=4.0):
        self.bpm = bpm
        self.beat = 60.0 / bpm
        self.bar = self.beat * 4
        self.bars = bars
        self.length = self.bar * bars
        n = int((self.length + tail) * SR)
        self.buses = {k: np.zeros((n, 2)) for k in ("drums", "bass", "pad", "choir", "lead", "fx")}
        self.kick_times = []

    def at(self, bar, beat=0.0):
        return bar * self.bar + beat * self.beat

    def add(self, bus, sig, t, p=0.0, gain=1.0):
        if sig.ndim == 1:
            sig = pan(sig, p)
        place(self.buses[bus], sig * gain, t)

    def kick(self, t, **kw):
        self.kick_times.append(t)
        self.add("drums", kick808(**kw), t, 0, 1.0)

    def sidechain(self, depth=0.6, release=0.25):
        n = len(self.buses["pad"])
        env = np.ones(n)
        for kt in self.kick_times:
            s = int(kt * SR)
            ln = int(release * 2.5 * SR)
            if s >= n:
                continue
            e = min(n, s + ln)
            curve = 1 - depth * np.exp(-np.arange(e - s) / SR / release)
            env[s:e] = np.minimum(env[s:e], curve)
        return env[:, None]

    def render(self, rv_pad=4.5, rv_choir=5.0, master_drive=1.6, loop=True):
        sc = self.sidechain()
        drums = reverb(self.buses["drums"], 1.2, 0.15)
        bass = self.buses["bass"]
        pad = reverb(self.buses["pad"], rv_pad, 0.55, 5000) * sc
        ch = reverb(self.buses["choir"], rv_choir, 0.65, 4500) * (0.5 + 0.5 * sc)
        lead = reverb(self.buses["lead"], 2.5, 0.4, 7000)
        fx = reverb(self.buses["fx"], 3.0, 0.5)
        mix = drums * 1.0 + bass * 0.95 + pad * 0.55 + ch * 0.7 + lead * 0.5 + fx * 0.5
        mix = hp(mix.T, 25).T
        L = int(self.length * SR)
        if loop:
            tail = mix[L:]
            mix = mix[:L].copy()
            mix[: len(tail)] += tail
        # tape-ish saturation, light crush on the whole thing for that lo-fi grit
        mix = softclip(mix, master_drive)
        mix = mix * 0.85 + bitcrush(mix, 12, 1) * 0.15
        mix /= np.max(np.abs(mix)) + 1e-9
        # leave headroom: the game layers SFX on top, and Vorbis overshoots peaks
        return mix * 0.62


def write_ogg(name, stereo, q=5):
    tmp = os.path.join(MUS, name + ".tmp.wav")
    write_wav(tmp, stereo)
    out = os.path.join(MUS, name + ".ogg")
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", tmp, "-c:a", "libvorbis", "-q:a", str(q), out], check=True)
    os.remove(tmp)
    print("  wrote", os.path.relpath(out, ROOT), f"{len(stereo) / SR:.1f}s", f"{os.path.getsize(out) // 1024} KB")


def write_wav(path, data):
    data = np.asarray(data)
    ch = 1 if data.ndim == 1 else data.shape[1]
    pcm = (np.clip(data, -1, 1) * 32767).astype("<i2")
    with wave.open(path, "wb") as w:
        w.setnchannels(ch)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(pcm.tobytes())


# ==========================================================================
# TRACK 1 : "Rift Hymn" - title screen. 66 BPM, slow, sacred, sickly.
# ==========================================================================
def track_rift_hymn():
    s = Song(66, 20)
    # D minor: Dm - Bb - Gm - A
    prog = [(50, [50, 53, 57]), (46, [46, 50, 53]), (43, [43, 46, 50]), (45, [45, 49, 52])]
    vowels = ["oo", "oh", "oo", "ah"]
    for bar in range(s.bars):
        root, chord = prog[bar % 4]
        t = s.at(bar)
        for i, nte in enumerate(chord):
            s.add("pad", supersaw(midi(nte), s.bar + 0.5, cutoff=900 + 300 * (bar % 4), attack=1.2, seed=bar * 3 + i), t, (i - 1) * 0.5, 0.5)
        if bar >= 4:
            s.add("choir", choir(midi(chord[2] + 12), s.bar, vowels[bar % 4], seed=bar), t, -0.3, 0.6)
            s.add("choir", choir(midi(chord[0] + 12), s.bar, vowels[bar % 4], seed=bar + 50), t, 0.3, 0.5)
        if bar >= 8:
            s.add("bass", bass808(midi(root - 12), s.bar * 0.95, drive=2.5), t, 0, 0.8)
            s.kick(t)
            s.kick(s.at(bar, 2.5))
            s.add("drums", clap(), s.at(bar, 2), 0, 0.8)
            for k in range(8):
                if bar >= 12 or k % 2 == 0:
                    s.add("drums", hat(), s.at(bar, k * 0.5), 0.3, 0.6 if k % 2 else 0.9)
            if bar % 2 == 1:
                for k in range(6):
                    s.add("drums", hat(0.04), s.at(bar, 3.0 + k / 6), 0.3, 0.5 + k * 0.08)
    # haunted music box melody
    mel = [62, 65, 69, 74, 72, 70, 69, 65, 62, 65, 67, 70, 69, 67, 65, 64]
    for bar in range(2, s.bars):
        for k in range(4):
            note = mel[((bar - 2) * 4 + k) % len(mel)]
            if (bar + k) % 3 != 2:
                s.add("lead", bell(midi(note + 12), 2.0), s.at(bar, k), 0.4 if k % 2 else -0.4, 0.55)
    # reverse swells into each phrase
    for bar in (3, 7, 11, 15):
        sw = reverse_swell(choir(midi(74), 1.0, "ee"), 2.0)
        s.add("fx", sw, s.at(bar + 1) - len(sw) / SR, 0, 0.6)
    write_ogg("rift_hymn", s.render(master_drive=1.3))


# ==========================================================================
# TRACK 2 : "Ichor Drill" - main combat. 74 BPM half-time, HARD.
# ==========================================================================
def track_ichor_drill():
    s = Song(74, 40)
    # E phrygian/harmonic: Em - C - Am - B
    prog = [(40, [52, 55, 59]), (36, [48, 52, 55]), (45, [45, 48, 52]), (47, [47, 51, 54])]
    for bar in range(s.bars):
        root, chord = prog[(bar // 2) % 4]
        t = s.at(bar)
        breakdown = 24 <= bar < 28
        intro = bar < 4
        # pads
        for i, nte in enumerate(chord):
            s.add("pad", supersaw(midi(nte), s.bar + 0.3, cutoff=1400 if not breakdown else 600, attack=0.3, seed=bar * 7 + i), t, (i - 1) * 0.6, 0.45)
        # chopped choir: gated 8ths
        if not intro:
            v = ["oo", "ah", "oh", "ee"][(bar // 2) % 4]
            for k in range(8):
                if k in (0, 3, 6) or (bar % 2 and k == 7):
                    c = choir(midi(chord[k % 3] + 12), s.beat * 0.45, v, attack=0.01, release=0.15, seed=bar * 11 + k)
                    s.add("choir", c, s.at(bar, k * 0.5), 0.5 if k % 2 else -0.5, 0.8)
        if breakdown:
            s.add("choir", choir(midi(64), s.bar, "ah", seed=bar), t, 0, 0.9)
            s.add("fx", riser(s.bar), t, 0, 0.6 if bar == 27 else 0.2)
            continue
        if intro:
            if bar == 3:
                s.add("fx", riser(s.bar), t, 0, 0.8)
            for k in range(4):
                s.add("drums", hat(), s.at(bar, k), 0.2, 0.7)
            continue
        # drums: half-time trap pattern
        s.kick(t, freq=40, drive=4.0)
        s.kick(s.at(bar, 1.75), freq=40, drive=4.0)
        if bar % 2 == 1:
            s.kick(s.at(bar, 3.25), freq=40, drive=4.0)
        s.add("drums", snare(), s.at(bar, 2), 0, 1.0)
        s.add("drums", clap(), s.at(bar, 2), 0, 0.7)
        if bar % 4 == 3:
            s.add("drums", snare(0.3), s.at(bar, 3.75), 0, 0.6)
        # hats: 16ths with rolls
        for k in range(16):
            s.add("drums", hat(), s.at(bar, k * 0.25), 0.35, 0.55 + 0.25 * (k % 2 == 0))
        if bar % 2 == 1:
            for k in range(8):
                s.add("drums", hat(0.03), s.at(bar, 2.5 + k / 8), -0.35, 0.4 + k * 0.06)
        if bar % 4 == 2:
            for k in range(12):
                s.add("drums", hat(0.025), s.at(bar, 3.0 + k / 12), 0.0, 0.3 + k * 0.05)
        s.add("drums", hat(open_=True), s.at(bar, 3.5), -0.3, 0.5)
        # 808 bass with slides
        b = midi(root - 12)
        s.add("bass", bass808(b, s.beat * 1.7, drive=5.0), t)
        s.add("bass", bass808(b, s.beat * 1.2, drive=5.0, glide_from=b * 1.5), s.at(bar, 1.75))
        if bar % 2 == 1:
            s.add("bass", bass808(b * 0.94, s.beat * 0.7, drive=5.0, glide_from=b * 1.33), s.at(bar, 3.25))
        # saw stabs
        if bar >= 8 and bar % 2 == 0:
            for k, off in enumerate((0.0, 0.75, 1.5)):
                s.add("lead", stab(midi(chord[0] + 12)), s.at(bar, off), -0.2 + 0.2 * k, 0.6)
        # siren lead in second half
        if 16 <= bar < 24 or bar >= 32:
            if bar % 4 == 0:
                s.add("lead", siren(midi(64), midi(71), s.bar * 4, rate=0.25), t, 0.3, 0.35)
        # music box ghosts
        if bar % 2 == 0:
            for k, nte in enumerate((chord[2] + 24, chord[1] + 24, chord[0] + 24)):
                s.add("lead", bell(midi(nte), 1.5), s.at(bar, 1 + k * 0.5), 0.6, 0.25)
    write_ogg("ichor_drill", s.render(rv_pad=3.5, master_drive=1.8))


# ==========================================================================
# TRACK 3 : "Black Static" - boss / high-tier. 82 BPM, darker, faster rolls.
# ==========================================================================
def track_black_static():
    s = Song(82, 32)
    # C minor descending: Cm - Bb/D - Ab - G
    prog = [(36, [48, 51, 55]), (34, [46, 50, 53]), (32, [44, 48, 51]), (31, [43, 47, 50])]
    for bar in range(s.bars):
        root, chord = prog[bar % 4]
        t = s.at(bar)
        for i, nte in enumerate(chord):
            s.add("pad", supersaw(midi(nte - 12), s.bar + 0.3, cutoff=700, attack=0.05, voices=9, spread=30, seed=bar * 5 + i), t, (i - 1) * 0.7, 0.55)
        # screwed choir
        c = choir(midi(chord[0] + 12) * 1.25, s.bar * 0.8, "ah" if bar % 2 else "oh", seed=bar * 13)
        s.add("choir", screwed(c, 0.8), t, 0.2 * (1 if bar % 2 else -1), 0.7)
        if bar < 2:
            continue
        # drums - double-kick bursts, snare on 3
        for off in (0, 0.75, 1.5):
            s.kick(s.at(bar, off), freq=38, drive=5.5)
        if bar % 2:
            s.kick(s.at(bar, 3.0), freq=38, drive=5.5)
            s.kick(s.at(bar, 3.5), freq=38, drive=5.5)
        s.add("drums", snare(tone=170), s.at(bar, 2), 0, 1.0)
        s.add("drums", clap(), s.at(bar, 2), 0, 0.9)
        for k in range(16):
            s.add("drums", hat(0.04), s.at(bar, k * 0.25), 0.4 if k % 2 else -0.4, 0.6)
        for k in range(16):
            if bar % 2 == 0:
                s.add("drums", hat(0.02), s.at(bar, 3.0 + k / 16), 0.0, 0.25 + k * 0.03)
        b = midi(root - 12)
        s.add("bass", bass808(b, s.beat * 1.4, drive=6.0), t)
        s.add("bass", bass808(b, s.beat * 1.0, drive=6.0, glide_from=b * 2), s.at(bar, 1.5))
        s.add("bass", bass808(b * 1.19, s.beat * 1.0, drive=6.0), s.at(bar, 2.75))
        # detuned descending lead
        lead_notes = [chord[2] + 12, chord[1] + 12, chord[0] + 12, chord[1] + 11]
        for k, nte in enumerate(lead_notes):
            st = stab(midi(nte), 0.5, 3500)
            s.add("lead", st, s.at(bar, k), 0.5 if k % 2 else -0.5, 0.45)
        if bar % 8 == 7:
            s.add("fx", riser(s.bar, False), t, 0, 0.7)
        if bar % 8 == 0:
            s.add("fx", siren(midi(55), midi(67), s.bar * 2, 1.0), t, -0.5, 0.4)
    write_ogg("black_static", s.render(rv_pad=2.5, master_drive=2.0))


# ==========================================================================
# TRACK 4 : "Elegy for the Squad" - game over. 60 BPM, drifting.
# ==========================================================================
def track_elegy():
    s = Song(60, 8)
    prog = [(45, [57, 60, 64]), (41, [53, 57, 60]), (43, [55, 59, 62]), (40, [52, 56, 59])]
    for bar in range(s.bars):
        root, chord = prog[bar % 4]
        t = s.at(bar)
        for i, nte in enumerate(chord):
            s.add("choir", choir(midi(nte), s.bar + 0.5, ["oo", "oh"][i % 2], attack=1.0, seed=bar * 9 + i), t, (i - 1) * 0.6, 0.55)
        s.add("pad", supersaw(midi(root), s.bar + 0.5, cutoff=500, attack=1.5, seed=bar), t, 0, 0.6)
        for k in range(2):
            s.add("lead", bell(midi(chord[(bar + k) % 3] + 24), 3.0), s.at(bar, k * 2 + 1), 0.5 - k, 0.4)
    write_ogg("elegy", s.render(rv_pad=6.0, rv_choir=7.0, master_drive=1.1))


# ==========================================================================
# SOUND EFFECTS
# ==========================================================================
def norm(x, peak=0.9):
    return x / (np.max(np.abs(x)) + 1e-9) * peak


def sfx(name, x, peak=0.9):
    write_wav(os.path.join(SFX, name + ".wav"), norm(x, peak))


def noise(n):
    return R.randn(n)


def gen_sfx():
    # Pulse carbine: chattery electric thump with a ringing tail
    n = int(0.22 * SR)
    t = np.arange(n) / SR
    body = osc_sine_glide(420, 90, 0.22, 0.012) * exp_env(n, 0.03)
    crack = bp(noise(n), 1500, 8000) * exp_env(n, 0.012)
    ring = np.sin(2 * np.pi * 2300 * t) * exp_env(n, 0.03) * 0.25
    zap = saw_from_phase(np.cumsum(np.linspace(1800, 300, n)) / SR) * exp_env(n, 0.02) * 0.3
    sfx("pulse", softclip(body + crack + ring + zap, 2.5))

    # pistol
    n = int(0.15 * SR)
    sfx("pistol", softclip(bp(noise(n), 600, 6000) * exp_env(n, 0.02) + osc_sine_glide(300, 80, 0.15, 0.01) * exp_env(n, 0.03), 2))

    # shotgun
    n = int(0.5 * SR)
    boom = lp(noise(n), 2500) * exp_env(n, 0.08) + osc_sine_glide(160, 45, 0.5, 0.03) * exp_env(n, 0.1)
    sfx("shotgun", softclip(boom, 3))

    # rocket launch
    n = int(0.7 * SR)
    whoosh = bp(noise(n), 300, 3000) * env_adsr(n, 0.01, 0.1, 0.6, 0.5)
    thump = osc_sine_glide(200, 50, 0.7, 0.05) * exp_env(n, 0.08)
    sfx("rocket", softclip(whoosh + thump, 2))

    # grenade launch: hollow bloop
    n = int(0.25 * SR)
    sfx("grenade", osc_sine_glide(500, 140, 0.25, 0.03) * exp_env(n, 0.06) + lp(noise(n), 1500) * exp_env(n, 0.02) * 0.4)

    # bounce
    n = int(0.08 * SR)
    sfx("bounce", np.sin(2 * np.pi * 900 * np.arange(n) / SR) * exp_env(n, 0.015) + noise(n) * exp_env(n, 0.005) * 0.3, 0.6)

    # explosion
    n = int(1.6 * SR)
    ex = lp(noise(n), 1200) * exp_env(n, 0.35) + osc_sine_glide(120, 30, 1.6, 0.15) * exp_env(n, 0.4) * 1.2
    ex += lp(noise(n), 6000) * exp_env(n, 0.05) * 0.6
    sfx("explosion", softclip(ex, 3))

    # singularity bomb: implosion suck + massive boom
    n = int(3.0 * SR)
    suck = reverse_swell(bp(noise(int(0.4 * SR)), 200, 3000), 1.0)
    boom = lp(noise(n), 700) * exp_env(n, 0.8) + osc_sine_glide(90, 22, 3.0, 0.3) * exp_env(n, 0.9) * 1.5
    out = np.zeros(len(suck) + n)
    out[: len(suck)] += suck * 0.8
    out[len(suck):] += boom
    sfx("singularity", softclip(out, 3))

    # rail lance: charged zap with metallic ring
    n = int(0.9 * SR)
    t = np.arange(n) / SR
    zap = saw_from_phase(np.cumsum(np.linspace(3000, 200, n)) / SR) * exp_env(n, 0.1)
    ringm = np.sin(2 * np.pi * 1760 * t) * np.sin(2 * np.pi * 2637 * t) * exp_env(n, 0.3)
    sfx("rail", softclip(zap * 0.7 + ringm * 0.6 + hp(noise(n), 4000) * exp_env(n, 0.03), 2))

    # flame burst (short, plays rapidly)
    n = int(0.18 * SR)
    fl = bp(noise(n), 200, 2500) * env_adsr(n, 0.01, 0.05, 0.7, 0.1)
    sfx("flame", softclip(fl * 2, 2), 0.6)

    # mine drop
    n = int(0.3 * SR)
    sfx("mine", np.sin(2 * np.pi * 220 * np.arange(n) / SR) * exp_env(n, 0.05) + bp(noise(n), 2000, 5000) * exp_env(n, 0.01))

    # fire pillar
    n = int(1.2 * SR)
    fp = lp(noise(n), 1800) * env_adsr(n, 0.05, 0.2, 0.7, 0.6)
    sfx("firepillar", softclip(fp * 2 + osc_sine_glide(100, 60, 1.2, 0.2) * exp_env(n, 0.4), 2))

    # harpoon: twang + whip
    n = int(0.5 * SR)
    t = np.arange(n) / SR
    tw = saw_from_phase(110 * t + 0.02 * np.sin(2 * np.pi * 9 * t)) * exp_env(n, 0.15)
    sfx("harpoon", lp(tw, 2000) + bp(noise(n), 2000, 8000) * exp_env(n, 0.02))

    # smart rifle: tighter, higher pulse
    n = int(0.14 * SR)
    sfx("smart", softclip(osc_sine_glide(700, 150, 0.14, 0.008) * exp_env(n, 0.02) + bp(noise(n), 3000, 9000) * exp_env(n, 0.008), 2), 0.7)

    # empty click
    n = int(0.05 * SR)
    sfx("empty", bp(noise(n), 3000, 9000) * exp_env(n, 0.004), 0.5)

    # weapon switch: rack
    n = int(0.25 * SR)
    sw = np.zeros(n)
    for off in (0, 0.09):
        s = int(off * SR)
        m = int(0.05 * SR)
        sw[s:s + m] += bp(noise(m), 1500, 7000) * exp_env(m, 0.008)
    sfx("switch", sw, 0.6)

    # pickup / powerup / medkit
    n = int(0.35 * SR)
    t = np.arange(n) / SR
    arp = np.zeros(n)
    for k, f in enumerate((659, 880, 1318)):
        s = int(k * 0.06 * SR)
        m = n - s
        arp[s:] += np.sign(np.sin(2 * np.pi * f * np.arange(m) / SR)) * exp_env(m, 0.08) * 0.4
    sfx("pickup", lp(arp, 6000), 0.6)
    n = int(1.2 * SR)
    pw = choir(midi(76), 1.2, "ah", attack=0.02, release=0.6) + bell(midi(88), 1.2) * 0.5
    sfx("powerup", reverb(pw, 1.5, 0.4)[:, 0], 0.8)
    n = int(0.6 * SR)
    md = sum(np.sin(2 * np.pi * f * np.arange(n) / SR) * exp_env(n, 0.2) for f in (523, 659, 784))
    sfx("medkit", md, 0.6)

    # hurt / death (marine)
    n = int(0.3 * SR)
    sfx("hurt", softclip(osc_sine_glide(260, 120, 0.3, 0.05) * exp_env(n, 0.08) + bp(noise(n), 400, 2500) * exp_env(n, 0.05), 2), 0.7)
    n = int(1.4 * SR)
    sfx("death", reverb(softclip(choir(midi(45), 1.0, "ah", attack=0.01, release=0.6) + osc_sine_glide(200, 50, 1.0, 0.4) * 0.5, 2), 2.0, 0.5)[:n, 0])

    # jump / land
    n = int(0.12 * SR)
    sfx("jump", bp(noise(n), 500, 2500) * exp_env(n, 0.03) + osc_sine_glide(150, 300, 0.12, 0.05) * exp_env(n, 0.03) * 0.3, 0.4)
    n = int(0.12 * SR)
    sfx("land", lp(noise(n), 800) * exp_env(n, 0.02) + osc_sine_glide(120, 60, 0.12, 0.02) * exp_env(n, 0.03), 0.5)
    n = int(0.6 * SR)
    sfx("hyperjump", softclip(bp(noise(n), 200, 4000) * exp_env(n, 0.15) + osc_sine_glide(80, 600, 0.6, 0.2) * exp_env(n, 0.2), 2), 0.7)

    # motion tracker ping
    n = int(0.25 * SR)
    t = np.arange(n) / SR
    sfx("tracker", np.sin(2 * np.pi * 1450 * t) * exp_env(n, 0.06) + np.sin(2 * np.pi * 2900 * t) * exp_env(n, 0.02) * 0.2, 0.35)

    # rift open: tearing reality
    n = int(1.5 * SR)
    rp = reverse_swell(bp(noise(int(0.3 * SR)), 300, 4000), 1.0)
    tear = bp(noise(n), 100, 1200) * env_adsr(n, 0.02, 0.3, 0.3, 1.0) + siren(midi(30), midi(42), 1.5, 3) * 0.6
    sfx("rift", np.concatenate([rp * 0.6, softclip(tear, 2)]), 0.7)

    # horror vocalizations (screwed choir + noise screams)
    def scream(base, dur, vowel, screw):
        c = choir(base, dur, vowel, voices=6, vibrato=11, attack=0.03, release=0.3, seed=int(base))
        g = bp(noise(len(c)), 800, 4000) * env_adsr(len(c), 0.02, 0.1, 0.4, 0.3) * 0.5
        x = softclip((c + g) * 2, 3)
        return screwed(x, screw)

    sfx("gazer", reverb(scream(midi(70), 0.6, "ee", 0.7), 1.2, 0.3)[:, 0], 0.7)
    sc = scream(midi(40), 1.0, "ah", 0.6)
    sfx("maw", reverb(sc + lp(noise(len(sc)), 300) * 0.3 * exp_env(len(sc), 0.4), 1.5, 0.3)[:, 0], 0.8)
    sfx("seraph", reverb(choir(midi(79), 1.2, "ah", 5, 6, 0.1, 0.6) + choir(midi(86), 1.2, "ee", 5, 6, 0.1, 0.6), 2.5, 0.6)[:, 0], 0.7)
    sfx("boss", reverb(scream(midi(28), 2.4, "oh", 0.5), 3.0, 0.4)[:, 0], 0.95)

    # flesh hit / death splat / void bolt
    n = int(0.09 * SR)
    sfx("hit", lp(noise(n), 1500) * exp_env(n, 0.015) + osc_sine_glide(200, 90, 0.09, 0.01) * exp_env(n, 0.02) * 0.5, 0.45)
    n = int(0.9 * SR)
    splat = lp(noise(n), 900) * exp_env(n, 0.12) + bp(noise(n), 200, 900) * exp_env(n, 0.3) * 0.5
    sfx("splat", softclip(splat, 2))
    n = int(0.25 * SR)
    t = np.arange(n) / SR
    bolt = np.sin(2 * np.pi * np.cumsum(np.linspace(900, 200, n)) / SR) * exp_env(n, 0.07)
    sfx("voidbolt", bolt + hp(noise(n), 3000) * exp_env(n, 0.02) * 0.3, 0.5)

    # time rift on / off
    n = int(1.0 * SR)
    sfx("timerift", screwed(reverb(bell(midi(84), 1.0, 3.0, False), 2.0, 0.6)[:, 0], 0.5), 0.7)

    # UI
    n = int(0.08 * SR)
    sfx("ui_move", np.sign(np.sin(2 * np.pi * 440 * np.arange(n) / SR)) * exp_env(n, 0.02), 0.3)
    n = int(0.4 * SR)
    sfx("ui_select", reverb(bell(midi(81), 0.4, 1.5, True), 1.0, 0.4)[:n, 0], 0.6)
    print("  wrote", len(os.listdir(SFX)), "sfx")


if __name__ == "__main__":
    import sys
    which = sys.argv[1:] or ["sfx", "hymn", "ichor", "static", "elegy"]
    print("RIFT ATTACK audio generator")
    if "sfx" in which:
        gen_sfx()
    if "hymn" in which:
        track_rift_hymn()
    if "ichor" in which:
        track_ichor_drill()
    if "static" in which:
        track_black_static()
    if "elegy" in which:
        track_elegy()
    print("done.")
