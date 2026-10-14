#[must_use]
pub fn scaled(value: f64, factor: f64) -> i64 {
    let v = (value * factor).round();
    if !v.is_finite() {
        return if v.is_sign_negative() {
            i64::MIN
        } else {
            i64::MAX
        };
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "v is finite and elapsed times are orders of magnitude below i64::MAX"
    )]
    let n = v as i64;
    n
}

pub fn push_fixed<const D: u32>(out: &mut String, value: i64) {
    if value < 0 {
        out.push('-');
    }
    let mag = value.unsigned_abs();
    let scale = 10u64.pow(D);
    let mut whole = mag / scale;
    let frac = mag % scale;

    let mut buf = [0u8; 20];
    let mut at = buf.len();
    loop {
        at -= 1;
        buf[at] = b'0' + (whole % 10) as u8;
        whole /= 10;
        if whole == 0 {
            break;
        }
    }
    for &b in &buf[at..] {
        out.push(char::from(b));
    }

    if D > 0 {
        out.push('.');
        let mut div = scale / 10;
        for _ in 0..D {
            out.push(char::from(b'0' + ((frac / div) % 10) as u8));
            div /= 10;
        }
    }
}

#[must_use]
pub fn fixed<const D: u32>(value: i64) -> String {
    let mut s = String::with_capacity(16);
    push_fixed::<D>(&mut s, value);
    s
}
