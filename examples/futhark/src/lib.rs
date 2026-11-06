//! Futhark — the Elder Futhark, hewn in fire.
//!
//! A Rune app: all 24 rune glyphs are drawn from scratch as animated vector
//! strokes (Rune's `Shape` polylines) chiselled into stone slabs — a dark
//! undercut groove with a molten core, a spark at the chisel tip while it
//! cuts. The hall around them glows: magma veins in the floor, embers rising,
//! torch-flicker on every lit edge. Browse the alphabet, tap a rune to re-cut
//! it, or cast three runes for a reading. Runs native; on wasm it mounts in
//! a WebGPU canvas.

use rune::tok::TextStyle;
use rune::{canvas, col, el, row, text, AppSpec, Color, Cx, El, LineCap, ScrollSpec, Shadow, Shape, Tx};

/// A rune in the 100 × 140 view box, as strokes of connected points.
struct Rune {
    name: &'static str,
    sound: &'static str,
    word: &'static str,
    meaning: &'static str,
    strokes: &'static [&'static [[f32; 2]]],
}

macro_rules! poly {
    ([$($pt:expr),*]) => { &[$($pt),*] };
}

static RUNES: &[Rune] = &[
    Rune {
        name: "Fehu",
        sound: "f",
        word: "cattle",
        meaning: "wealth · abundance · earned force",
        strokes: &[
            poly!([[50., 12.], [50., 128.]]),
            poly!([[50., 42.], [90., 16.]]),
            poly!([[50., 78.], [90., 52.]]),
        ],
    },
    Rune {
        name: "Uruz",
        sound: "u",
        word: "aurochs",
        meaning: "strength · primal will · vigor",
        strokes: &[poly!([[28., 128.], [28., 18.], [74., 48.], [74., 128.]])],
    },
    Rune {
        name: "Thurisaz",
        sound: "th",
        word: "giant",
        meaning: "thorn · protection · the catalyst",
        strokes: &[
            poly!([[36., 12.], [36., 128.]]),
            poly!([[36., 38.], [78., 60.], [36., 82.]]),
        ],
    },
    Rune {
        name: "Ansuz",
        sound: "a",
        word: "god",
        meaning: "breath · wisdom · the message",
        strokes: &[
            poly!([[36., 12.], [36., 128.]]),
            poly!([[36., 28.], [78., 50.]]),
            poly!([[36., 64.], [78., 86.]]),
        ],
    },
    Rune {
        name: "Raido",
        sound: "r",
        word: "ride",
        meaning: "journey · rhythm · right motion",
        strokes: &[
            poly!([[36., 12.], [36., 128.]]),
            poly!([[36., 18.], [80., 42.], [36., 66.]]),
            poly!([[36., 66.], [80., 128.]]),
        ],
    },
    Rune {
        name: "Kenaz",
        sound: "k",
        word: "torch",
        meaning: "fire · craft · revealed insight",
        strokes: &[poly!([[74., 14.], [30., 70.], [74., 126.]])],
    },
    Rune {
        name: "Gebo",
        sound: "g",
        word: "gift",
        meaning: "exchange · generosity · the bond",
        strokes: &[
            poly!([[26., 16.], [74., 124.]]),
            poly!([[74., 16.], [26., 124.]]),
        ],
    },
    Rune {
        name: "Wunjo",
        sound: "w",
        word: "joy",
        meaning: "harmony · bliss · well met",
        strokes: &[
            poly!([[36., 12.], [36., 128.]]),
            poly!([[36., 18.], [80., 42.], [36., 66.]]),
        ],
    },
    Rune {
        name: "Hagalaz",
        sound: "h",
        word: "hail",
        meaning: "upheaval · patience · raw nature",
        strokes: &[
            poly!([[30., 12.], [30., 128.]]),
            poly!([[70., 12.], [70., 128.]]),
            poly!([[30., 56.], [70., 84.]]),
        ],
    },
    Rune {
        name: "Nauthiz",
        sound: "n",
        word: "need",
        meaning: "constraint · resourcefulness · friction fire",
        strokes: &[
            poly!([[50., 12.], [50., 128.]]),
            poly!([[24., 46.], [76., 90.]]),
        ],
    },
    Rune {
        name: "Isa",
        sound: "i",
        word: "ice",
        meaning: "stillness · the pause · inward",
        strokes: &[poly!([[50., 12.], [50., 128.]])],
    },
    Rune {
        name: "Jera",
        sound: "j",
        word: "year",
        meaning: "harvest · cycles · patient return",
        strokes: &[
            poly!([[68., 12.], [32., 40.], [68., 68.]]),
            poly!([[32., 72.], [68., 100.], [32., 128.]]),
        ],
    },
    Rune {
        name: "Eihwaz",
        sound: "ï",
        word: "yew",
        meaning: "endurance · the axis · life and death",
        strokes: &[
            poly!([[50., 12.], [50., 128.]]),
            poly!([[50., 28.], [88., 10.]]),
            poly!([[50., 112.], [12., 130.]]),
        ],
    },
    Rune {
        name: "Perthro",
        sound: "p",
        word: "lot-cup",
        meaning: "fate · chance · the unrevealed",
        strokes: &[
            poly!([[36., 12.], [36., 128.]]),
            poly!([[78., 14.], [36., 36.], [78., 58.]]),
        ],
    },
    Rune {
        name: "Algiz",
        sound: "z",
        word: "elk",
        meaning: "shelter · vigilance · the higher self",
        strokes: &[
            poly!([[50., 40.], [50., 128.]]),
            poly!([[20., 12.], [50., 40.], [80., 12.]]),
        ],
    },
    Rune {
        name: "Sowilo",
        sound: "s",
        word: "sun",
        meaning: "success · vitality · the whole",
        strokes: &[poly!([[74., 14.], [40., 42.], [70., 72.], [36., 100.]])],
    },
    Rune {
        name: "Tiwaz",
        sound: "t",
        word: "Tyr",
        meaning: "justice · courage · the guiding hand",
        strokes: &[
            poly!([[50., 12.], [50., 128.]]),
            poly!([[24., 34.], [50., 12.], [76., 34.]]),
        ],
    },
    Rune {
        name: "Berkanan",
        sound: "b",
        word: "birch",
        meaning: "renewal · healing · quickening",
        strokes: &[
            poly!([[36., 12.], [36., 128.]]),
            poly!([[36., 18.], [76., 38.], [36., 60.]]),
            poly!([[36., 66.], [76., 88.], [36., 110.]]),
        ],
    },
    Rune {
        name: "Ehwaz",
        sound: "e",
        word: "horse",
        meaning: "trust · partnership · twin momentum",
        strokes: &[
            poly!([[30., 12.], [30., 128.]]),
            poly!([[70., 12.], [70., 128.]]),
            poly!([[30., 26.], [50., 56.], [70., 26.]]),
        ],
    },
    Rune {
        name: "Mannaz",
        sound: "m",
        word: "man",
        meaning: "self · kinship · the pattern shared",
        strokes: &[
            poly!([[30., 12.], [30., 128.]]),
            poly!([[70., 12.], [70., 128.]]),
            poly!([[30., 16.], [70., 64.]]),
            poly!([[70., 16.], [30., 64.]]),
        ],
    },
    Rune {
        name: "Laguz",
        sound: "l",
        word: "water",
        meaning: "intuition · flow · the deep dream",
        strokes: &[
            poly!([[36., 12.], [36., 128.]]),
            poly!([[36., 14.], [80., 46.]]),
        ],
    },
    Rune {
        name: "Ingwaz",
        sound: "ŋ",
        word: "Ing",
        meaning: "seed · gestation · held potential",
        strokes: &[poly!([[50., 26.], [82., 70.], [50., 114.], [18., 70.], [50., 26.]])],
    },
    Rune {
        name: "Dagaz",
        sound: "d",
        word: "day",
        meaning: "dawn · breakthrough · paradox resolved",
        strokes: &[
            poly!([[28., 12.], [28., 128.]]),
            poly!([[72., 12.], [72., 128.]]),
            poly!([[28., 12.], [72., 128.]]),
            poly!([[72., 12.], [28., 128.]]),
        ],
    },
    Rune {
        name: "Othala",
        sound: "o",
        word: "homestead",
        meaning: "heritage · roots · the inherited ground",
        strokes: &[
            poly!([[50., 14.], [80., 50.], [50., 86.], [20., 50.], [50., 14.]]),
            poly!([[35., 68.], [18., 128.]]),
            poly!([[65., 68.], [82., 128.]]),
        ],
    },
];
const VIEW_W: f32 = 100.0;
const VIEW_H: f32 = 140.0;
const CARVE_SECS: f64 = 1.4;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Alphabet,
    Cast,
}

struct Cast {
    runes: [usize; 3],
    start: f64,
    p: [f32; 3],
}

pub struct State {
    mode: Mode,
    selected: usize,
    /// Clock time the current carve started; negative asks the next poll to stamp it.
    carve_start: f64,
    carve: f32,
    cast: Option<Cast>,
    casts: u64,
    clock: f64,
    /// Extra seed for the cast rng (from `?seed=` on web, for deterministic screenshots).
    seed: u64,
    /// `?beat`: present every frame even when static — a capture hook only.
    beat: bool,
}

impl Default for State {
    fn default() -> Self {
        let mut s = State {
            mode: Mode::Alphabet,
            selected: 0,
            carve_start: -1.0,
            carve: 1.0,
            cast: None,
            casts: 0,
            clock: 0.0,
            seed: 0,
            beat: false,
        };
        // Native headless tooling (`--snap`): FUTHARK_SEED / FUTHARK_RUNE / FUTHARK_CAST.
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(v) = std::env::var("FUTHARK_SEED") {
                s.seed = v.parse().unwrap_or(0);
            }
            if let Ok(v) = std::env::var("FUTHARK_RUNE") {
                s.selected = v.parse::<usize>().unwrap_or(0).min(RUNES.len() - 1);
            }
            if std::env::var("FUTHARK_CAST").is_ok() {
                s.mode = Mode::Cast;
                do_cast(&mut s);
            }
        }
        s
    }
}

// ---------------------------------------------------------------------------
// Dwarven palette: basalt halls, old gold, molten rock. No system theme —
// this forge is always dark, lit by the cracks in its floor. `Color` is
// linear-light (Rune's `hex` srgb-decodes), so each constant lists its
// sRGB hex next to the decoded value.
// ---------------------------------------------------------------------------
const BASALT_TOP: Color = Color { r: 0.0110, g: 0.0080, b: 0.0056, a: 1.0 }; // #1b1611
const BASALT_BOT: Color = Color { r: 0.0033, g: 0.0027, b: 0.0018, a: 1.0 }; // #0b0906
const STONE_TOP: Color = Color { r: 0.0343, g: 0.0254, b: 0.0160, a: 1.0 }; // #342c22
const STONE_BOT: Color = Color { r: 0.0145, g: 0.0103, b: 0.0065, a: 1.0 }; // #201a13
const EDGE_HI: Color = Color { r: 0.1022, g: 0.0722, b: 0.0408, a: 1.0 }; // #5a4c39
const EDGE_LO: Color = Color { r: 0.0021, g: 0.0015, b: 0.0012, a: 1.0 }; // #070504
const INK: Color = Color { r: 0.7380, g: 0.6655, b: 0.5584, a: 1.0 }; // #dfd5c5
const INK_DIM: Color = Color { r: 0.3324, g: 0.2746, b: 0.1946, a: 1.0 }; // #9c8f7a
const GOLD: Color = Color { r: 0.8148, g: 0.4508, b: 0.0703, a: 1.0 }; // #e9b34b
const HOT: Color = Color { r: 1.0, g: 0.6939, b: 0.3466, a: 1.0 }; // #ffd99f
const EMBER: Color = Color { r: 1.0, g: 0.2582, b: 0.0241, a: 1.0 }; // #ff8b2b
const LAVA: Color = Color { r: 1.0, g: 0.1000, b: 0.0065, a: 1.0 }; // #ff5913
const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }; // #000000

/// A torch-like flicker in 0.55..=1.0: incommensurate sines, deterministic per seed,
/// so headless snapshots at a fixed `--at` catch it mid-flicker exactly like a live frame.
fn flick(t: f64, seed: f64) -> f32 {
    let v = (t * 6.3 + seed).sin() + 0.55 * (t * 11.7 + seed * 2.1).sin() + 0.3 * (t * 19.3 + seed * 3.7).sin();
    (0.78 + 0.16 * v).clamp(0.55, 1.0) as f32
}

fn frac(x: f64) -> f32 {
    (x - x.floor()) as f32
}

fn hash(i: u64) -> f32 {
    frac((i as f64 * 127.1).sin() * 43758.5453)
}

/// Magma veins in the hall floor, authored in a 440 × 800 box that stretches
/// to the window (Shape `view_box`); edges sit just outside it.
static CRACKS: &[&[[f32; 2]]] = &[
    poly!([[-12., 766.], [64., 742.], [128., 760.], [196., 734.], [262., 752.], [338., 726.], [404., 746.], [452., 734.]]),
    poly!([[128., 760.], [110., 698.], [132., 646.], [114., 588.]]),
    poly!([[262., 752.], [282., 696.], [266., 650.], [288., 602.]]),
    poly!([[-8., 306.], [44., 322.], [78., 302.], [116., 324.]]),
    poly!([[452., 522.], [404., 506.], [372., 528.], [334., 510.]]),
    poly!([[36., 812.], [58., 704.], [42., 624.]]),
    poly!([[408., 812.], [392., 724.], [414., 658.], [398., 602.]]),
    poly!([[196., 734.], [210., 688.]]),
];

/// The whole ambient scene behind the UI: glowing veins in the floor and a field
/// of embers drifting up through the hall.
fn forge_shape(t: f64) -> Shape {
    let mut s = Shape::new().view_box(440.0, 800.0);
    // Heat pool along the floor.
    for k in 0..3 {
        let y = 772.0 + k as f32 * 12.0;
        s = s.line([-24.0, y], [464.0, y + 4.0], 30.0 - k as f32 * 8.0, LineCap::Round, LAVA.alpha(0.035 + 0.02 * k as f32));
    }
    for (i, crack) in CRACKS.iter().enumerate() {
        let g = flick(t, i as f64 * 7.0);
        for w in crack.windows(2) {
            s = s.line(w[0], w[1], 11.0, LineCap::Round, LAVA.alpha(0.09 * g));
            s = s.line(w[0], w[1], 4.0, LineCap::Round, LAVA.alpha(0.26 * g));
            s = s.line(w[0], w[1], 1.5, LineCap::Round, EMBER.alpha(0.72 * g));
            s = s.line([w[0][0] + 0.4, w[0][1] + 0.6], [w[1][0] + 0.4, w[1][1] + 0.6], 0.6, LineCap::Round, HOT.alpha(0.45 * g));
        }
    }
    // Embers rising off the veins: hash-seeded lanes, sway, fade toward the ceiling.
    for i in 0..34u64 {
        let x0 = hash(i * 3 + 1) * 440.0;
        let speed = 26.0 + hash(i * 3 + 2) * 42.0;
        let phase = hash(i * 3 + 3) * 880.0;
        let y = 816.0 - ((t * speed as f64 + phase as f64) % 900.0) as f32;
        let x = x0 + (t * 1.6 + i as f64 * 2.4).sin() as f32 * 7.0;
        let a = (y / 800.0).clamp(0.0, 1.0) * flick(t, i as f64);
        let size = 0.9 + hash(i * 7 + 5) * 1.7;
        s = s.line([x, y], [x, y + size * 2.4], size, LineCap::Round, EMBER.alpha(0.5 * a));
        if hash(i * 11 + 7) > 0.62 {
            s = s.line([x, y], [x, y + size * 1.1], size * 0.55, LineCap::Round, HOT.alpha(0.75 * a));
        }
    }
    s
}

fn total_len(r: &Rune) -> f32 {
    let mut t = 0.0;
    for stroke in r.strokes {
        for w in stroke.windows(2) {
            t += ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
        }
    }
    t
}

/// A rune chiselled into stone, revealed up to fraction `p` of its stroke length.
/// While carving the uncut ghost shows as faint hairlines; the cut itself is a
/// dark undercut groove with a molten core, and a spark burns at the chisel tip.
/// `ink`/`core` pick cold-carved or forge-lit; `heat` pulses the molten glow.
fn carve_shape(
    r: &Rune,
    p: f32,
    width: f32,
    depth: f32,
    ink: Color,
    core: Option<Color>,
    ghost: Option<Color>,
    heat: f32,
) -> Shape {
    let mut s = Shape::new().view_box(VIEW_W, VIEW_H);
    if p < 1.0 {
        if let Some(g) = ghost {
            for stroke in r.strokes {
                for w in stroke.windows(2) {
                    s = s.line(w[0], w[1], width * 0.4, LineCap::Round, g);
                }
            }
        }
    }
    let target = p.clamp(0.0, 1.0) * total_len(r);
    let mut acc = 0.0f32;
    let mut tip: Option<[f32; 2]> = None;
    let mut segs: Vec<([f32; 2], [f32; 2])> = Vec::new();
    'strokes: for stroke in r.strokes {
        for w in stroke.windows(2) {
            let dx = w[1][0] - w[0][0];
            let dy = w[1][1] - w[0][1];
            let l = (dx * dx + dy * dy).sqrt();
            if acc + l <= target {
                segs.push((w[0], w[1]));
                acc += l;
            } else {
                let f = ((target - acc) / l).clamp(0.0, 1.0);
                if f > 0.0 {
                    let q = [w[0][0] + dx * f, w[0][1] + dy * f];
                    segs.push((w[0], q));
                    tip = Some(q);
                } else {
                    tip = Some(w[0]);
                }
                break 'strokes;
            }
        }
    }
    // The groove: an offset dark cut, wider than the light it holds.
    let off = 1.0 * depth;
    for (a, b) in &segs {
        s = s.line([a[0] + off, a[1] + off * 1.4], [b[0] + off, b[1] + off * 1.4], width * 1.7, LineCap::Round, BLACK.alpha(0.42 * depth.max(0.35)));
    }
    // The cut itself, with a hotter filament for forge-lit runes.
    for (a, b) in &segs {
        s = s.line(*a, *b, width, LineCap::Round, ink);
        if let Some(c) = core {
            s = s.line(*a, *b, width * 0.42, LineCap::Round, c);
        }
    }
    // Chisel spark: a star of light at the point of the cut while it runs.
    if p < 1.0 {
        if let Some(t) = tip {
            let g = heat;
            for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                s = s.line(t, [t[0] + dx * 7.5, t[1] + dy * 7.5], 1.1, LineCap::Round, HOT.alpha(0.75 * g));
            }
            s = s.line([t[0] - 4.0, t[1]], [t[0] + 4.0, t[1]], 2.2, LineCap::Round, EMBER.alpha(0.85));
            s = s.line([t[0], t[1] - 4.0], [t[0], t[1] + 4.0], 2.2, LineCap::Round, EMBER.alpha(0.85));
        }
    }
    s
}

fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

fn retouch(s: &mut State) {
    s.carve = 0.0;
    s.carve_start = -1.0;
}

fn do_cast(s: &mut State) {
    let mut x = (((s.clock * 1_000_000.0) as i64) as u64)
        .wrapping_add(s.casts.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .swap_bytes()
        ^ s.seed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    if x == 0 {
        x = 0x2545_F491_4F6C_DD1D;
    }
    let mut next = move || -> usize {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        (x >> 40) as usize % RUNES.len()
    };
    let mut picked: [usize; 3] = [usize::MAX; 3];
    for i in 0..3 {
        loop {
            let r = next();
            if !picked[..i].contains(&r) {
                picked[i] = r;
                break;
            }
        }
    }
    s.casts += 1;
    s.cast = Some(Cast { runes: picked, start: -1.0, p: [0.0; 3] });
}

fn poll(s: &mut State, t: f64, tx: &mut Tx) -> bool {
    s.clock = t;
    let mut animating = false;
    if s.carve < 1.0 {
        if s.carve_start < 0.0 {
            s.carve_start = t;
        }
        let p = ((t - s.carve_start) / CARVE_SECS).clamp(0.0, 1.0);
        s.carve = p as f32;
        animating |= p < 1.0;
    }
    if let Some(cast) = &mut s.cast {
        if cast.start < 0.0 {
            cast.start = t;
        }
        let mut done = true;
        for (i, p) in cast.p.iter_mut().enumerate() {
            let v = ((t - cast.start - i as f64 * 0.45) / CARVE_SECS).clamp(0.0, 1.0);
            *p = ease_out(v as f32);
            done &= v >= 1.0;
        }
        animating |= !done;
    }
    // The hall is never quite still: veins pulse and embers climb, so the frame
    // clock keeps a slow ambient beat even at rest (the runtime only requests
    // frames while work is pending, so the app schedules its own next beat).
    tx.wake_after(std::time::Duration::from_millis(if animating || s.beat { 16 } else { 30 }));
    true
}

// ---------------------------------------------------------------------------
// The masonry: one helper paints every panel as a chiselled slab — a stone
// gradient, bevelled edges (light above, shadow below), an inset lip and a
// drop shadow into the hall's dark.
// ---------------------------------------------------------------------------
fn slab(e: El) -> El {
    e.gradient(STONE_TOP, STONE_BOT)
        .border_gradient(1.5, EDGE_HI.alpha(0.8), EDGE_LO.alpha(0.95))
        .box_shadow(Shadow { x: 0.0, y: 2.0, blur: 3.0, spread: -0.5, color: BLACK.alpha(0.5), inset: true })
        .box_shadow(Shadow { x: 0.0, y: -1.5, blur: 1.5, spread: -1.0, color: EDGE_HI.alpha(0.25), inset: true })
        .shadow(9.0, 22.0, BLACK.alpha(0.35))
}

/// Letters chiselled into the panel face: an ember rim peeking over the top edge,
/// a dark recessed copy under it, and the pale stone face on top — stacked by
/// negative margins so all three share one line box.
fn engrave(label: &str, size: f32, weight: f32, track: f32, ink: Color) -> El {
    let lh = size * 1.32;
    let font = || TextStyle::new(size, weight).lh(lh).tracking(track);
    col()
        .gap(0.0)
        .child(text(label).font(font()).color(EMBER.alpha(0.32)).mt(0.0))
        .child(text(label).font(font()).color(BLACK.alpha(0.8)).mt(-lh + 2.2))
        .child(text(label).font(font()).color(ink).mt(-lh - 2.2))
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let [top, right, bottom, left] = cx.safe_area();
    let t = s.clock;
    let heat = flick(t, 0.0);
    let heat2 = flick(t, 13.0);

    let header = col()
        .w_full()
        .gap(4.0)
        .child(engrave("FUTHARK", 26.0, 700.0, 6.0, INK))
        .child(
            text("the elder futhark · hewn in fire")
                .font(TextStyle::new(13.0, 400.0).tracking(1.2))
                .color(INK_DIM),
        )
        .child(
            canvas({
                let mut sh = Shape::new().view_box(340.0, 12.0);
                let jag: [[f32; 2]; 9] = [
                    [4., 7.], [48., 4.], [96., 8.], [146., 3.], [196., 8.], [244., 4.], [288., 8.], [336., 5.], [336., 5.],
                ];
                for w in jag.windows(2) {
                    sh = sh.line(w[0], w[1], 6.0, LineCap::Round, LAVA.alpha(0.16 * heat2));
                    sh = sh.line(w[0], w[1], 2.0, LineCap::Round, EMBER.alpha(0.6 * heat2));
                    sh = sh.line([w[0][0], w[0][1] + 0.5], [w[1][0], w[1][1] + 0.5], 0.7, LineCap::Round, HOT.alpha(0.7 * heat2));
                }
                sh
            })
            .w_full()
            .h(12.0),
        );

    let tab = |label: &'static str, key: &'static str, active: bool, on_press: fn(&mut State)| {
        let mut b = slab(
            el()
                .key(key)
                .h(42.0)
                .px(16.0)
                .center()
                .radius(11.0)
                .press_scale(0.94),
        );
        if active {
            b = b
                .border(1.5, EMBER.alpha(0.55 * heat))
                .box_shadow(Shadow { x: 0.0, y: 0.0, blur: 14.0, spread: 1.0, color: EMBER.alpha(0.22 * heat), inset: true });
        }
        b.child(engrave(label, 13.0, 650.0, 1.5, if active { GOLD } else { INK })).on_press(on_press)
    };
    const TO_ALPHA: fn(&mut State) = |s: &mut State| s.mode = Mode::Alphabet;
    const TO_CAST: fn(&mut State) = |s: &mut State| s.mode = Mode::Cast;
    let tabs = row()
        .w_full()
        .gap(10.0)
        .child(tab("ALPHABET", "tab-alphabet", s.mode == Mode::Alphabet, TO_ALPHA))
        .child(tab("THE CAST", "tab-cast", s.mode == Mode::Cast, TO_CAST));

    let body = match s.mode {
        Mode::Alphabet => {
            let r = &RUNES[s.selected];
            let glyph = carve_shape(
                r,
                ease_out(s.carve),
                4.0,
                1.0,
                LAVA.alpha(0.55 + 0.4 * heat),
                Some(HOT.alpha(0.3 + 0.55 * heat)),
                Some(EDGE_HI.alpha(0.16)),
                heat,
            );
            let stage = el()
                .key("stage")
                .size(152.0, 198.0)
                .center()
                .radial_gradient(
                    [0.5, 0.52],
                    100.0,
                    [(0.0, LAVA.alpha(0.30 * heat)), (0.5, LAVA.alpha(0.10 * heat)), (1.0, LAVA.alpha(0.0))],
                )
                .on_press(retouch)
                .child(canvas(glyph).size(116.0, 162.0));
            let info = col()
                .grow(1.0)
                .min_w(0.0)
                .gap(8.0)
                .child(engrave(r.name, 19.0, 650.0, 1.0, GOLD))
                .child(
                    el()
                        .px(10.0)
                        .py(5.0)
                        .radius(7.0)
                        .h(26.0)
                        .center()
                        .fill(BLACK.alpha(0.35))
                        .border(1.0, GOLD.alpha(0.35))
                        .child(
                            text(format!("{} · “{}”", r.sound, r.word))
                                .font(TextStyle::new(12.5, 500.0).tracking(1.0))
                                .color(GOLD),
                        ),
                )
                .child(text(r.meaning).w_full().wrap().font(TextStyle::new(13.0, 400.0).lh(18.0)).color(INK_DIM));
            let detail = slab(
                el()
                    .key("detail")
                    .w_full()
                    .radius(16.0)
                    .corners([16.0, 16.0, 9.0, 9.0])
                    .padding(14.0)
                    .child(row().w_full().gap(10.0).center_y().child(stage).child(info)),
            );

            let mut grid = col().w_full().gap(10.0).py(2.0);
            for row_i in 0..RUNES.len().div_ceil(3) {
                let mut line = row().w_full().gap(10.0);
                for col_i in 0..3 {
                    let idx = row_i * 3 + col_i;
                    if idx >= RUNES.len() {
                        break;
                    }
                    let rune = &RUNES[idx];
                    let sel = s.selected == idx;
                    let mut cell = slab(
                        col()
                            .key(format!("cell-{idx}"))
                            .grow(1.0)
                            .h(88.0)
                            .radius(12.0)
                            .center()
                            .gap(2.0)
                            .press_scale(0.93),
                    );
                    if sel {
                        cell = cell
                            .border(1.5, EMBER.alpha(0.5 * heat))
                            .box_shadow(Shadow { x: 0.0, y: 0.0, blur: 12.0, spread: 1.0, color: EMBER.alpha(0.16 * heat), inset: true });
                    }
                    cell = cell
                        .on_press(move |s: &mut State| {
                            s.selected = idx;
                            retouch(s);
                        })
                        .child(
                            canvas(carve_shape(
                                rune,
                                1.0,
                                5.5,
                                if sel { 0.9 } else { 0.45 },
                                if sel { LAVA.alpha(0.55 + 0.4 * heat) } else { INK.alpha(0.88) },
                                sel.then(|| HOT.alpha(0.35 + 0.5 * heat)),
                                None,
                                heat,
                            ))
                            .size(40.0, 56.0),
                        )
                        .child(
                            text(rune.name)
                                .font(TextStyle::new(11.5, 500.0).tracking(0.6))
                                .color(if sel { GOLD } else { INK_DIM }),
                        );
                    line = line.child(cell);
                }
                grid = grid.child(line);
            }
            let scroll = col()
                .key("grid")
                .w_full()
                .grow(1.0)
                .shrink()
                .min_h(0.0)
                .scroll(ScrollSpec { stick_bottom: false })
                .scroll_edge_fade(16.0)
                .child(grid);

            col().w_full().grow(1.0).min_h(0.0).gap(14.0).child(detail).child(scroll)
        }
        Mode::Cast => {
            let slots = ["PAST", "PRESENT", "FUTURE"];
            let mut cards = row().w_full().gap(10.0).center_y();
            for (i, slot) in slots.iter().enumerate() {
                let mut card = slab(
                    col()
                        .key(format!("card-{i}"))
                        .grow(1.0)
                        .min_w(0.0)
                        .radius(13.0)
                        .corners([13.0, 13.0, 7.0, 7.0])
                        .padding(10.0)
                        .gap(6.0),
                )
                .border(1.5, EMBER.alpha(0.35 * heat))
                .box_shadow(Shadow { x: 0.0, y: 0.0, blur: 12.0, spread: 1.0, color: LAVA.alpha(0.14 * heat), inset: true })
                .child(text(*slot).font(TextStyle::new(11.0, 600.0).tracking(2.0)).color(GOLD.alpha(0.85)));
                let (glyph, name, meaning) = match &s.cast {
                    Some(cast) => {
                        let rune = &RUNES[cast.runes[i]];
                        (
                            carve_shape(
                                rune,
                                cast.p[i],
                                5.0,
                                0.9,
                                LAVA.alpha(0.55 + 0.4 * heat),
                                Some(HOT.alpha(0.3 + 0.55 * heat)),
                                Some(EDGE_HI.alpha(0.16)),
                                heat,
                            ),
                            rune.name,
                            rune.meaning,
                        )
                    }
                    None => (Shape::new().view_box(VIEW_W, VIEW_H), "", ""),
                };
                card = card
                    .child(canvas(glyph).key(format!("cast-{i}")).size(58.0, 81.0))
                    .child(text(name).w_full().wrap().font(TextStyle::new(13.0, 650.0).tracking(0.6)).color(GOLD));
                if !meaning.is_empty() {
                    card = card.child(
                        text(meaning)
                            .w_full()
                            .wrap()
                            .font(TextStyle::new(12.0, 400.0).lh(16.0))
                            .color(INK_DIM),
                    );
                }
                cards = cards.child(card);
            }
            let button = el()
                .key("cast-btn")
                .w_full()
                .h(54.0)
                .center()
                .radius(13.0)
                .corners([13.0, 13.0, 8.0, 8.0])
                .press_scale(0.97)
                .gradient(Color { r: 0.0452, g: 0.0145, b: 0.0052, a: 1.0 }, Color { r: 0.0164, g: 0.0056, b: 0.0027, a: 1.0 }) // #3c2010 → #231109
                .border_gradient(2.0, EMBER.alpha(0.8 * heat), LAVA.alpha(0.35))
                .box_shadow(Shadow { x: 0.0, y: -1.0, blur: 16.0, spread: 1.0, color: LAVA.alpha(0.30 * heat), inset: true })
                .box_shadow(Shadow { x: 0.0, y: 4.0, blur: 26.0, spread: 0.0, color: LAVA.alpha(0.30 * heat2), inset: false })
                .child(engrave(
                    if s.cast.is_some() { "STRIKE AGAIN" } else { "STRIKE THE RUNES" },
                    14.0,
                    700.0,
                    2.5,
                    GOLD,
                ))
                .on_press(do_cast);
            let hint = text("three of the twenty-four, hewn from the mountain — past, present, future")
                .w_full()
                .wrap()
                .font(TextStyle::new(12.5, 400.0).lh(17.0))
                .color(INK_DIM.alpha(0.8));
            col().w_full().grow(1.0).min_h(0.0).gap(14.0).center().child(cards).child(button).child(hint)
        }
    };

    let inner = col()
        .w_full()
        .max_w(440.0)
        .h_full()
        .padding(16.0)
        .gap(14.0)
        .pt(16.0 + top)
        .pr(16.0 + right)
        .pb(16.0 + bottom)
        .pl(16.0 + left)
        .child(header)
        .child(tabs)
        .child(body);

    // The hall itself: basalt fading to soot, magma veins and rising embers
    // painted as the root element's own visual, with the UI floating on it.
    el()
        .w_full()
        .h_full()
        .center()
        .linear_gradient(std::f32::consts::PI, [(0.0, BASALT_TOP), (1.0, BASALT_BOT)])
        .shape(forge_shape(t))
        .child(inner)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn start() {
    #[allow(unused_mut)] // mutated only on wasm (URL args)
    let mut state = State::default();
    #[cfg(target_arch = "wasm32")]
    apply_url_args(&mut state);
    let app = rune::App::new(AppSpec {
        poll: Some(poll),
        ..AppSpec::new("Futhark", (390.0, 760.0), State::default, view)
    })
    .init(move || state);
    app.run();
}

/// `?mode=cast` `?seed=NNN` `?rune=NN` `?auto` — deterministic launches for
/// screenshots and testing; the app itself needs none of them.
#[cfg(target_arch = "wasm32")]
fn apply_url_args(s: &mut State) {
    let Some(window) = web_sys::window() else { return };
    let search = window.location().search().unwrap_or_default();
    for pair in search.trim_start_matches('?').split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        match k {
            "mode" if v == "cast" => s.mode = Mode::Cast,
            "seed" => s.seed = v.parse().unwrap_or(0),
            "rune" => s.selected = v.parse::<usize>().unwrap_or(0).min(RUNES.len() - 1),
            "auto" => do_cast(s),
            "beat" => s.beat = true,
            "retouch" => retouch(s),
            _ => {}
        }
    }
}
