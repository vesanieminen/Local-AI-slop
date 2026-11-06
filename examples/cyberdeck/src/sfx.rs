//! The deck talks back. On the web every keystroke is a synthesized blip —
//! WebAudio oscillators, no samples. Native builds get the same calls and
//! the same dignified silence.

#[cfg(target_arch = "wasm32")]
pub type Sink = Option<web_sys::AudioContext>;
#[cfg(not(target_arch = "wasm32"))]
pub type Sink = ();

/// Play one oscillator note starting `at` seconds from now.
#[cfg(target_arch = "wasm32")]
fn tone(sink: &mut Sink, f0: f32, f1: f32, dur: f32, vol: f32, wave: &str, at: f64) {
    use wasm_bindgen::JsCast;
    use web_sys::OscillatorType;
    if sink.is_none() {
        match web_sys::AudioContext::new() {
            Ok(c) => *sink = Some(c),
            Err(_) => return,
        }
    }
    let Some(ctx) = sink.as_mut() else { return };
    let _ = ctx.resume();
    let t = ctx.current_time() + at;
    let Ok(osc) = ctx.create_oscillator() else { return };
    let ty = match wave {
        "triangle" => OscillatorType::Triangle,
        "sawtooth" => OscillatorType::Sawtooth,
        "sine" => OscillatorType::Sine,
        _ => OscillatorType::Square,
    };
    osc.set_type(ty);
    let _ = osc.frequency().set_value(f0);
    let _ = osc.frequency().exponential_ramp_to_value_at_time(f1.max(0.001), t + dur as f64);
    let Ok(gain) = ctx.create_gain() else { return };
    let g = gain.gain();
    let _ = g.set_value_at_time(0.0001, t);
    let _ = g.exponential_ramp_to_value_at_time(vol, t + 0.012);
    let _ = g.exponential_ramp_to_value_at_time(0.0001, t + dur as f64);
    let dest = ctx.destination();
    let _ = gain.connect_with_audio_node(dest.unchecked_ref::<web_sys::AudioNode>());
    let _ = osc.connect_with_audio_node(gain.unchecked_ref());
    let _ = osc.start_with_when(t);
    let _ = osc.stop_with_when(t + dur as f64 + 0.03);
}

#[cfg(not(target_arch = "wasm32"))]
fn tone(_sink: &mut Sink, _f0: f32, _f1: f32, _dur: f32, _vol: f32, _wave: &str, _at: f64) {}

/// A physical key. `slot` is the button index — each key sits at its own pitch,
/// like the keybed of a hardware synth.
pub fn press(sink: &mut Sink, slot: usize, kind: Press) {
    match kind {
        Press::Digit => tone(sink, 700.0 + slot as f32 * 46.0, 640.0 + slot as f32 * 46.0, 0.05, 0.10, "square", 0.0),
        Press::Dot => tone(sink, 520.0, 500.0, 0.05, 0.09, "square", 0.0),
        Press::Op => tone(sink, 410.0, 370.0, 0.07, 0.10, "triangle", 0.0),
        Press::Eq => {
            tone(sink, 660.0, 655.0, 0.06, 0.11, "square", 0.0);
            tone(sink, 990.0, 985.0, 0.09, 0.11, "square", 0.045);
        }
        Press::Ac => tone(sink, 320.0, 190.0, 0.09, 0.12, "sawtooth", 0.0),
        Press::Switch => tone(sink, 2500.0, 2200.0, 0.03, 0.05, "square", 0.0),
    }
}

/// Division by zero: two klaxon bites.
pub fn ice(sink: &mut Sink) {
    tone(sink, 220.0, 218.0, 0.16, 0.16, "sawtooth", 0.0);
    tone(sink, 165.0, 163.0, 0.16, 0.16, "sawtooth", 0.2);
    tone(sink, 220.0, 218.0, 0.16, 0.16, "sawtooth", 0.4);
}

/// 1337: an ascending arpeggio — something unlocked.
pub fn ghost(sink: &mut Sink) {
    for (i, f) in [523.25f32, 659.25, 783.99, 1046.5, 1318.5].into_iter().enumerate() {
        tone(sink, f, f, 0.16, 0.11, "square", i as f64 * 0.09);
    }
}

/// The boot chime, fired on the first frame: cold synth, two notes.
pub fn boot(sink: &mut Sink) {
    tone(sink, 330.0, 328.0, 0.5, 0.07, "triangle", 0.0);
    tone(sink, 494.0, 493.0, 0.5, 0.07, "triangle", 0.16);
}

#[derive(Clone, Copy)]
pub enum Press {
    Digit,
    Dot,
    Op,
    Eq,
    Ac,
    Switch,
}
