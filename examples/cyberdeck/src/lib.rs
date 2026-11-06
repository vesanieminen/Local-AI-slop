//! CALC//DECK — a calculator from 2077.
//!
//! A Rune app (Rust → WASM → WebGPU) dressed as a netrunner's field deck:
//! the readout is drawn glyph-by-glyph as seven-segment neon (never a font),
//! the window behind it rains on a synthwave city that never sleeps, every
//! key throws sparks and squeals in WebAudio, division by zero trips an ICE
//! alarm, and 1337 opens a door. Native on the desktop; one canvas in the
//! browser.

mod engine;
mod scene;
mod seg;
mod sfx;

use engine::{Engine, Event, Op, Radix};
use rune::tok::TextStyle;
use rune::{
    canvas, col, el, row, text, App, AppSpec, Color, Cx, El, Key, KeyEvent, LineCap, Pixels,
    ScrollSpec, Shadow, Shape, Tx,
};
use scene::hash;

// ---------------------------------------------------------------------------
// Channels. Tapping the brand retunes the deck's whole soul: neon, amber CRT,
// blood, growth-terminal green.
// ---------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Ghostwire,
    Amber,
    Blood,
    Moss,
}

pub struct Pal {
    pub acc: Color,
    pub acc2: Color,
    pub hot: Color,
    pub amber: Color,
    pub red: Color,
    pub ghost: Color,
    pub sky_hi: Color,
    pub sky_lo: Color,
    pub bld_far: Color,
    pub bld_near: Color,
    pub plate_hi: Color,
    pub plate_lo: Color,
    pub screen: Color,
}

fn hx(s: &str) -> Color {
    rune::color::hex(s).unwrap_or(Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 })
}

impl Pal {
    fn get(ch: Channel) -> Pal {
        let (acc, acc2, hot, sky_hi, sky_lo, plate_hi, plate_lo) = match ch {
            Channel::Ghostwire => ("#38f2ff", "#ff38c4", "#eafeff", "#070714", "#170a26", "#141a2b", "#0a0d18"),
            Channel::Amber => ("#ffb648", "#ff7a2d", "#fff0d6", "#100a06", "#1d1007", "#211708", "#120c06"),
            Channel::Blood => ("#ff4257", "#ff9a3d", "#ffe9ec", "#120409", "#230610", "#23101a", "#140810"),
            Channel::Moss => ("#45ff8b", "#d4ff45", "#eafff0", "#04100a", "#0a1f12", "#0e2016", "#08130d"),
        };
        Pal {
            acc: hx(acc),
            acc2: hx(acc2),
            hot: hx(hot),
            amber: hx("#ffc245"),
            red: hx("#ff3355"),
            ghost: hx("#8f9bb3"),
            sky_hi: hx(sky_hi),
            sky_lo: hx(sky_lo),
            bld_far: hx("#131a2c"),
            bld_near: hx("#080b13"),
            plate_hi: hx(plate_hi),
            plate_lo: hx(plate_lo),
            screen: hx("#03060a"),
        }
    }

    fn name(ch: Channel) -> &'static str {
        match ch {
            Channel::Ghostwire => "GHOSTWIRE",
            Channel::Amber => "AMBER",
            Channel::Blood => "BLOOD",
            Channel::Moss => "MOSS",
        }
    }
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
enum Act {
    Ac,
    Back,
    Pct,
    Op(Op),
    Dig(char),
    Hex(char), // letters the hex row shows when the deck thinks in 16s
    Dot,
    Sign,
    Eq,
}

/// Keypad, reading order. Indices 20..26 are the hex row that slides in when
/// the deck is told to think in 16s.
const KEYS: [(&str, Act); 26] = [
    ("AC", Act::Ac),
    ("DEL", Act::Back),
    ("%", Act::Pct),
    ("/", Act::Op(Op::Div)),
    ("7", Act::Dig('7')),
    ("8", Act::Dig('8')),
    ("9", Act::Dig('9')),
    ("x", Act::Op(Op::Mul)),
    ("4", Act::Dig('4')),
    ("5", Act::Dig('5')),
    ("6", Act::Dig('6')),
    ("-", Act::Op(Op::Sub)),
    ("1", Act::Dig('1')),
    ("2", Act::Dig('2')),
    ("3", Act::Dig('3')),
    ("+", Act::Op(Op::Add)),
    ("+-", Act::Sign),
    ("0", Act::Dig('0')),
    (".", Act::Dot),
    ("=", Act::Eq),
    ("A", Act::Hex('A')),
    ("B", Act::Hex('B')),
    ("C", Act::Hex('C')),
    ("D", Act::Hex('D')),
    ("E", Act::Hex('E')),
    ("F", Act::Hex('F')),
];

struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    born: f64,
    color: Color,
}

#[derive(Clone, Copy, PartialEq)]
enum LogKind {
    Sys,
    Calc,
    Ghost,
}

struct Log {
    text: String,
    kind: LogKind,
}

const BOOT_END: f64 = 1.75;
const READY_AT: f64 = 1.35;

const BOOT_LINES: [(f64, &str); 5] = [
    (0.25, "> CALCTR OS v2.077 ......... NO CARRIER"),
    (0.55, "> MEM 64 TB ................ OK"),
    (0.85, "> ICE WAFERS ............... 4 ARMED"),
    (1.12, "> GHOSTWIRE UPLINK ......... UP"),
    (1.38, "> DECK READY. MIND THE ICE."),
];

const TICKS: [&str; 6] = [
    "RAIN INDEX 88 // TRANSIT 7 RUNS SILENT",
    "KURO-TECH STOCK +12% // WHO CARES",
    "GHOST SIGNAL REPEATS ON 5G",
    "ICE PATROLS DOUBLE IN SECTOR 4",
    "TAP THE BRAND TO RETUNE THE DECK",
    "TYPE 1337 WHEN YOU DARE",
];

pub struct State {
    clock: f64,
    seed: u64,
    /// `?beat`: present every frame even when static — a capture hook only.
    beat: bool,
    eng: Engine,
    log: Vec<Log>,
    granted: bool,
    channel: Channel,
    key_t: [f64; 26],
    key_r: [[f32; 4]; 26],
    sparks: Vec<Spark>,
    spark_n: u64,
    glitch_until: f64,
    ice_t: f64,
    eq_t: f64,
    /// What the readout spells while `eng.err` holds.
    err_word: &'static str,
    chimed: bool,
    sfx: sfx::Sink,
    vig: Pixels,
}

impl Default for State {
    fn default() -> Self {
        // `mut` only feeds the native tooling block below; on wasm nothing mutates it.
        #[allow(unused_mut)]
        let mut s = State {
            clock: 0.0,
            seed: 0,
            beat: false,
            eng: Engine::default(),
            log: vec![Log { text: "> DECK COLD. UPLINK STUBBORN.".into(), kind: LogKind::Sys }],
            granted: false,
            channel: Channel::Ghostwire,
            key_t: [-9.0; 26],
            key_r: [[0.0; 4]; 26],
            sparks: Vec::new(),
            spark_n: 0,
            glitch_until: 0.0,
            ice_t: -9.0,
            eq_t: -9.0,
            err_word: "",
            chimed: false,
            sfx: Default::default(),
            vig: scene::vignette(),
        };
        // Native headless tooling (`--snap`): CYBER_SEED / CYBER_DEMO / CYBER_HERO / CYBER_ICE / CYBER_BEAT.
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(v) = std::env::var("CYBER_SEED") {
                s.seed = v.parse().unwrap_or(0);
            }
            if std::env::var("CYBER_HERO").is_ok() {
                seed_demo(&mut s, true);
            } else if std::env::var("CYBER_ICE").is_ok() {
                seed_ice(&mut s);
            } else if std::env::var("CYBER_DEMO").is_ok() {
                seed_demo(&mut s, false);
            }
            if std::env::var("CYBER_BEAT").is_ok() {
                s.beat = true;
            }
        }
        s
    }
}

/// Scripted deterministic state for snaps, screenshots and CI: the deck has
/// just finished 1024 × 16, and in hero mode somebody then typed 1337.
fn seed_demo(s: &mut State, hero: bool) {
    s.clock = READY_AT + 0.05; // the presses below shouldn't wait out the boot
    for d in ['1', '0', '2', '4'] {
        press(s, None, digit_idx(d));
    }
    press(s, None, 7); // ×
    for d in ['1', '6'] {
        press(s, None, digit_idx(d));
    }
    press(s, None, 19); // =
    if hero {
        for d in ['1', '3', '3', '7'] {
            press(s, None, digit_idx(d));
        }
    }
}

fn digit_idx(d: char) -> usize {
    KEYS.iter().position(|(l, a)| *l == d.to_string() && matches!(a, Act::Dig(_))).unwrap_or(17)
}

/// Scripted divide-by-zero so the ICE alarm is caught mid-flash. The alarm
/// fires as late as possible before the snap (`--at` a hair past this) so the
/// red wash and glitch are near full strength; `err_word` holds the readout.
fn seed_ice(s: &mut State) {
    s.clock = 2.3;
    press(s, None, digit_idx('5'));
    press(s, None, 3); // ÷
    press(s, None, digit_idx('0'));
    press(s, None, 19); // =  → ICE//
}

// ---------------------------------------------------------------------------
// Pressing keys
// ---------------------------------------------------------------------------
fn press(s: &mut State, tx: Option<&mut Tx>, idx: usize) {
    if s.clock < READY_AT {
        return;
    }
    s.key_t[idx] = s.clock;
    let act = KEYS[idx].1;

    let sound = match act {
        Act::Dig(_) | Act::Hex(_) => sfx::Press::Digit,
        Act::Dot => sfx::Press::Dot,
        Act::Op(_) => sfx::Press::Op,
        Act::Eq => sfx::Press::Eq,
        Act::Ac => sfx::Press::Ac,
        Act::Back | Act::Pct | Act::Sign => sfx::Press::Switch,
    };
    sfx::press(&mut s.sfx, idx, sound);

    let pal = Pal::get(s.channel);
    let spark_c = match act {
        Act::Eq | Act::Hex(_) => pal.acc2,
        Act::Ac | Act::Pct => pal.amber,
        _ => pal.acc,
    };
    zap(s, idx, 9, spark_c);

    match act {
        Act::Ac => s.eng.reset(),
        Act::Back => s.eng.back(),
        Act::Pct => s.eng.percent(),
        Act::Op(o) => s.eng.op(o),
        Act::Dig(d) => s.eng.digit(d),
        Act::Hex(d) => {
            if s.eng.radix == Radix::Hex {
                s.eng.digit(d);
            }
        }
        Act::Dot => s.eng.dot(),
        Act::Sign => s.eng.sign(),
        Act::Eq => s.eng.equals(),
    }

    let mut logged = false;
    match s.eng.event.take() {
        Some(Event::Equals) => {
            s.eq_t = s.clock;
            s.glitch_until = (s.glitch_until).max(s.clock + 0.24);
            if !s.eng.expr.is_empty() {
                let line = format!("> {} {}", s.eng.expr, s.eng.cur);
                s.log.push(Log { text: line, kind: LogKind::Calc });
                s.eng.expr.clear();
                logged = true;
            }
        }
        Some(Event::Ice) => {
            s.ice_t = s.clock;
            s.err_word = "ICE//";
            s.glitch_until = s.clock + 0.9;
            s.log.push(Log { text: "> // ICE DETECTED — RESULT FROZEN".into(), kind: LogKind::Ghost });
            logged = true;
            sfx::ice(&mut s.sfx);
        }
        Some(Event::Out) => {
            s.ice_t = s.clock;
            s.err_word = "OUT//";
            s.glitch_until = s.clock + 0.55;
            s.log.push(Log { text: "> RESULT ESCAPED THE DECK (OUT)".into(), kind: LogKind::Ghost });
            logged = true;
            sfx::ice(&mut s.sfx);
        }
        Some(Event::Ghost) => {
            s.glitch_until = s.clock + 0.7;
            if !s.granted {
                s.granted = true;
                s.log.push(Log { text: "> ACCESS GRANTED — GHOST LINK UP".into(), kind: LogKind::Ghost });
                logged = true;
            }
            sfx::ghost(&mut s.sfx);
        }
        _ => {}
    }
    if logged {
        s.log.drain(..s.log.len().saturating_sub(48));
        if let Some(tx) = tx {
            tx.scroll_to_bottom("tape");
        }
    }
}

/// Spark burst from a key's centre (learned from `on_drawn_layout`).
fn zap(s: &mut State, idx: usize, n: usize, color: Color) {
    let [x, y, w, h] = s.key_r[idx];
    if w <= 0.0 {
        return;
    }
    let cx = x + w * (0.3 + 0.4 * frand(s, 1));
    let cy = y + h * 0.42;
    for _ in 0..n {
        let ang = std::f32::consts::PI * (0.12 + 0.76 * frand(s, 2)); // fan across the sky
        let spd = 90.0 + 330.0 * frand(s, 3);
        let vx = -ang.cos() * spd * (0.6 + 0.8 * frand(s, 4));
        let vy = -ang.sin() * spd * (0.5 + 0.7 * frand(s, 5));
        s.sparks.push(Spark {
            x: cx,
            y: cy,
            vx,
            vy,
            born: s.clock,
            color,
        });
        s.spark_n = s.spark_n.wrapping_add(7);
    }
    if s.sparks.len() > 110 {
        let cut = s.sparks.len() - 110;
        s.sparks.drain(..cut);
    }
}

fn frand(s: &mut State, k: u64) -> f32 {
    hash(s.spark_n.wrapping_mul(31).wrapping_add(k) ^ s.seed.wrapping_mul(0x9E37_79B9))
}

// ---------------------------------------------------------------------------
// The frame clock
// ---------------------------------------------------------------------------
fn poll(s: &mut State, t: f64, tx: &mut Tx) -> bool {
    let dt = (t - s.clock).clamp(0.0, 0.05) as f32;
    s.clock = t;
    if !s.chimed && t > 0.3 {
        s.chimed = true;
        sfx::boot(&mut s.sfx);
    }
    for sp in s.sparks.iter_mut() {
        sp.x += sp.vx * dt;
        sp.y += sp.vy * dt;
        sp.vy += 620.0 * dt;
        sp.vx *= 1.0 - 1.6 * dt;
    }
    s.sparks.retain(|sp| t - sp.born < 0.6);
    let busy = t < 4.0 || !s.sparks.is_empty() || s.eng.err || t < s.glitch_until + 1.6;
    tx.wake_after(std::time::Duration::from_millis(if busy || s.beat { 16 } else { 25 }));
    true
}

// ---------------------------------------------------------------------------
// Physical keyboard
// ---------------------------------------------------------------------------
fn key_for(c: char, hex: bool) -> Option<usize> {
    let find = |label: &str| KEYS.iter().position(|(l, _)| *l == label);
    match c {
        '0'..='9' => KEYS.iter().position(|(l, a)| *l == c.to_string() && matches!(a, Act::Dig(_))),
        '.' => find("."),
        '+' => find("+"),
        '-' => find("-"),
        'x' | 'X' | '*' => find("x"),
        '/' => find("/"),
        '=' => find("="),
        '%' => find("%"),
        'c' | 'C' => find("AC"),
        'a'..='f' | 'A'..='F' if hex => {
            let u = c.to_ascii_uppercase();
            KEYS.iter().position(|(l, _)| l.starts_with(u))
        }
        _ => None,
    }
}

fn on_key(s: &mut State, ev: &KeyEvent, tx: &mut Tx) -> bool {
    if s.clock < READY_AT {
        return false;
    }
    let hex = s.eng.radix == Radix::Hex;
    match &ev.key {
        Key::Text(t) => match t.chars().next() {
            Some('h' | 'H') => {
                set_radix(s, !hex);
                true
            }
            Some(c) if key_for(c, hex).is_some() => {
                let idx = key_for(c, hex).unwrap();
                press(s, Some(tx), idx);
                true
            }
            _ => false,
        },
        Key::Enter => {
            press(s, Some(tx), 19);
            true
        }
        Key::Escape => {
            press(s, Some(tx), 0);
            true
        }
        Key::Backspace => {
            press(s, Some(tx), 1);
            true
        }
        _ => false,
    }
}

fn set_radix(s: &mut State, hex: bool) {
    s.eng.set_radix(if hex { Radix::Hex } else { Radix::Dec });
    s.glitch_until = (s.glitch_until).max(s.clock + 0.2);
    s.log.push(Log { text: format!("> RADIX {}", if hex { 16 } else { 10 }), kind: LogKind::Sys });
    sfx::press(&mut s.sfx, 0, sfx::Press::Switch);
}

// ---------------------------------------------------------------------------
// Views: plates, chips, keys, readout, tape, and the deck itself
// ---------------------------------------------------------------------------
fn plate(e: El, pal: &Pal) -> El {
    e.linear_gradient(std::f32::consts::PI, [(0.0, pal.plate_hi), (1.0, pal.plate_lo)])
        .border_gradient(1.5, pal.acc.alpha(0.22), pal.acc.alpha(0.06))
        .box_shadow(Shadow { x: 0.0, y: 2.0, blur: 3.0, spread: -0.5, color: Color::BLACK.alpha(0.55), inset: true })
        .box_shadow(Shadow { x: 0.0, y: -1.5, blur: 2.0, spread: -1.0, color: pal.acc.alpha(0.05), inset: true })
        .shadow(10.0, 26.0, Color::BLACK.alpha(0.55))
}

/// Neon text: a dim halo copy under the hot face, stacked in one line box.
fn neon(label: &str, size: f32, weight: f32, track: f32, halo: Color, face: Color) -> El {
    let lh = size * 1.3;
    let f = || TextStyle::new(size, weight).lh(lh).tracking(track);
    col()
        .gap(0.0)
        .child(text(label.to_string()).font(f()).color(halo))
        .child(text(label.to_string()).font(f()).color(face).mt(-lh + 1.5))
}

fn chip(label: String, active: bool, pal: &Pal, on_press: impl Fn(&mut State, &mut Tx) + 'static) -> El {
    el()
        .h(24.0)
        .px(9.0)
        .center()
        .radius(5.0)
        .fill(if active { pal.acc.alpha(0.16) } else { Color::BLACK.alpha(0.35) })
        .border(1.0, if active { pal.acc.alpha(0.65) } else { pal.ghost.alpha(0.35) })
        .press_scale(0.93)
        .hover(pal.acc.alpha(0.08))
        .child(
            text(label)
                .font(TextStyle::new(10.5, 700.0).tracking(1.6))
                .color(if active { pal.hot } else { pal.ghost }),
        )
        .on_press_tx(on_press)
}

fn deck_key(s: &State, idx: usize, kh: f32, pal: &Pal) -> El {
    let (label, act) = KEYS[idx];
    let t = s.clock;
    let age = t - s.key_t[idx];
    let bump = if age < 0.45 { (1.0 - age / 0.45) as f32 } else { 0.0 };
    let is_eq = matches!(act, Act::Eq);
    let is_ac = matches!(act, Act::Ac | Act::Pct);
    let is_hex = matches!(act, Act::Hex(_));
    let accent = if is_eq || is_hex { pal.acc2 } else if is_ac { pal.amber } else { pal.acc };

    let mut k = el()
        .key(format!("k{idx}"))
        .grow(1.0)
        .min_w(0.0)
        .h(kh)
        .center()
        .radius(10.0)
        .fill(if is_eq { pal.acc2.alpha(0.12) } else { pal.plate_hi.alpha(0.4) })
        .linear_gradient(std::f32::consts::PI, [(0.0, pal.plate_hi.alpha(0.75)), (1.0, pal.plate_lo.alpha(0.9))])
        .press_scale(0.92)
        .hover(accent.alpha(0.08))
        .border(1.2 + 0.9 * bump, accent.alpha(0.30 + 0.55 * bump))
        .box_shadow(Shadow {
            x: 0.0,
            y: 0.0,
            blur: 12.0 + 10.0 * bump,
            spread: 0.5,
            color: accent.alpha(0.10 + 0.32 * bump),
            inset: true,
        })
        .child(neon(label, if is_eq { 20.0 } else { 17.0 }, 700.0, 1.5, accent.alpha(0.85), if is_eq { pal.hot } else { accent }));
    if bump > 0.01 {
        k = k.shadow(0.0, 4.0 + 10.0 * bump, accent.alpha(0.4 * bump));
    }
    k.on_drawn_layout(move |st: &mut State, r| st.key_r[idx] = r)
        .on_press_tx(move |st: &mut State, tx: &mut Tx| press(st, Some(tx), idx))
}

fn keypad(s: &State, kh: f32, gap: f32, pal: &Pal) -> El {
    let mut grid = col().w_full().gap(gap);
    for row_i in 0..5 {
        let mut line = row().w_full().gap(gap);
        for col_i in 0..4 {
            line = line.child(deck_key(s, row_i * 4 + col_i, kh, pal));
        }
        grid = grid.child(line);
    }
    if s.eng.radix == Radix::Hex {
        let mut line = row().w_full().gap(gap);
        for k in 20..26 {
            line = line.child(deck_key(s, k, kh * 0.72, pal));
        }
        grid = grid.child(line);
    }
    grid
}

/// The readout glyphs right now: boot scramble, the entry, or the alarm word.
fn readout(s: &State) -> String {
    if s.clock < READY_AT {
        if s.clock > 1.1 {
            return "0".into();
        }
        let bucket = (s.clock * 13.0).floor() as u64;
        return (0..8u64)
            .map(|i| "0123456789HEX-".chars().nth(hash(bucket * 17 + i) as usize % 14).unwrap_or('0'))
            .collect();
    }
    if s.eng.err {
        return s.err_word.to_string();
    }
    s.eng.cur.clone()
}

fn screen(s: &State, cx: &mut Cx<State>, sh: f32, pal: &Pal, compact: bool) -> El {
    let t = s.clock;
    let booting = t < BOOT_END;
    let alarm = s.eng.err || t - s.ice_t < 1.05;

    // Readout metrics: shrink cells until the whole entry fits the panel.
    let avail_w = cx.layout_size("screen-inner").map_or(320.0, |w| w[0]) - 8.0;
    let chars: Vec<char> = readout(s).chars().collect();
    let units: f32 = chars.iter().map(|c| seg::advance(seg::glyph(*c))).sum::<f32>()
        + 0.08 * (chars.len().max(1) - 1) as f32;
    let max_h = (sh - if compact { 66.0 } else { 84.0 }).max(30.0);
    let cell = ((max_h * seg::CW / seg::CH).min(avail_w / units.max(0.1))).clamp(13.0, 56.0);
    let dw = seg::measure(&chars, cell, 0.08);
    let dh = cell * seg::CH / seg::CW;

    let split = if t < s.glitch_until || t < READY_AT + 0.2 { 2.0 + 3.0 * hash((t * 40.0) as u64) } else { 0.0 };
    let heat = if booting { (((t - 0.4) / 0.7).clamp(0.0, 1.0)) as f32 } else { 1.0 };
    let alarm_f = if alarm { 0.78 + 0.22 * hash((t * 15.0) as u64) } else { 1.0 };
    let sweep = if t - s.eq_t < 0.35 { Some((t - s.eq_t) as f32 / 0.35 * (dw + 40.0) - 20.0) } else { None };

    let mut body = if alarm {
        seg::draw(Shape::new(), &chars, 0.0, 0.0, cell, pal.red.alpha(0.05), pal.red, pal.hot, pal.red, t, 3, heat * alarm_f, sweep)
    } else {
        seg::draw(Shape::new(), &chars, 0.0, 0.0, cell, pal.ghost.alpha(0.05), pal.acc, pal.hot, pal.acc, t, 0, heat, sweep)
    };
    if split > 0.0 {
        let off = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
        body = seg::draw(body, &chars, -split, 0.0, cell, off, pal.acc, pal.acc, pal.acc, t, 5, heat * 0.45, None);
        body = seg::draw(body, &chars, split, 1.5, cell, off, pal.acc2, pal.acc2, pal.acc2, t, 9, heat * 0.45, None);
    }

    let boot_line = BOOT_LINES.iter().rev().find(|(at, _)| t >= *at).map(|(_, l)| *l).unwrap_or("> WAKING…");

    plate(
        el()
            .w_full()
            .h(sh)
            .radius(14.0)
            .padding(if compact { 9.0 } else { 12.0 })
            .fill(pal.screen),
        pal,
    )
    .border(1.3, if alarm { pal.red.alpha(0.85) } else { pal.acc.alpha(0.32 + 0.14 * hash((t * 2.0) as u64)) })
    .box_shadow(Shadow {
        x: 0.0,
        y: 0.0,
        blur: 16.0,
        spread: 1.0,
        color: if alarm { pal.red.alpha(0.25) } else { pal.acc.alpha(0.10) },
        inset: true,
    })
    .child(
        col()
            .key("screen-inner")
            .w_full()
            .h_full()
            .gap(4.0)
            // top line: the pending expression (boot log during boot)
            .child(
                row()
                    .w_full()
                    .h(16.0)
                    .child(
                        text(if booting { boot_line.to_string() } else { s.eng.expr.clone() })
                            .font(TextStyle::new(11.0, 500.0).tracking(0.8))
                            .color(if booting { pal.acc.alpha(0.9) } else { pal.ghost })
                            .grow(1.0)
                            .min_w(0.0)
                            .truncate(),
                    )
                    .child(
                        text(format!("R{}", if s.eng.radix == Radix::Hex { 16 } else { 10 }))
                            .font(TextStyle::new(10.0, 700.0).tracking(1.8))
                            .color(pal.amber.alpha(0.85)),
                    ),
            )
            // the readout itself
            .child(
                row()
                    .w_full()
                    .grow(1.0)
                    .min_h(0.0)
                    .center()
                    .child(canvas(body).key("readout").size(dw, dh)),
            )
            // bottom line: the machine mirror and deck status
            .child(
                row()
                    .w_full()
                    .h(16.0)
                    .child(
                        text(s.eng.mirror())
                            .font(TextStyle::new(10.5, 500.0).tracking(0.6))
                            .color(pal.acc2.alpha(0.75))
                            .grow(1.0)
                            .min_w(0.0)
                            .truncate(),
                    )
                    .child(
                        text(if alarm {
                            "ALARM"
                        } else if s.granted {
                            "GHOST LINK"
                        } else {
                            "ONLINE"
                        })
                        .font(TextStyle::new(10.0, 700.0).tracking(2.0))
                        .color(if alarm { pal.red } else if s.granted { pal.amber } else { pal.ghost.alpha(0.8) }),
                    ),
            ),
    )
    .child(
        // glass: a faint raking reflection over the panel
        el()
            .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
            .radius(14.0)
            .linear_gradient(0.62, [(0.0, Color::WHITE.alpha(0.035)), (0.45, Color::WHITE.alpha(0.0)), (1.0, Color::WHITE.alpha(0.012))])
            .inert(),
    )
}

fn tape(s: &State, pal: &Pal) -> El {
    let mut board = col().w_full().gap(4.0).py(2.0);
    for (i, l) in s.log.iter().enumerate() {
        let c = match l.kind {
            LogKind::Sys => pal.acc.alpha(0.75),
            LogKind::Calc => pal.ghost,
            LogKind::Ghost => pal.amber,
        };
        board = board.child(
            text(l.text.clone())
                .key(format!("log-{i}"))
                .w_full()
                .wrap()
                .font(TextStyle::new(11.5, 500.0).tracking(0.4).lh(15.0))
                .color(c),
        );
    }
    plate(el().key("tape").max_w(320.0).h_full().radius(14.0).padding(12.0), pal).child(
        col()
            .w_full()
            .h_full()
            .gap(8.0)
            .child(
                row()
                    .w_full()
                    .child(text("TX LOG").font(TextStyle::new(11.0, 700.0).tracking(3.0)).color(pal.acc))
                    .child(el().grow(1.0))
                    .child(text("// HISTORY").font(TextStyle::new(9.5, 500.0).tracking(1.5)).color(pal.ghost.alpha(0.6))),
            )
            .child(
                col()
                    .w_full()
                    .grow(1.0)
                    .shrink()
                    .min_h(0.0)
                    .scroll(ScrollSpec { stick_bottom: true })
                    .scroll_edge_fade(14.0)
                    .child(board),
            ),
    )
}

fn vents(pal: &Pal, w: f32, h: f32) -> El {
    canvas((0..7u32).fold(Shape::new(), |s, i| {
        let x = 4.0 + i as f32 * (w / 7.0);
        s.line([x, 0.0], [x, h], 2.2, LineCap::Round, pal.acc.alpha(0.12))
    }))
    .size(w, h)
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let [safe_t, safe_r, safe_b, safe_l] = cx.safe_area();
    let [rw, rh] = cx.layout_size("root").unwrap_or([900.0, 800.0]);
    let t = s.clock;
    let pal = Pal::get(s.channel);
    let wide = rw >= 760.0;
    let compact = rh < 600.0; // landscape phones: drop the fripperies first
    let ready = t >= READY_AT;

    // — the city behind the glass
    let city = el()
        .key("root")
        .w_full()
        .h_full()
        .linear_gradient(std::f32::consts::PI, [(0.0, pal.sky_hi), (1.0, pal.sky_lo)])
        .shape(scene::city(rw, rh, t, &pal))
        .child(el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).pixels(&s.vig));

    // — the deck column
    let sh = if compact { (rh * 0.28).clamp(88.0, 130.0) } else { (rh * 0.21).clamp(150.0, 220.0) };
    let chrome = if compact { 202.0 } else { 188.0 };
    let kh = ((rh - safe_t - safe_b - sh - chrome) / 5.0).clamp(if compact { 30.0 } else { 44.0 }, if compact { 64.0 } else { 92.0 });
    // The hex row slides in above the footer's headroom — give every key back the sliver.
    let kh = if s.eng.radix == Radix::Hex { kh * 5.0 / 5.75 } else { kh };

    let u = (((t - 1.6).max(0.0) / 3.8) % 1.0) as f32;
    let ticker_i = (((t - 1.6).max(0.0) / 3.8).floor() as usize) % TICKS.len();
    let tick_a = (u * 5.0).min((1.0 - u) * 5.0).clamp(0.0, 1.0);

    let header = row()
        .w_full()
        .gap(10.0)
        .center_y()
        .child(
            col()
                .key("brand")
                .gap(1.0)
                .press_scale(0.97)
                .on_press_tx(|st: &mut State, tx: &mut Tx| {
                    st.channel = match st.channel {
                        Channel::Ghostwire => Channel::Amber,
                        Channel::Amber => Channel::Blood,
                        Channel::Blood => Channel::Moss,
                        Channel::Moss => Channel::Ghostwire,
                    };
                    st.glitch_until = (st.glitch_until).max(st.clock + 0.35);
                    st.log.push(Log { text: format!("> CHANNEL → {}", Pal::name(st.channel)), kind: LogKind::Sys });
                    tx.scroll_to_bottom("tape");
                    sfx::press(&mut st.sfx, 0, sfx::Press::Switch);
                })
                .child(neon(
                    if s.granted { "CALC//DECK G" } else { "CALC//DECK" },
                    21.0,
                    800.0,
                    3.0,
                    pal.acc2.alpha(0.65),
                    pal.hot,
                ))
                .child(
                    text(format!("MK-IX · {} · FIELD UNIT", Pal::name(s.channel)))
                        .font(TextStyle::new(10.0, 500.0).tracking(2.2))
                        .color(pal.ghost.alpha(0.75)),
                ),
        )
        .child(el().grow(1.0).min_w(0.0))
        .child(
            chip(
                format!("R{}", if s.eng.radix == Radix::Hex { 16 } else { 10 }),
                s.eng.radix == Radix::Hex,
                &pal,
                |st: &mut State, _tx: &mut Tx| {
                    let now = st.eng.radix == Radix::Hex;
                    set_radix(st, !now);
                },
            ),
        );

    let latest = s.log.iter().rev().find(|l| l.kind == LogKind::Calc).map(|l| l.text.clone()).unwrap_or_else(|| "> NO TRANSACTIONS".into());

    let mut deck = plate(
        col().w_full().h_full().radius(18.0).padding(if compact { 8.0 } else { 15.0 }).gap(if compact { 5.0 } else { 10.0 }),
        &pal,
    )
    .child(header);
    if !compact {
        deck = deck.child(
            text(TICKS[ticker_i])
                .w_full()
                .font(TextStyle::new(9.5, 500.0).tracking(1.4))
                .color(pal.acc2.alpha(0.8 * tick_a))
                .truncate(),
        );
    }
    deck = deck
        .child(screen(s, cx, sh, &pal, compact))
        .child(if ready { keypad(s, kh, if compact { 5.0 } else { 9.0 }, &pal) } else { keypad(s, kh, if compact { 5.0 } else { 9.0 }, &pal).inert() })
        .child(el().grow(1.0).min_h(0.0));
    if !compact {
        deck = deck.child(
            row()
                .w_full()
                .center_y()
                .child(
                    text(if wide { "MOD.9 // WARRANTY VOID // NOC 2077".to_string() } else { latest })
                        .font(TextStyle::new(9.0, 500.0).tracking(1.4))
                        .color(pal.ghost.alpha(0.55))
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                )
                .child(vents(&pal, 70.0, 12.0)),
        );
    }

    let deck_col = col()
        .grow(1.0)
        .max_w(470.0)
        .h_full()
        .pt(safe_t + 10.0)
        .pr(safe_r + 10.0)
        .pb(safe_b + 10.0)
        .pl(safe_l + 10.0)
        .child(deck);

    // A rare holographic dip — the deck is a projection, after all.
    let dip = hash((t / 6.3).floor() as u64) < 0.02 && t > 3.0;

    let body = if wide {
        row().w_full().h_full().gap(4.0).child(deck_col).child(
            col()
                .grow(0.8)
                .min_w(0.0)
                .h_full()
                .pt(safe_t + 10.0)
                .pr(safe_r + 10.0)
                .pb(safe_b + 10.0)
                .child(tape(s, &pal)),
        )
    } else {
        col().w_full().h_full().child(deck_col)
    };

    // — overlays painted over everything, in window points
    let mut fx = Shape::new();
    for sp in &s.sparks {
        let a = (1.0 - ((t - sp.born) / 0.6).clamp(0.0, 1.0)) as f32;
        fx = fx.line(
            [sp.x, sp.y],
            [sp.x - sp.vx * 0.022, sp.y - sp.vy * 0.022],
            1.6,
            LineCap::Round,
            sp.color.alpha(0.85 * a),
        );
    }
    // glitch bars while a burst runs
    if t < s.glitch_until || t - s.ice_t < 0.9 {
        let bucket = (t * 40.0).floor() as u64;
        for i in 0..3u64 {
            let y = hash(bucket * 5 + i) * rh;
            let bh = 3.0 + 9.0 * hash(bucket * 7 + i);
            let x = hash(bucket * 11 + i) * rw * 0.6;
            let bw = rw * (0.15 + 0.5 * hash(bucket * 13 + i));
            fx = fx
                .fill(rect_shape(x, y, bw, bh), pal.acc2.alpha(0.16))
                .fill(rect_shape(x + 9.0, y, bw, bh), pal.acc.alpha(0.12));
        }
    }
    // 1337: the ghost occasionally strokes the screen
    if s.granted && hash(((t / 5.0).floor() as u64) * 3 + 1) < 0.12 {
        fx = fx.fill(rect_shape(0.0, hash(((t / 5.0).floor() as u64) * 17) * rh, rw, 2.0), pal.amber.alpha(0.25));
    }

    let ice_a = ((1.0 - (t - s.ice_t) / 1.05).max(0.0) as f32 * (0.55 + 0.45 * (t * 26.0).sin() as f32)).max(0.0);

    city.child(body.opacity(if dip { 0.9 } else { 1.0 }))
        .child(el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).shape(fx).inert())
        .child(
            el()
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .shape(scene::scanlines(rw, rh, t))
                .inert(),
        )
        .child(
            el()
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .radial_gradient(
                    [0.5, 0.5],
                    [rw * 0.75, rh * 0.75],
                    [(0.0, pal.red.alpha(0.0)), (0.65, pal.red.alpha(0.10)), (1.0, pal.red.alpha(0.42))],
                )
                .opacity(ice_a)
                .inert(),
        )
}

fn rect_shape(x: f32, y: f32, w: f32, h: f32) -> Vec<[f32; 2]> {
    vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}

// ---------------------------------------------------------------------------
// Entrypoints
// ---------------------------------------------------------------------------
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn start() {
    #[allow(unused_mut)] // mutated only on wasm (URL args)
    let mut state = State::default();
    #[cfg(target_arch = "wasm32")]
    apply_url_args(&mut state);
    let app = App::new(AppSpec {
        min_size: Some((300.0, 480.0)),
        poll: Some(poll),
        on_key: Some(on_key),
        ..AppSpec::new("CALC//DECK MK-IX", (390.0, 780.0), State::default, view)
    })
    .init(move || state);
    app.run();
}

/// `?demo` `?hero` `?beat` `?seed=NNN` — deterministic launches for
/// screenshots and testing; the app itself needs none of them.
#[cfg(target_arch = "wasm32")]
fn apply_url_args(s: &mut State) {
    let Some(window) = web_sys::window() else { return };
    let search = window.location().search().unwrap_or_default();
    for pair in search.trim_start_matches('?').split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        match k {
            "hero" => seed_demo(s, true),
            "demo" => seed_demo(s, false),
            "ice" => seed_ice(s),
            "beat" => s.beat = true,
            "seed" => s.seed = v.parse().unwrap_or(0),
            _ => {}
        }
    }
}
