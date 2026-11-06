//! The calculator engine behind the deck: immediate-execute arithmetic in
//! radix 10 or 16, with the deck's own idea of what an error looks like.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    pub fn sym(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "×",
            Op::Div => "÷",
        }
    }
    fn apply(self, a: f64, b: f64) -> f64 {
        match self {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div => a / b,
        }
    }
}

/// What a finished operation was, so the view can react (sound, glitch, log).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Key,
    Equals,
    Ice,   // division by zero — the ICE bit into the result
    Out,   // the answer escaped the deck's width
    Ghost, // 1337 — something answered
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Radix {
    Dec,
    Hex,
}

pub struct Engine {
    pub radix: Radix,
    /// The current entry, as typed text ("-A0", "3.14").
    pub cur: String,
    pub acc: Option<f64>,
    pub op: Option<Op>,
    /// The next digit starts a new entry.
    pub fresh: bool,
    /// The pending expression, e.g. "1024 ×".
    pub expr: String,
    /// ICE or OUT is on the readout.
    pub err: bool,
    /// The last result, so the view can hex-mirror it even while you type on.
    pub last: Option<f64>,
    /// Set by the last action for the view to consume (sound + fx).
    pub event: Option<Event>,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            radix: Radix::Dec,
            cur: "0".into(),
            acc: None,
            op: None,
            fresh: false,
            expr: String::new(),
            err: false,
            last: None,
            event: None,
        }
    }
}

const MAX_HEX: u64 = 0xFFFF_FFFF_FFFF; // 16^12 — the biggest exact f64 integer we allow
const MAX_DEC: usize = 12;             // significant digits typed in decimal

impl Engine {
    pub fn set_radix(&mut self, r: Radix) {
        if self.radix == r {
            return;
        }
        // Re-read the entry under the new radix when it's unambiguous.
        self.radix = r;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.cur = "0".into();
        self.acc = None;
        self.op = None;
        self.fresh = false;
        self.expr = String::new();
        self.err = false;
        self.event = Some(Event::Key);
    }

    /// The numeric value of the current entry, in the current radix.
    pub fn value(&self) -> f64 {
        let neg = self.cur.starts_with('-');
        let body = self.cur.trim_start_matches('-');
        let v = match self.radix {
            Radix::Dec => body.parse::<f64>().unwrap_or(0.0),
            Radix::Hex => u64::from_str_radix(body, 16).map(|v| v as f64).unwrap_or(0.0),
        };
        if neg { -v } else { v }
    }

    /// Format a finite value for the readout in the current radix.
    pub fn fmt(&self, v: f64) -> String {
        match self.radix {
            Radix::Hex => {
                let iv = v as i64;
                if iv < 0 {
                    format!("-{:X}", -iv)
                } else {
                    format!("{iv:X}")
                }
            }
            Radix::Dec => {
                if v == v.trunc() && v.abs() < 1e13 {
                    format!("{}", v as i64)
                } else {
                    let a = v.abs();
                    let s = if a >= 1e13 || (a > 0.0 && a < 1e-6) {
                        format!("{:.5e}", v) // scientific; the readout draws the E
                    } else {
                        let mut s = format!("{:.8}", v);
                        while s.ends_with('0') {
                            s.pop();
                        }
                        if s.ends_with('.') {
                            s.pop();
                        }
                        s
                    };
                    s.to_uppercase()
                }
            }
        }
    }

    fn too_big(&self, v: f64) -> bool {
        match self.radix {
            Radix::Hex => v.abs() >= MAX_HEX as f64,
            Radix::Dec => v.abs() >= 1e13,
        }
    }

    fn digit_ok(d: char) -> bool {
        d.is_ascii_digit() || ('a'..='f').contains(&d.to_ascii_lowercase())
    }

    fn push_char(&mut self, d: char) {
        self.event = Some(Event::Key);
        if self.err {
            self.reset();
        }
        let d = if self.radix == Radix::Hex { d.to_ascii_uppercase() } else { d };
        if self.fresh {
            self.cur = d.to_string();
            self.fresh = false;
        } else if self.cur == "0" {
            self.cur = d.to_string();
        } else if self.cur.trim_start_matches('-').len()
            < match self.radix {
                Radix::Hex => 12,
                Radix::Dec => MAX_DEC,
            }
        {
            self.cur.push(d);
        }
    }

    pub fn digit(&mut self, d: char) {
        if Self::digit_ok(d) {
            self.push_char(d);
            self.ghost_check();
        }
    }

    pub fn dot(&mut self) {
        self.event = Some(Event::Key);
        if self.err {
            self.reset();
        }
        if self.radix == Radix::Hex {
            return; // whole bytes only on the wire
        }
        if self.fresh {
            self.cur = "0.".into();
            self.fresh = false;
        } else if !self.cur.contains('.') {
            self.cur.push('.');
        }
    }

    pub fn sign(&mut self) {
        self.event = Some(Event::Key);
        if self.err {
            self.reset();
            return;
        }
        if self.cur == "0" || self.cur == "0." {
            return;
        }
        if self.cur.starts_with('-') {
            self.cur.remove(0);
        } else {
            self.cur.insert(0, '-');
        }
    }

    pub fn back(&mut self) {
        self.event = Some(Event::Key);
        if self.err {
            self.reset();
            return;
        }
        if self.fresh {
            return;
        }
        self.cur.pop();
        if self.cur.is_empty() || self.cur == "-" {
            self.cur = "0".into();
            self.fresh = true;
        }
    }

    /// Percent: + and − take it of the accumulator; × and ÷ of one.
    pub fn percent(&mut self) {
        if self.err {
            return;
        }
        self.event = Some(Event::Key);
        let v = self.value();
        let v = match (self.op, self.acc) {
            (Some(Op::Add) | Some(Op::Sub), Some(a)) => a * v / 100.0,
            _ => v / 100.0,
        };
        self.cur = self.fmt(v);
        self.fresh = true;
    }

    pub fn op(&mut self, o: Op) {
        self.event = Some(Event::Key);
        if self.err {
            self.reset();
        }
        let v = self.value();
        let chained = match (self.op, self.acc, self.fresh) {
            (Some(p), Some(a), false) => match p.apply(a, v) {
                r if !r.is_finite() => {
                    self.fail(Event::Out);
                    return;
                }
                r if self.too_big(r) => {
                    self.fail(Event::Out);
                    return;
                }
                r => {
                    self.expr = format!("{} {}", self.fmt(r), o.sym());
                    Some(r)
                }
            },
            (Some(_), _, true) => {
                self.expr = format!("{} {}", self.fmt(self.acc.unwrap_or(v)), o.sym());
                self.acc
            }
            _ => {
                self.expr = format!("{} {}", self.fmt(v), o.sym());
                Some(v)
            }
        };
        self.acc = chained;
        self.op = Some(o);
        self.fresh = true;
    }

    pub fn equals(&mut self) {
        if self.err {
            return;
        }
        let (Some(p), Some(a)) = (self.op, self.acc) else {
            self.event = Some(Event::Equals);
            return;
        };
        let b = self.value();
        self.expr = format!("{} {} {} =", self.fmt(a), p.sym(), self.fmt(b));
        if p == Op::Div && b == 0.0 {
            self.fail(Event::Ice);
            return;
        }
        let r = p.apply(a, b);
        if !r.is_finite() || self.too_big(r) {
            self.fail(Event::Out);
            return;
        }
        self.acc = None;
        self.op = None;
        self.last = Some(r);
        self.cur = self.fmt(r);
        self.fresh = true;
        self.event = Some(Event::Equals);
        self.ghost_check();
    }

    fn fail(&mut self, ev: Event) {
        self.err = true;
        self.acc = None;
        self.op = None;
        self.expr.clear();
        self.event = Some(ev);
    }

    fn ghost_check(&mut self) {
        if self.value() == 1337.0 {
            self.event = Some(Event::Ghost);
        }
    }

    /// The hex mirror line under the readout: what the number is in machine ink.
    /// While you type it follows the entry; at rest it shows the last answer.
    pub fn mirror(&self) -> String {
        let v = if self.fresh { self.last } else { Some(self.value()) }.unwrap_or(0.0);
        let neg = v < 0.0;
        let a = v.abs();
        if a == a.trunc() && a < f64::from(u32::MAX) {
            let u = a as u32;
            let bin = format!("{u:b}");
            let bin = if bin.len() > 21 { bin[bin.len() - 21..].to_string() } else { bin };
            format!("{}0x{:X}  0b{}", if neg { "-" } else { "" }, u, bin)
        } else if a < 1e15 {
            format!("{}0x{:X}", if neg { "-" } else { "" }, a as u64)
        } else {
            "0x—".to_string()
        }
    }
}

