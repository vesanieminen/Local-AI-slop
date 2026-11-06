//! Futhark — the Elder Futhark, carved by springs.
//!
//! A Rune app: all 24 rune glyphs are drawn from scratch as animated vector
//! strokes (Rune's `Shape` polylines), revealed stroke by stroke like a chisel
//! cut. Browse the alphabet, tap a rune to re-cut it, or cast three runes for a
//! reading. Runs native; on wasm it mounts in a WebGPU canvas.

use rune::tok::{self, type_, TextStyle};
use rune::{canvas, col, el, row, text, AppSpec, Color, Cx, El, LineCap, ScrollSpec, Shape, Tx};

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

fn total_len(r: &Rune) -> f32 {
    let mut t = 0.0;
    for stroke in r.strokes {
        for w in stroke.windows(2) {
            t += ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
        }
    }
    t
}

/// The rune's strokes revealed up to fraction `p` of their total length.
/// While carving, the uncut ghost is shown as faint hairlines.
fn carve_shape(r: &Rune, p: f32, width: f32, ink: Color, ghost: Option<Color>) -> Shape {
    let mut s = Shape::new().view_box(VIEW_W, VIEW_H);
    if p < 1.0 {
        if let Some(g) = ghost {
            for stroke in r.strokes {
                for w in stroke.windows(2) {
                    s = s.line(w[0], w[1], width * 0.5, LineCap::Round, g);
                }
            }
        }
    }
    let target = p.clamp(0.0, 1.0) * total_len(r);
    let mut acc = 0.0;
    'strokes: for stroke in r.strokes {
        for w in stroke.windows(2) {
            let dx = w[1][0] - w[0][0];
            let dy = w[1][1] - w[0][1];
            let l = (dx * dx + dy * dy).sqrt();
            if acc + l <= target {
                s = s.line(w[0], w[1], width, LineCap::Round, ink);
                acc += l;
            } else {
                let f = ((target - acc) / l).clamp(0.0, 1.0);
                if f > 0.0 {
                    let q = [w[0][0] + dx * f, w[0][1] + dy * f];
                    s = s.line(w[0], q, width, LineCap::Round, ink);
                }
                break 'strokes;
            }
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
    if animating || s.beat {
        // Keep the frame clock running while a carve is mid-cut. On web the runtime
        // only requests frames while work is pending, so the app schedules its own
        // next beat (one poll a frame, the documented debounce cadence).
        tx.wake_after(std::time::Duration::from_millis(16));
    }
    animating || s.beat
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let dark = cx.system_dark();
    let t = tok::theme(dark);
    let [top, right, bottom, left] = cx.safe_area();
    let accent = tok::accent(0, dark);
    let mut accent_soft = accent;
    accent_soft.a = if dark { 0.22 } else { 0.18 };
    let on_accent = Color { r: 0.97, g: 0.98, b: 1.0, a: 1.0 };

    let header = col()
        .w_full()
        .gap(4.0)
        .child(
            text("FUTHARK")
                .font(TextStyle::new(24.0, 700.0).tracking(5.0))
                .color(t.text),
        )
        .child(
            text("the elder futhark · carved by springs")
                .font(type_::BODY)
                .color(t.text_secondary),
        );

    let tab = |label: &'static str, key: &'static str, active: bool, on_press: fn(&mut State)| {
        el()
            .key(key)
            .h(38.0)
            .px(14.0)
            .center()
            .radius(10.0)
            .press_scale(0.95)
            .fill(if active { accent } else { t.control_off })
            .child(text(label).font(type_::LABEL).color(if active { on_accent } else { t.text }))
            .on_press(on_press)
    };
    const TO_ALPHA: fn(&mut State) = |s: &mut State| s.mode = Mode::Alphabet;
    const TO_CAST: fn(&mut State) = |s: &mut State| s.mode = Mode::Cast;
    let tabs = row()
        .w_full()
        .gap(8.0)
        .child(tab("Alphabet", "tab-alphabet", s.mode == Mode::Alphabet, TO_ALPHA))
        .child(tab("Cast", "tab-cast", s.mode == Mode::Cast, TO_CAST));

    let body = match s.mode {
        Mode::Alphabet => {
            let r = &RUNES[s.selected];
            let stage = canvas(carve_shape(r, ease_out(s.carve), 4.0, accent, Some(accent_soft)))
                .key("stage")
                .size(116.0, 162.0)
                .on_press(retouch);
            let info = col()
                .grow(1.0)
                .min_w(0.0)
                .gap(8.0)
                .child(text(r.name).font(TextStyle::new(19.0, 600.0)).color(t.text))
                .child(
                    el()
                        .px(10.0)
                        .py(6.0)
                        .radius(8.0)
                        .h(26.0)
                        .center()
                        .fill(accent_soft)
                        .child(
                            text(format!("{} · “{}”", r.sound, r.word))
                                .font(type_::LABEL)
                                .color(t.text),
                        ),
                )
                .child(text(r.meaning).w_full().wrap().font(type_::BODY).color(t.text_secondary));
            let detail = el()
                .key("detail")
                .w_full()
                .radius(16.0)
                .fill(t.card)
                .padding(16.0)
                .child(row().w_full().gap(16.0).center_y().child(stage).child(info));

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
                    let cell = col()
                        .key(format!("cell-{idx}"))
                        .grow(1.0)
                        .h(88.0)
                        .radius(12.0)
                        .center()
                        .gap(2.0)
                        .press_scale(0.93)
                        .fill(if sel { accent_soft } else { t.card })
                        .on_press(move |s: &mut State| {
                            s.selected = idx;
                            retouch(s);
                        })
                        .child(
                            canvas(carve_shape(rune, 1.0, 5.5, if sel { accent } else { t.text }, None))
                                .size(40.0, 56.0),
                        )
                        .child(text(rune.name).font(type_::LABEL).color(t.text_secondary));
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
            let slots = ["past", "present", "future"];
            let mut cards = row().w_full().gap(10.0).center_y();
            for (i, slot) in slots.iter().enumerate() {
                let mut card = col().grow(1.0).min_w(0.0).radius(14.0).fill(t.card).padding(10.0).gap(6.0);
                let (glyph, name, meaning) = match &s.cast {
                    Some(cast) => {
                        let rune = &RUNES[cast.runes[i]];
                        (carve_shape(rune, cast.p[i], 5.0, accent, Some(accent_soft)), rune.name, rune.meaning)
                    }
                    None => (Shape::new().view_box(VIEW_W, VIEW_H), "", ""),
                };
                card = card
                    .child(text(*slot).font(type_::LABEL).color(t.text_secondary))
                    .child(canvas(glyph).key(format!("cast-{i}")).size(58.0, 81.0))
                    .child(text(name).w_full().wrap().font(type_::LABEL).color(t.text));
                if !meaning.is_empty() {
                    card = card.child(text(meaning).w_full().wrap().font(type_::BODY).color(t.text_secondary));
                }
                cards = cards.child(card);
            }
            let button = el()
                .key("cast-btn")
                .w_full()
                .h(52.0)
                .center()
                .radius(14.0)
                .press_scale(0.97)
                .fill(accent)
                .shadow(6.0, 22.0, {
                    let mut glow = accent;
                    glow.a = 0.35;
                    glow
                })
                .child(
                    text(if s.cast.is_some() { "cast again" } else { "cast the runes" })
                        .font(TextStyle::new(15.0, 700.0))
                        .color(on_accent),
                )
                .on_press(do_cast);
            let hint = text("three of the twenty-four, drawn for you — past, present, future")
                .w_full()
                .wrap()
                .font(type_::BODY)
                .color(t.text_secondary);
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

    row().w_full().h_full().center().fill(t.bg).child(inner)
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
