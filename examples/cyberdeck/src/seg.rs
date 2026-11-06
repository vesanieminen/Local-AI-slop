//! The readout itself: every glyph is drawn as seven bevelled segments —
//! never a font. Lit segments burn in three passes (haze, body, filament);
//! dark segments ghost behind them like a real panel. The word bank carries
//! the few letters the seven-segment alphabet can spell — enough to shout
//! ICE, OUT, HELLO.

use rune::{Color, LineCap, Shape};

/// One character cell; every polygon lives in these units and is scaled when drawn.
pub const CW: f32 = 100.0;
pub const CH: f32 = 170.0;

// a=top b=top-right c=bot-right d=bottom e=bot-left f=top-left g=middle
const SEGS: [&[[f32; 2]]; 7] = [
    &[[8., 0.], [92., 0.], [82., 14.], [18., 14.]],            // a
    &[[92., 8.], [100., 18.], [100., 68.], [92., 78.]],        // b
    &[[92., 92.], [100., 102.], [100., 152.], [92., 162.]],    // c
    &[[18., 156.], [82., 156.], [92., 170.], [8., 170.]],      // d
    &[[8., 92.], [0., 102.], [0., 152.], [8., 162.]],          // e
    &[[8., 8.], [0., 18.], [0., 68.], [8., 78.]],              // f
    &[[8., 78.], [92., 78.], [82., 92.], [18., 92.]],          // g
];

const SEG_W: [u8; 7] = [1, 2, 4, 8, 16, 32, 64];

/// What a character needs beyond segments.
#[derive(Clone, Copy, PartialEq)]
pub enum Glyph {
    Seg(u8),
    Dot,
    Slash,
    Bar,
    Blank,
}

pub fn glyph(ch: char) -> Glyph {
    use Glyph::*;
    match ch {
        '0' => Seg(0x3F),
        '1' => Seg(0x06),
        '2' => Seg(0x5B),
        '3' => Seg(0x4F),
        '4' => Seg(0x66),
        '5' => Seg(0x6D),
        '6' => Seg(0x7D),
        '7' => Seg(0x07),
        '8' => Seg(0x7F),
        '9' => Seg(0x6F),
        // The seven-segment word bank.
        'A' | 'a' => Seg(0x77),
        'C' | 'c' => Seg(0x39),
        'D' | 'd' => Seg(0x3E),
        'E' | 'e' => Seg(0x79),
        'F' | 'f' => Seg(0x71),
        'H' | 'h' => Seg(0x76),
        'L' | 'l' => Seg(0x38),
        'O' | 'o' => Seg(0x3F),
        'P' | 'p' => Seg(0x73),
        'S' | 's' => Seg(0x6D),
        'T' | 't' => Seg(0x70),
        'U' | 'u' => Seg(0x3E),
        '-' => Seg(0x40),
        '.' => Dot,
        '/' => Slash,
        '_' => Seg(0x08),
        'I' | 'i' => Bar, // no true I in seven segments — a centred bar reads best
        _ => Blank,
    }
}

/// The advance width of a glyph, in cell units.
pub fn advance(g: Glyph) -> f32 {
    match g {
        Glyph::Dot => 0.45,
        _ => 1.0,
    }
}

pub fn hash(i: u64) -> f32 {
    let x = (i as f64 * 127.1).sin() * 43758.5453;
    (x - x.floor()) as f32
}

/// Lay `chars` out left to right and return the total width in points for
/// cell width `w` (points) and gap `k` (fraction of `w`).
pub fn measure(chars: &[char], w: f32, k: f32) -> f32 {
    let mut x = 0.0;
    for (i, &ch) in chars.iter().enumerate() {
        x += advance(glyph(ch)) * w;
        if i + 1 < chars.len() {
            x += w * k;
        }
    }
    x
}

/// Scale a cell-space point to the canvas.
fn pt(p: [f32; 2], x0: f32, y0: f32, s: f32) -> [f32; 2] {
    [x0 + p[0] * s, y0 + p[1] * s]
}

/// Shrink a polygon toward its centroid — the filament inside the body.
fn inset(poly: &[[f32; 2]], k: f32) -> Vec<[f32; 2]> {
    let n = poly.len() as f32;
    let c = [
        poly.iter().map(|p| p[0]).sum::<f32>() / n,
        poly.iter().map(|p| p[1]).sum::<f32>() / n,
    ];
    poly.iter().map(|p| [c[0] + (p[0] - c[0]) * k, c[1] + (p[1] - c[1]) * k]).collect()
}

/// Draw the whole readout into `sh`, left to right from `x0`.
///
/// * `t` — app clock, for the segment flicker.
/// * `seed` — per-character flicker offset.
/// * `heat` — 0..1 brightness multiplier (boot fade, glitch dips).
/// * `sweep` — x in canvas points of the "result uploaded" scan band, or `None`.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    sh: Shape,
    chars: &[char],
    x0: f32,
    y0: f32,
    w: f32,
    ghost: Color,
    body: Color,
    core: Color,
    haze: Color,
    t: f64,
    seed: u64,
    heat: f32,
    sweep: Option<f32>,
) -> Shape {
    let mut sh = sh;
    let s = w / CW;
    let h = CH * s;
    let k = 0.08 * w; // inter-character gap
    let mut x = x0;
    let bucket = (t * 14.0).floor() as u64;

    for (ci, &ch) in chars.iter().enumerate() {
        let g = glyph(ch);
        match g {
            Glyph::Dot => {
                let d = 15.0 * s;
                let p = vec![
                    [x + (advance(g) * w - d) / 2.0, y0 + h - d],
                    [x + (advance(g) * w + d) / 2.0, y0 + h - d],
                    [x + (advance(g) * w + d) / 2.0, y0 + h],
                    [x + (advance(g) * w - d) / 2.0, y0 + h],
                ];
                sh = sh.fill(p.clone(), haze.alpha(0.55 * heat)).fill(inset(&p, 0.45), core.alpha(0.95 * heat));
            }
            Glyph::Slash => {
                let (mx, my) = (x + w * 0.5, y0 + h * 0.5);
                let (dx, dy) = (w * 0.36, h * 0.44);
                let (a, b) = ([mx - dx, my + dy], [mx + dx, my - dy]);
                sh = sh.line(a, b, w * 0.26, LineCap::Round, body.alpha(0.30 * heat));
                sh = sh.line(a, b, w * 0.10, LineCap::Round, core.alpha(0.95 * heat));
            }
            Glyph::Bar => {
                let mx = x + w * 0.5;
                let (a, b) = ([mx, y0 + h * 0.06], [mx, y0 + h * 0.94]);
                sh = sh.line(a, b, w * 0.30, LineCap::Round, body.alpha(0.28 * heat));
                sh = sh.line(a, b, w * 0.12, LineCap::Round, core.alpha(0.95 * heat));
            }
            Glyph::Blank => {}
            Glyph::Seg(mask) => {
                for (si, poly) in SEGS.iter().enumerate() {
                    let lit = mask & SEG_W[si] != 0;
                    let cx0 = x;
                    let pts: Vec<[f32; 2]> = poly.iter().map(|p| pt(*p, cx0, y0, s)).collect();
                    // A cold joint: a lit segment that keeps going dark.
                    let flake = hash(bucket * 91 + seed + (ci * 7 + si) as u64) < 0.006;
                    let f = heat * if flake { 0.18 } else { 1.0 };
                    if lit {
                        let ring: Vec<[f32; 2]> = pts.iter().copied().chain(std::iter::once(pts[0])).collect();
                        sh = sh.stroke(ring.clone(), w * 0.30, haze.alpha(0.10 * f));
                        sh = sh.stroke(ring.clone(), w * 0.14, body.alpha(0.26 * f));
                        sh = sh.fill(pts.clone(), body.alpha(0.85 * f));
                        sh = sh.fill(inset(&pts, 0.55), core.alpha(0.96 * f));
                    } else {
                        sh = sh.fill(pts, ghost.alpha(0.055));
                    }
                }
            }
        }
        x += advance(g) * w + k;
    }

    // The "result uploaded" band sweeping across the panel.
    if let Some(sx) = sweep {
        let p = vec![
            [sx - 26.0, y0],
            [sx, y0],
            [sx + 16.0, y0 + h],
            [sx - 10.0, y0 + h],
        ];
        sh = sh.fill(p, Color { r: 0.8, g: 1.0, b: 1.0, a: 0.07 });
    }
    sh
}
