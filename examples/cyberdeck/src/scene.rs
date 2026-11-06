//! The street outside the window: a synthwave sun sunk into a scrolling
//! perspective grid, two rows of skyline with flickering windows and neon
//! signs, and rain falling through it all. Everything is drawn fresh each
//! frame from the clock — the city is deterministic, never random.

use crate::Pal;
use rune::{Color, LineCap, Pixels, Shape};

/// Value hash in 0..1 — the city's only source of randomness.
pub fn hash(i: u64) -> f32 {
    let x = (i as f64 * 127.1).sin() * 43758.5453;
    (x - x.floor()) as f32
}

fn frac(x: f64) -> f32 {
    (x - x.floor()) as f32
}

/// Rect as a polygon.
fn rect(x: f32, y: f32, w: f32, h: f32) -> Vec<[f32; 2]> {
    vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}

fn circle(cx: f32, cy: f32, r: f32, n: u32) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            [cx + a.cos() * r, cy + a.sin() * r]
        })
        .collect()
}

/// The whole backdrop: sun, grid, skyline, neon, rain — in window points.
pub fn city(w: f32, h: f32, t: f64, pal: &Pal) -> Shape {
    let mut s = Shape::new();
    let horizon = h * 0.60;

    // — The sun: a magenta disc sunk behind the skyline, banded like the old posters.
    //     Off-centre so wide layouts don't hide it behind the tape panel.
    let r = (w * 0.26).min(h * 0.20).clamp(80.0, 170.0);
    let (sx, sy) = (w * 0.68, horizon - r * 0.14);
    s = s.fill(circle(sx, sy, r * 1.55, 48), pal.acc2.alpha(0.045));
    s = s.fill(circle(sx, sy, r * 1.18, 48), pal.acc2.alpha(0.10));
    s = s.fill(circle(sx, sy, r, 48), pal.acc2.alpha(0.42));
    s = s.fill(circle(sx, sy, r * 0.96, 48), pal.hot.alpha(0.16));
    // Bands: the sky eats the lower half of the sun in widening slices.
    for (i, k) in [0.30f32, 0.44, 0.58, 0.74, 0.90].into_iter().enumerate() {
        let y = sy + r * k;
        let th = 2.0 + i as f32 * 2.6;
        s = s.fill(rect(sx - r - 2.0, y, 2.0 * r + 4.0, th), pal.sky_lo.alpha(0.92));
    }

    // — The grid floor: perspective lines running from the horizon at you.
    for i in -9..=9 {
        let f = i as f32 / 9.0;
        let xb = w * 0.5 + f * w * 1.8;
        s = s.line([w * 0.5 + f * w * 0.10, horizon], [xb, h], 1.0, LineCap::Butt, pal.acc.alpha(0.055 + 0.05 * f.abs()));
    }
    for k in 0..15 {
        let p = frac(k as f64 / 15.0 + t * 0.06);
        let y = horizon + (h - horizon) * p * p;
        let a = 0.05 + 0.14 * p;
        s = s.line([0.0, y], [w, y], 0.8 + 1.4 * p, LineCap::Butt, pal.acc.alpha(a));
    }

    // — Back skyline: hazy towers with a slow blink of windows.
    let bucket = (t * 1.25).floor() as u64;
    for b in 0..17u64 {
        let bw = w * (0.045 + 0.055 * hash(b * 7 + 1));
        let bx = -30.0 + b as f32 * (w + 60.0) / 16.0 + hash(b * 7 + 2) * 26.0;
        let bh = h * (0.07 + 0.22 * hash(b * 7 + 3));
        let by = horizon - bh;
        s = s.fill(rect(bx, by, bw, bh + 2.0), pal.bld_far);
        s = s.line([bx, by], [bx + bw, by], 1.0, LineCap::Butt, Color { r: 0.10, g: 0.14, b: 0.22, a: 0.5 });
        for wk in 0..5u64 {
            if hash(b * 71 + wk) < 0.42 || hash(bucket + b * 5 + wk) < 0.30 {
                continue; // most windows are off; some blink
            }
            let wx = bx + bw * (0.12 + 0.76 * hash(b * 91 + wk));
            let wy = by + bh * (0.06 + 0.86 * hash(b * 111 + wk));
            let c = match hash(b * 31 + wk) {
                x if x < 0.45 => pal.acc,
                x if x < 0.78 => pal.acc2,
                _ => pal.amber,
            };
            s = s.fill(rect(wx, wy, 2.4, 3.2), c.alpha(0.42 + 0.3 * hash(bucket + wk * 3 + b)));
        }
    }

    // — Front skyline: darker, taller, closer; antennas with aviation lights.
    for b in 0..11u64 {
        let bw = w * (0.06 + 0.075 * hash(b * 13 + 5));
        let bx = -40.0 + b as f32 * (w + 80.0) / 10.0 + hash(b * 13 + 6) * 34.0;
        let bh = h * (0.10 + 0.26 * hash(b * 13 + 7));
        let by = horizon - bh + h * 0.035;
        s = s.fill(rect(bx, by, bw, bh + 30.0), pal.bld_near);
        // Roofline hairline catching the sun.
        s = s.line([bx, by], [bx + bw, by], 1.0, LineCap::Butt, pal.acc2.alpha(0.14));
        if hash(b * 13 + 8) > 0.45 {
            let ax = bx + bw * (0.3 + 0.4 * hash(b * 13 + 9));
            let ah = 14.0 + 26.0 * hash(b * 17 + 1);
            s = s.line([ax, by], [ax, by - ah], 1.0, LineCap::Round, Color { r: 0.12, g: 0.14, b: 0.2, a: 0.8 });
            let blink = 0.5 + 0.5 * (t * 3.1 + b as f64 * 1.7).sin();
            s = s.fill(circle(ax, by - ah - 1.5, 1.6, 10), pal.acc2.alpha(0.25 + 0.75 * blink as f32));
        }
        // A neon sign hung on the face: a vertical strip that breathes.
        if hash(b * 23 + 3) > 0.52 {
            let nx = bx + bw * (0.2 + 0.6 * hash(b * 23 + 4));
            let ny = by + bh * (0.14 + 0.4 * hash(b * 23 + 5));
            let nh = 26.0 + 42.0 * hash(b * 23 + 6);
            let c = if hash(b * 29 + 2) > 0.5 { pal.acc2 } else { pal.amber };
            let g = 0.72 + 0.28 * (t * 2.2 + b as f64 * 2.3).sin() as f32;
            s = s.fill(rect(nx - 3.0, ny, 6.0, nh), c.alpha(0.16 * g));
            s = s.fill(rect(nx - 1.2, ny + 1.5, 2.4, nh - 3.0), c.alpha(0.85 * g));
            for k in 0..4u32 {
                let ky = ny + nh * (0.15 + 0.22 * k as f32);
                s = s.fill(rect(nx - 2.6, ky, 5.2, 1.2), pal.hot.alpha(0.8 * g));
            }
        }
    }

    // — Rain: slanted streaks with a bright head, deterministic per column.
    let n = ((w * h) / 42_000.0).clamp(14.0, 44.0) as u64;
    for i in 0..n {
        let x = hash(i * 3 + 1) * (w + 40.0) - 20.0;
        let speed = 430.0 + 380.0 * hash(i * 3 + 2);
        let y = (t * f64::from(speed) + f64::from(hash(i * 5 + 3) * 3000.0)) % (h as f64 + 90.0) - 60.0;
        let len = 26.0 + 34.0 * hash(i * 5 + 4);
        let (y0, y1) = (y as f32, (y + f64::from(len)) as f32);
        let lean = len * 0.13;
        let c = if hash(i * 7 + 6) > 0.88 { pal.acc2 } else { pal.acc };
        s = s.line([x, y0], [x - lean, y1], 0.9, LineCap::Round, c.alpha(0.085));
        s = s.line([x, y1 - len * 0.18], [x - lean * 0.18, y1], 1.5, LineCap::Round, c.alpha(0.5));
    }

    s
}

/// CRT scanlines over everything, with a slow brighter band rolling down.
pub fn scanlines(w: f32, h: f32, t: f64) -> Shape {
    let mut s = Shape::new();
    let band = frac(t * 0.085) * (h + 160.0) - 80.0;
    let dark = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    let mut y = 1.0f32;
    while y < h {
        let in_band = (y - band).abs() < 46.0;
        s = s.line([0.0, y], [w, y], 1.0, LineCap::Butt, dark.alpha(if in_band { 0.11 } else { 0.045 }));
        y += 4.0;
    }
    s
}

/// A soft corner-darkening vignette, baked once into a tiny RGBA image the
/// renderer stretches over the window (bilinear — the gradient is perfect).
pub fn vignette() -> Pixels {
    const S: usize = 128;
    let mut rgba = vec![0u8; S * S * 4];
    for y in 0..S {
        for x in 0..S {
            let (u, v) = (x as f32 / S as f32 * 2.0 - 1.0, y as f32 / S as f32 * 2.0 - 1.0);
            let r = (u * u + v * v * 0.9).sqrt().min(1.4) / 1.4;
            let a = ((r - 0.58) / 0.42).clamp(0.0, 1.0).powf(1.7) * 0.62;
            let i = (y * S + x) * 4;
            rgba[i + 3] = (a * 255.0) as u8;
        }
    }
    let p = Pixels::new();
    p.set(S as u32, S as u32, rgba);
    p
}
