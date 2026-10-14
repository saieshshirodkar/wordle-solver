use wordle_solver::fmt::{fixed, push_fixed, scaled};

#[test]
fn fixed_scales_are_exact() {
    assert_eq!(fixed::<0>(0), "0");
    assert_eq!(fixed::<0>(704), "704");
    assert_eq!(fixed::<0>(9_999), "9999");
    assert_eq!(fixed::<1>(704), "70.4");
    assert_eq!(fixed::<2>(704), "7.04");
    assert_eq!(fixed::<3>(1_234_567), "1234.567");
    assert_eq!(fixed::<4>(34_644), "3.4644");
    assert_eq!(fixed::<4>(3), "0.0003");
}

#[test]
fn fixed_handles_negatives() {
    assert_eq!(fixed::<0>(-42), "-42");
    assert_eq!(fixed::<1>(-42), "-4.2");
    assert_eq!(fixed::<2>(-42), "-0.42");
    assert_eq!(fixed::<4>(-42), "-0.0042");
    assert_eq!(fixed::<2>(-1_234_567), "-12345.67");
}

#[test]
fn fixed_zero_pads_the_fraction() {
    assert_eq!(fixed::<4>(1_000), "0.1000");
    assert_eq!(fixed::<4>(1_010), "0.1010");
    assert_eq!(fixed::<4>(1), "0.0001");
}

#[test]
fn push_fixed_appends_into_the_caller_buffer() {
    let mut s = String::from("pre-");
    push_fixed::<3>(&mut s, 1_500);
    push_fixed::<0>(&mut s, 42);
    assert_eq!(s, "pre-1.50042");
}

#[test]
fn push_fixed_handles_zero_and_negative() {
    let mut s = String::new();
    push_fixed::<2>(&mut s, 0);
    assert_eq!(s, "0.00");

    let mut s = String::new();
    push_fixed::<2>(&mut s, -1);
    assert_eq!(s, "-0.01");
}

#[test]
fn scaled_rounds_to_nearest() {
    assert_eq!(scaled(1.4, 1.0), 1);
    assert_eq!(scaled(1.5, 1.0), 2);
    assert_eq!(scaled(2.5, 1.0), 3);
    assert_eq!(scaled(1_234.567, 1.0), 1235);
    assert_eq!(scaled(3.4337, 1e4), 34_337);
}

#[test]
fn scaled_multiplies_by_the_factor() {
    assert_eq!(scaled(2.5, 1e3), 2_500);
    assert_eq!(scaled(0.001, 1e6), 1_000);
    assert_eq!(scaled(0.0, 1e3), 0);
}

#[test]
fn scaled_saturates_on_non_finite() {
    assert_eq!(scaled(f64::INFINITY, 1.0), i64::MAX);
    assert_eq!(scaled(f64::NEG_INFINITY, 1.0), i64::MIN);
    assert_eq!(scaled(f64::NAN, 1.0), i64::MAX);
}

#[test]
fn scaled_never_truncates_an_absurd_value() {
    assert_eq!(scaled(f64::MAX, 1.0), i64::MAX);
}
