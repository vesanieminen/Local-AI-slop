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
use rune::{
    canvas, col, el, row, text, text_input, AppSpec, Color, Cx, El, LineCap, Pixels, ScrollSpec,
    Shadow, Shape, SurfaceFit, TextEdit, Tx,
};

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
    Ledger,
}

struct Cast {
    runes: [usize; 3],
    start: f64,
    p: [f32; 3],
}

/// A task carved on the ledger. An untouched task slides toward the magma
/// line as its heat bar fills, then cracks, glows and crumbles (`melt` is
/// the 0..1 burn past its deadline). A done task seals in gold, then fades
/// into the wall.
struct Task {
    label: String,
    born: f64,
    /// 0..=1 toward the lava; frozen once done.
    sink: f32,
    done: bool,
    done_at: f64,
}

const TASK_LIFE: f64 = 90.0;
const BURN_SECS: f64 = 0.9;
const SEAL_SECS: f64 = 7.0;
const MAX_TASKS: usize = 7;
const SINK_DROP: f32 = 22.0;

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
    tasks: Vec<Task>,
    draft: TextEdit,
    /// Seamless basalt faces, shared by every slab in the hall.
    rock_a: Pixels,
    rock_b: Pixels,
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
            tasks: Vec::new(),
            draft: TextEdit::new(""),
            rock_a: make_rock(0xA17C3),
            rock_b: make_rock(0x5E3A9),
        };
        // Native headless tooling (`--snap`): FUTHARK_SEED / FUTHARK_RUNE / FUTHARK_CAST / FUTHARK_TASKS.
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
            if std::env::var("FUTHARK_TASKS").is_ok() {
                s.mode = Mode::Ledger;
                seed_tasks(&mut s, true);
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
const BASALT_TOP: Color = Color { r: 0.0082, g: 0.0082, b: 0.0103, a: 1.0 }; // #16161a
const BASALT_BOT: Color = Color { r: 0.0030, g: 0.0030, b: 0.0037, a: 1.0 }; // #0a0a0c
const EDGE_HI: Color = Color { r: 0.1471, g: 0.1471, b: 0.1213, a: 1.0 }; // #6b6a63
const EDGE_LO: Color = Color { r: 0.0012, g: 0.0012, b: 0.0014, a: 1.0 }; // #060607
const INK: Color = Color { r: 0.7380, g: 0.6655, b: 0.5584, a: 1.0 }; // #dfd5c5
const INK_DIM: Color = Color { r: 0.3324, g: 0.2746, b: 0.1946, a: 1.0 }; // #9c8f7a
const GOLD: Color = Color { r: 0.8148, g: 0.4508, b: 0.0703, a: 1.0 }; // #e9b34b
const HOT: Color = Color { r: 1.0, g: 0.6939, b: 0.3466, a: 1.0 }; // #ffd99f
const EMBER: Color = Color { r: 1.0, g: 0.2582, b: 0.0241, a: 1.0 }; // #ff8b2b
const LAVA: Color = Color { r: 1.0, g: 0.1000, b: 0.0065, a: 1.0 }; // #ff5913
const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }; // #000000

// ---------------------------------------------------------------------------
// Procedural basalt: a seamless height field whose normals are baked into
// Lambert shading from a fixed top-left light — a displacement-map-style bump
// map rendered once into a `Pixels` raster, then used as the face of every
// slab in the hall.
// ---------------------------------------------------------------------------
fn vhash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA6B)
        ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3131);
    h ^= h >> 12;
    h as f32 / u32::MAX as f32
}

/// Value noise on a wrapping `period × period` lattice, so the texture tiles seamlessly.
fn vnoise(x: f32, y: f32, period: i32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (xf, yf) = (x - xi, y - yi);
    let w = |a: f32| ((a as i32).rem_euclid(period)) as u32;
    let (x0, x1) = (w(xi), w(xi + 1.0));
    let (y0, y1) = (w(yi), w(yi + 1.0));
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let a = vhash(x0 as i32, y0 as i32, seed);
    let b = vhash(x1 as i32, y0 as i32, seed);
    let c = vhash(x0 as i32, y1 as i32, seed);
    let d = vhash(x1 as i32, y1 as i32, seed);
    (a * (1.0 - u) + b * u) * (1.0 - v) + (c * (1.0 - u) + d * u) * v
}

/// Fractal sum over `cu,cv` in `0..1`, lattices from `1 << base` up, all wrapping.
fn fbm(cu: f32, cv: f32, octaves: u32, base: u32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut norm) = (0.0f32, 1.0f32, 0.0f32);
    for o in 0..octaves {
        let f = (1u32 << (base + o).min(12)) as f32;
        sum += amp * vnoise(cu * f, cv * f, f as i32, seed.wrapping_add(o * 7919));
        norm += amp;
        amp *= 0.55;
    }
    sum / norm
}

fn make_rock(seed: u32) -> Pixels {
    const S: usize = 256;
    let h = |x: usize, y: usize| -> f32 {
        0.5 + 0.5 * fbm((x as f32 + 0.5) / S as f32, (y as f32 + 0.5) / S as f32, 5, 3, seed)
    };
    let mut rgba = vec![0u8; S * S * 4];
    // Light from the upper left; the surface normal comes from the height gradient.
    let (lx, ly, lz) = (-0.52f32, -0.62, 0.58);
    let ln = (lx * lx + ly * ly + lz * lz).sqrt();
    let (lx, ly, lz) = (lx / ln, ly / ln, lz / ln);
    for y in 0..S {
        for x in 0..S {
            let dhx = h((x + 1) % S, y) - h((x + S - 1) % S, y);
            let dhy = h(x, (y + 1) % S) - h(x, (y + S - 1) % S);
            let (nx, ny, nz) = (-dhx * 3.1, -dhy * 3.1, 1.0); // bump strength
            let nn = (nx * nx + ny * ny + nz * nz).sqrt();
            let diff = ((nx / nn) * lx + (ny / nn) * ly + (nz / nn) * lz).max(0.0);
            let shade = 0.50 + 0.80 * diff + 0.16 * diff.powi(9); // a hint of specular
            let tone = fbm(x as f32 / S as f32, y as f32 / S as f32, 3, 2, seed.wrapping_add(31));
            let (mut r, mut g, mut b) = (47.0 + 17.0 * tone, 45.0 + 16.0 * tone, 41.0 + 14.0 * tone);
            r *= shade;
            g *= shade;
            b *= shade;
            // Mineral flecks catch the light; pores sit deep.
            let sp = vhash(x as i32, y as i32, seed.wrapping_add(77));
            if sp > 0.9972 {
                r += 58.0;
                g += 48.0;
                b += 28.0;
            } else if sp < 0.0038 {
                r *= 0.52;
                g *= 0.52;
                b *= 0.52;
            }
            let i = (y * S + x) * 4;
            rgba[i] = r.clamp(0.0, 255.0) as u8;
            rgba[i + 1] = g.clamp(0.0, 255.0) as u8;
            rgba[i + 2] = b.clamp(0.0, 255.0) as u8;
            rgba[i + 3] = 255;
        }
    }
    let p = Pixels::new();
    p.set(S as u32, S as u32, rgba);
    p
}

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

/// Magma rivers that flow across the hall floor, in the same 440 × 800 box.
static RIVERS: &[&[[f32; 2]]] = &[
    poly!([[-14., 688.], [58., 676.], [118., 694.], [184., 672.], [250., 690.], [316., 668.], [382., 686.], [452., 674.]]),
    poly!([[-14., 590.], [66., 604.], [148., 584.], [232., 606.], [314., 586.], [398., 608.], [452., 598.]]),
    poly!([[40., 812.], [78., 758.], [64., 698.], [116., 656.], [104., 596.], [130., 540.]]),
    poly!([[398., 812.], [368., 764.], [384., 706.], [338., 668.], [352., 616.]]),
];

fn polyline_len(path: &[[f32; 2]]) -> f32 {
    path.windows(2).map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt()).sum()
}

/// The point `frac` (0 to 1) of the way along `path` by arc length.
fn polyline_at(path: &[[f32; 2]], frac: f32) -> [f32; 2] {
    let target = frac.clamp(0.0, 1.0) * polyline_len(path);
    let mut acc = 0.0f32;
    for w in path.windows(2) {
        let dx = w[1][0] - w[0][0];
        let dy = w[1][1] - w[0][1];
        let l = (dx * dx + dy * dy).sqrt();
        if acc + l >= target {
            let f = if l == 0.0 { 0.0 } else { ((target - acc) / l).clamp(0.0, 1.0) };
            return [w[0][0] + dx * f, w[0][1] + dy * f];
        }
        acc += l;
    }
    *path.last().unwrap()
}

/// The sub-polyline of `path` from arc-length `from`, spanning `len`.
fn sub_polyline(path: &[[f32; 2]], from: f32, len: f32) -> Vec<[f32; 2]> {
    let mut out = Vec::new();
    let mut acc = 0.0f32;
    for w in path.windows(2) {
        let dx = w[1][0] - w[0][0];
        let dy = w[1][1] - w[0][1];
        let l = (dx * dx + dy * dy).sqrt();
        if acc + l < from {
            acc += l;
            continue;
        }
        let f0 = ((from - acc) / l).clamp(0.0, 1.0);
        if out.is_empty() {
            out.push([w[0][0] + dx * f0, w[0][1] + dy * f0]);
        }
        let f1 = ((from + len - acc) / l).clamp(0.0, 1.0);
        if f1 < 1.0 {
            out.push([w[0][0] + dx * f1, w[0][1] + dy * f1]);
            return out;
        }
        out.push(w[1]);
        acc += l;
    }
    out
}

/// A licking flame at (x, y): three sampled quadratic teardrops with sway and
/// height driven by the clock, a hot core inside each tongue.
fn flame(s: Shape, x: f32, y: f32, h: f32, t: f64, seed: u64) -> Shape {
    let mut s = s;
    let g = flick(t, seed as f64);
    s = s.line([x - 16.0, y], [x + 16.0, y], 8.0, LineCap::Round, LAVA.alpha(0.30 * g));
    for i in 0..3u32 {
        let ph = seed as f64 * 2.1 + i as f64 * 1.7;
        let hh = h * (0.55 + 0.45 * g) * (0.7 + 0.3 * hash(seed * 3 + i as u64));
        let sway = ((t * 5.5 + ph).sin() + 0.4 * (t * 11.0 + ph * 2.0).sin()) as f32 * 6.0;
        let bx = x - 6.0 + 3.0 * i as f32;
        let (tx, ty) = (bx + sway, y - hh);
        let (cx, cy) = (bx + sway * 0.25, y - hh * 0.55);
        let mut left = Vec::new();
        let mut right = Vec::new();
        for k in 0..=6u32 {
            let u = k as f32 / 6.0;
            let (iu, du) = (1.0 - u, u);
            let px = iu * iu * bx + 2.0 * iu * du * cx + du * du * tx;
            let py = iu * iu * y + 2.0 * iu * du * cy + du * du * ty;
            let w = 5.2 * (1.0 - u) + 0.3;
            left.push([px - w, py]);
            right.push([px + w, py]);
        }
        right.reverse();
        let tongue = [left, right].concat();
        s = s.stroke([[bx, y], [cx, cy], [tx, ty]].to_vec(), 9.0, LAVA.alpha(0.16 * g));
        s = s.fill(tongue, EMBER.alpha(0.50 * g));
        // inner core: same curve, half the width
        let mut left = Vec::new();
        let mut right = Vec::new();
        for k in 1..=6u32 {
            let u = k as f32 / 6.0;
            let (iu, du) = (1.0 - u, u);
            let px = iu * iu * bx + 2.0 * iu * du * cx + du * du * tx;
            let py = iu * iu * y + 2.0 * iu * du * cy + du * du * ty;
            let w = 2.0 * (1.0 - u);
            left.push([px - w, py]);
            right.push([px + w, py]);
        }
        right.reverse();
        s = s.fill([left, right].concat(), HOT.alpha(0.72 * g));
    }
    s
}

/// The whole ambient scene behind the UI: a flagstone floor with magma rivers
/// flowing across it, fire licking from the vents, glowing cracks and a thick
/// field of embers drifting up the hall.
fn forge_shape(t: f64) -> Shape {
    let mut s = Shape::new().view_box(440.0, 800.0);
    // Flagstones: small jittered quads in crooked courses, each a slightly
    // different grey with a lit top edge — a floor, not a pile of cards.
    let (cols, rows) = (8u32, 13u32);
    let (tw, th) = (440.0 / cols as f32, 800.0 / (rows + 5) as f32);
    let y0 = th * 3.2;
    for cy in 0..rows {
        for cx in 0..cols {
            let idx = (cy * cols + cx) as u64;
            let jx = |k: u64| (hash(idx * 4 + k) - 0.5) * 11.0;
            let jy = |k: u64| (hash(idx * 4 + 40 + k) - 0.5) * 8.0;
            let (x, y) = (cx as f32 * tw, y0 + cy as f32 * th);
            let quad = [
                [x + jx(0), y + jy(0)],
                [x + tw + jx(1), y + jy(1)],
                [x + tw + jx(2), y + th + jy(2)],
                [x + jx(3), y + th + jy(3)],
            ];
            let v = hash(idx * 9 + 5);
            let stone = Color { r: 0.0082 + 0.0045 * v, g: 0.0078 + 0.0042 * v, b: 0.0071 + 0.0038 * v, a: 1.0 };
            // Thickness: a dark copy sunk below the face, then the face itself.
            let sunk = quad.map(|p| [p[0] + 1.5, p[1] + 5.0]);
            s = s.fill(sunk.to_vec(), BLACK.alpha(0.55));
            s = s.fill(quad.to_vec(), stone);
            // The face catches the hall light along its top edge; chips break it up.
            s = s.line(quad[0], quad[1], 1.0, LineCap::Round, EDGE_HI.alpha(0.09 + 0.07 * v));
            s = s.line(quad[3], quad[2], 1.2, LineCap::Round, BLACK.alpha(0.55));
            if hash(idx * 13 + 7) > 0.55 {
                let ax = quad[0][0] + tw * (0.2 + 0.5 * hash(idx * 3 + 1));
                let ay = quad[0][1] + th * (0.3 + 0.4 * hash(idx * 3 + 2));
                s = s.line([ax, ay], [ax + 12.0 + 18.0 * hash(idx * 5 + 3), ay + 6.0 * (hash(idx * 5 + 4) - 0.5)], 0.7, LineCap::Round, EDGE_HI.alpha(0.05 + 0.05 * v));
            }
            // Some slabs sit cracked: one hot edge, others just dark seams.
            if hash(idx * 17 + 3) > 0.8 {
                let g = flick(t, idx as f64 * 3.0);
                s = s.line(quad[2], quad[3], 1.6, LineCap::Round, LAVA.alpha(0.30 * g));
                s = s.line(quad[2], quad[3], 4.5, LineCap::Round, LAVA.alpha(0.07 * g));
            }
            if hash(idx * 23 + 11) > 0.88 {
                let g = flick(t, idx as f64 * 3.0 + 1.7);
                s = s.line(quad[1], quad[2], 1.2, LineCap::Round, EMBER.alpha(0.22 * g));
            }
        }
    }
    // The magma rivers: wide molten channels with heat bleeding onto the stone,
    // dark crust rafts riding downstream, and a glowing seam burning through
    // the gaps between the plates.
    for (ri, river) in RIVERS.iter().enumerate() {
        let g = flick(t, ri as f64 * 11.0);
        let len = polyline_len(river);
        s = s.stroke(river.to_vec(), 64.0, LAVA.alpha(0.06 + 0.02 * g)); // heat bleed
        s = s.stroke(river.to_vec(), 40.0, LAVA.alpha(0.45 + 0.12 * g)); // molten body
        s = s.stroke(river.to_vec(), 20.0, EMBER.alpha(0.30 + 0.10 * g)); // molten core
        // Dark crust rafts drifting downstream, breaking the glow into plates.
        for k in 0..6u64 {
            let pos = frac(t * (0.02 + 0.006 * ri as f64) + k as f64 / 6.0 + ri as f64 * 0.31) * len;
            let raft = sub_polyline(river, pos, 22.0 + 30.0 * hash(ri as u64 * 13 + k));
            s = s.stroke(raft, 17.0, Color { r: 0.006, g: 0.005, b: 0.005, a: 0.72 });
        }
        // The seam burns through between the crust, and hot blobs surge along it.
        s = s.stroke(river.to_vec(), 5.0, EMBER.alpha(0.35 + 0.15 * g));
        s = s.stroke(river.to_vec(), 1.8, HOT.alpha(0.30 + 0.25 * g));
        for k in 0..4u64 {
            let pos = frac(t * (0.045 + 0.011 * ri as f64) + k as f64 / 4.0) * len;
            let dash = sub_polyline(river, pos, 34.0 + 22.0 * hash(ri as u64 * 7 + k));
            s = s.stroke(dash.clone(), 9.0, EMBER.alpha(0.30 * g));
            s = s.stroke(dash, 3.2, HOT.alpha(0.55));
        }
        for k in 0..3u32 {
            let p = polyline_at(river, (k as f32 + 0.5) / 3.0);
            s = flame(s, p[0], p[1], 26.0 + 16.0 * hash(ri as u64 * 17 + k as u64), t, ri as u64 * 100 + k as u64);
        }
    }
    // Wall cracks up high: flicker with a slow pulse crawling along them.
    for (i, crack) in CRACKS.iter().enumerate() {
        let g = flick(t, i as f64 * 7.0);
        for w in crack.windows(2) {
            s = s.line(w[0], w[1], 11.0, LineCap::Round, LAVA.alpha(0.09 * g));
            s = s.line(w[0], w[1], 4.0, LineCap::Round, LAVA.alpha(0.26 * g));
            s = s.line(w[0], w[1], 1.5, LineCap::Round, EMBER.alpha(0.72 * g));
            s = s.line([w[0][0] + 0.4, w[0][1] + 0.6], [w[1][0] + 0.4, w[1][1] + 0.6], 0.6, LineCap::Round, HOT.alpha(0.45 * g));
        }
        let len = polyline_len(crack);
        let pos = frac(t * 0.06 + i as f64 / CRACKS.len() as f64) * len;
        let dash = sub_polyline(crack, pos, 26.0);
        s = s.stroke(dash, 1.1, HOT.alpha(0.55 * g));
    }
    // The magma lake along the bottom edge, flowing.
    for k in 0..3 {
        let y = 772.0 + k as f32 * 12.0;
        let g = flick(t, k as f64 * 6.0 + 2.0);
        s = s.line([-24.0, y], [464.0, y + 4.0], 30.0 - k as f32 * 8.0, LineCap::Round, LAVA.alpha(0.05 + 0.03 * k as f32 + 0.02 * g));
        let lake = [[-24.0f32, y], [464.0, y + 4.0]];
        for m in 0..4u64 {
            let pos = frac(t * 0.03 + m as f64 / 4.0 + k as f64 * 0.17) * 488.0;
            let dash = sub_polyline(&lake, pos, 60.0 + 30.0 * hash((k * 9 + m as usize) as u64));
            s = s.stroke(dash, 4.0 - k as f32, EMBER.alpha(0.22 * g));
        }
    }
    for x in [-8.0f32, 96.0, 214.0, 330.0, 442.0] {
        s = flame(s, x, 792.0, 34.0, t, (x as u64) ^ 0x51ED);
    }
    // A thick ember field rising off the hall, some flaring bright.
    for i in 0..58u64 {
        let x0 = hash(i * 3 + 1) * 440.0;
        let speed = 26.0 + hash(i * 3 + 2) * 42.0;
        let phase = hash(i * 3 + 3) * 880.0;
        let y = 816.0 - ((t * speed as f64 + phase as f64) % 900.0) as f32;
        let x = x0 + (t * 1.6 + i as f64 * 2.4).sin() as f32 * 7.0;
        let a = (y / 800.0).clamp(0.0, 1.0) * flick(t, i as f64);
        let size = 0.9 + hash(i * 7 + 5) * 1.7;
        s = s.line([x, y], [x, y + size * 2.4], size, LineCap::Round, EMBER.alpha(0.5 * a));
        if hash(i * 11 + 7) > 0.5 {
            s = s.line([x, y], [x, y + size * 1.1], size * 0.55, LineCap::Round, HOT.alpha(0.75 * a));
        }
        if hash(i * 13 + 9) > 0.9 {
            // a rare flare: a bigger, brighter coal with a tail
            s = s.line([x, y], [x, y + 9.0], 2.6, LineCap::Round, EMBER.alpha(0.85 * a));
            s = s.line([x, y], [x, y + 4.0], 1.3, LineCap::Round, HOT.alpha(0.95 * a));
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

/// Ledger mode: dwarves do not procrastinate; the magma does it for them.
/// `aged` seeds pre-heated tasks for deterministic snapshots.
fn seed_tasks(s: &mut State, aged: bool) {
    const PRESETS: [(&str, f64, bool); 4] = [
        ("re-lit the braziers in the deep hall", 14.0, false),
        ("pay the smith three silver", 4.0, true),
        ("count the silver in shaft nine", 52.0, false),
        ("mend the magma-channel gate", 88.0, false),
    ];
    let t = s.clock;
    for (label, age, done) in PRESETS {
        if s.tasks.len() >= MAX_TASKS {
            break;
        }
        let born = if aged { t - age } else { t };
        let sink = if aged { (age / TASK_LIFE) as f32 } else { 0.0 };
        s.tasks.push(Task { label: label.into(), born, sink, done, done_at: t });
    }
}

fn add_task(s: &mut State) {
    let text = s.draft.text.trim().to_owned();
    if text.is_empty() || s.tasks.len() >= MAX_TASKS {
        return;
    }
    s.tasks.push(Task { label: text, born: s.clock, sink: 0.0, done: false, done_at: 0.0 });
    s.draft.replace_text("");
}

fn task_press(s: &mut State, i: usize) {
    if let Some(t) = s.tasks.get_mut(i) {
        if t.done {
            // Un-seal: back on the slab, at the heat it had earned.
            t.done = false;
            t.born = s.clock - f64::from(t.sink) * TASK_LIFE;
        } else {
            t.done = true;
            t.done_at = s.clock;
            t.sink = ((s.clock - t.born) / TASK_LIFE) as f32;
        }
    }
}

/// Burning cracks across a slab too close to the magma.
fn crackle(sink: f32, seed: usize, t: f64) -> Shape {
    let mut s = Shape::new().view_box(300.0, 56.0);
    let heat = flick(t, seed as f64 * 5.0);
    let a = ((sink - 0.62) * 2.4).clamp(0.0, 1.0);
    for k in 0..5 {
        let x0 = hash((seed * 9 + k) as u64) * 240.0;
        let y0 = 12.0 + hash((seed * 9 + k + 1) as u64) * 30.0;
        let mut p = [x0, y0];
        for j in 0..4 {
            let q = [
                p[0] + 8.0 + hash((seed * 40 + k * 4 + j) as u64) * 18.0,
                (p[1] + (j as f32 - 1.5) * 7.0).clamp(9.0, 46.0),
            ];
            s = s.line(p, q, 5.5, LineCap::Round, LAVA.alpha(0.16 * a * heat));
            s = s.line(p, q, 1.2, LineCap::Round, EMBER.alpha(0.7 * a));
            p = q;
        }
    }
    s
}

/// The seal-mark check for a done task.
fn check_shape() -> Shape {
    Shape::new().view_box(18.0, 18.0).line([4.5, 9.5], [7.8, 13.0], 2.4, LineCap::Round, BASALT_BOT).line([7.8, 13.0], [13.8, 5.2], 2.4, LineCap::Round, BASALT_BOT)
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
    // The ledger: untouched tasks creep toward the magma; melted ones crumble
    // away, sealed ones fade into the wall.
    for t in &mut s.tasks {
        if !t.done {
            t.sink = ((s.clock - t.born) / TASK_LIFE) as f32;
        }
    }
    s.tasks.retain(|t| {
        if t.done {
            s.clock - t.done_at < SEAL_SECS
        } else {
            s.clock - t.born < TASK_LIFE + BURN_SECS
        }
    });
    // The hall is never quite still: veins pulse and embers climb, so the frame
    // clock keeps a slow ambient beat even at rest (the runtime only requests
    // frames while work is pending, so the app schedules its own next beat).
    tx.wake_after(std::time::Duration::from_millis(if animating || s.beat { 16 } else { 30 }));
    true
}

// ---------------------------------------------------------------------------
// The masonry: every panel is a slab of the procedural basalt — the baked-bump
// face, a top-lit wash, bevelled edges (light above, shadow below), an inset
// lip and a thick drop into the hall's dark.
// ---------------------------------------------------------------------------
fn slab(e: El, rock: &Pixels) -> El {
    e.pixels_fit(rock, SurfaceFit::Fill)
        .linear_gradient(
            std::f32::consts::PI,
            [(0.0, EDGE_HI.alpha(0.18)), (0.38, BLACK.alpha(0.0)), (1.0, BLACK.alpha(0.38))],
        )
        .border_gradient(1.5, EDGE_HI.alpha(0.9), EDGE_LO.alpha(0.95))
        .box_shadow(Shadow { x: 0.0, y: 2.0, blur: 3.0, spread: -0.5, color: BLACK.alpha(0.55), inset: true })
        .box_shadow(Shadow { x: 0.0, y: -1.5, blur: 1.5, spread: -1.0, color: EDGE_HI.alpha(0.3), inset: true })
        .shadow(11.0, 24.0, BLACK.alpha(0.45))
        .shadow(3.0, 0.5, EDGE_LO.alpha(0.9))
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

    let tab = |label: &'static str, key: &'static str, active: bool, rock: &Pixels, w: f32, on_press: fn(&mut State)| {
        let mut b = slab(
            el()
                .key(key)
                .w(w)
                .h(42.0)
                .center()
                .radius(9.0)
                .corners([9.0, 9.0, 5.0, 5.0])
                .press_scale(0.94),
            rock,
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
    const TO_LEDGER: fn(&mut State) = |s: &mut State| s.mode = Mode::Ledger;
    let tabs = row()
        .w_full()
        .gap(10.0)
        .child(tab("ALPHABET", "tab-alphabet", s.mode == Mode::Alphabet, &s.rock_a, 110.0, TO_ALPHA))
        .child(tab("THE CAST", "tab-cast", s.mode == Mode::Cast, &s.rock_b, 98.0, TO_CAST))
        .child(tab("THE LEDGER", "tab-ledger", s.mode == Mode::Ledger, &s.rock_a, 118.0, TO_LEDGER));

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
                    .h(212.0)
                    .radius(16.0)
                    .corners([16.0, 16.0, 9.0, 9.0])
                    .padding(14.0)
                    .child(row().w_full().h_full().gap(10.0).center_y().child(stage).child(info)),
                &s.rock_b,
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
                            .radius(9.0)
                            .corners([9.0, 9.0, 5.0, 5.0])
                            .center()
                            .gap(2.0)
                            .press_scale(0.93),
                        if idx % 2 == 0 { &s.rock_a } else { &s.rock_b },
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
                        .aspect_ratio(1_000_000.0) // let content drive height; a texture never stretches it
                        .radius(13.0)
                        .corners([13.0, 13.0, 7.0, 7.0])
                        .padding(10.0)
                        .gap(6.0),
                    if i % 2 == 0 { &s.rock_b } else { &s.rock_a },
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
                .radius(11.0)
                .corners([11.0, 11.0, 6.0, 6.0])
                .press_scale(0.97)
                .pixels_fit(&s.rock_a, SurfaceFit::Fill)
                .gradient(Color { r: 0.0452, g: 0.0145, b: 0.0052, a: 0.6 }, Color { r: 0.0164, g: 0.0056, b: 0.0027, a: 0.75 }) // iron wash over basalt (#3c2010 → #231109)
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
        Mode::Ledger => {
            let field = slab(
                el()
                    .key("task-field")
                    .w_full()
                    .h(48.0)
                    .radius(11.0)
                    .px(12.0)
                    .gap(10.0)
                    .center_y()
                    .fill(BLACK.alpha(0.5))
                    .child(
                        text_input(&s.draft, TextStyle::new(14.0, 400.0))
                            .key("task-input")
                            .color(INK)
                            .placeholder("carve a new task…", INK_DIM.alpha(0.55))
                            .single_line()
                            .grow(1.0)
                            .min_w(0.0)
                            .on_edit(|s: &mut State| &mut s.draft)
                            .on_submit(|s: &mut State, _tx| add_task(s)),
                    )
                    .child(
                        el()
                            .key("task-add")
                            .h(32.0)
                            .px(12.0)
                            .center()
                            .radius(8.0)
                            .press_scale(0.94)
                            .fill(GOLD.alpha(0.14))
                            .border(1.0, GOLD.alpha(0.55))
                            .child(engrave("STRIKE", 11.0, 700.0, 1.5, GOLD))
                            .on_press(add_task),
                    ),
                &s.rock_b,
            );
            let mut board = col().w_full().gap(9.0).pt(4.0);
            for (i, task) in s.tasks.iter().enumerate() {
                let melt = if task.done { -1.0 } else { ((s.clock - task.born) - TASK_LIFE) / BURN_SECS };
                let seal = if task.done { (SEAL_SECS - (s.clock - task.done_at)) / 1.4 } else { 1.0 };
                let mut card = slab(
                    col()
                        .key(format!("task-{i}"))
                        .w_full()
                        .aspect_ratio(1_000_000.0) // content-driven height over the rock face
                        .radius(11.0)
                        .corners([11.0, 11.0, 6.0, 6.0])
                        .px(12.0)
                        .py(10.0)
                        .gap(7.0)
                        .press_scale(0.98)
                        .mt(task.sink.max(0.0) * SINK_DROP)
                        .rotate(task.sink * 0.03)
                        .opacity(if task.done {
                            seal.clamp(0.0, 1.0) as f32
                        } else {
                            (1.0 - melt.max(0.0) as f32 * 0.92).max(0.0)
                        }),
                    if i % 2 == 0 { &s.rock_a } else { &s.rock_b },
                );
                if task.done {
                    card = card
                        .border(1.5, GOLD.alpha(0.6))
                        .box_shadow(Shadow { x: 0.0, y: 0.0, blur: 12.0, spread: 1.0, color: GOLD.alpha(0.16), inset: true });
                } else if task.sink > 0.5 {
                    card = card.border(1.5, EMBER.alpha(((task.sink - 0.5) * 1.6).min(0.9)));
                }
                if !task.done && task.sink > 0.6 {
                    card = card.shape(crackle(task.sink, i, t));
                }
                let mut mark = el()
                    .size(18.0, 18.0)
                    .radius(9.0)
                    .fill(if task.done { GOLD.alpha(0.9) } else { BLACK.alpha(0.55) })
                    .border(1.2, if task.done { GOLD } else { EDGE_HI.alpha(0.5) })
                    .box_shadow(Shadow { x: 0.0, y: 0.0, blur: 8.0, spread: 1.0, color: EMBER.alpha(((task.sink - 0.35) * 1.3 * heat).min(0.9)), inset: true })
                    .center();
                if task.done {
                    mark = mark.child(canvas(check_shape()).size(18.0, 18.0));
                }
                let mut label = text(task.label.clone())
                    .w_full()
                    .wrap()
                    .font(TextStyle::new(13.5, 450.0).lh(18.0))
                    .color(if task.done { GOLD.alpha(0.85) } else { INK });
                if task.done {
                    label = label.strike(0, task.label.len(), Some(GOLD.alpha(0.7)));
                }
                card = card
                    .on_press(move |s: &mut State| task_press(s, i))
                    .child(row().w_full().gap(10.0).center_y().child(mark).child(label.grow(1.0).min_w(0.0)));
                if !task.done {
                    card = card.child(
                        row()
                            .w_full()
                            .h(3.0)
                            .gap(0.0)
                            .radius(2.0)
                            .fill(BLACK.alpha(0.5))
                            .child(el().grow(task.sink.clamp(0.002, 1.0)).radius(2.0).fill(LAVA.alpha(0.6 + 0.35 * heat)))
                            .child(el().grow((1.0 - task.sink).clamp(0.002, 1.0))),
                    );
                }
                board = board.child(card);
            }
            let scroll = col()
                .key("ledger")
                .w_full()
                .grow(1.0)
                .shrink()
                .min_h(0.0)
                .scroll(ScrollSpec { stick_bottom: false })
                .scroll_edge_fade(16.0)
                .child(board);
            let hint = text("an untouched task slides into the magma — strike it down or lose it to the hall")
                .w_full()
                .wrap()
                .font(TextStyle::new(12.5, 400.0).lh(17.0))
                .color(INK_DIM.alpha(0.8));
            col().w_full().grow(1.0).min_h(0.0).gap(12.0).child(field).child(hint).child(scroll)
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
            "mode" if v == "tasks" => {
                s.mode = Mode::Ledger;
                seed_tasks(s, true);
            }
            "seed" => s.seed = v.parse().unwrap_or(0),
            "rune" => s.selected = v.parse::<usize>().unwrap_or(0).min(RUNES.len() - 1),
            "auto" => do_cast(s),
            "beat" => s.beat = true,
            "retouch" => retouch(s),
            _ => {}
        }
    }
}
