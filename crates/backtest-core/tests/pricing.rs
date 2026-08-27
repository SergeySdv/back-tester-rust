use backtest_core::{
    DomainError, InvalidValue, PRICE_ABSOLUTE_TOLERANCE, PRICE_RELATIVE_TOLERANCE,
    SECONDS_PER_YEAR, black_scholes, black_scholes_many, expiry_payoff,
};

// Provenance and the reproducible high-precision calculation are documented in
// REFERENCE_CASES.md next to this test.
const REFERENCE_ABSOLUTE_TOLERANCE: f64 = 1.0e-12;
const REFERENCE_RELATIVE_TOLERANCE: f64 = 1.0e-12;

fn assert_close(actual: f64, expected: f64) {
    let tolerance = REFERENCE_ABSOLUTE_TOLERANCE
        + REFERENCE_RELATIVE_TOLERANCE * actual.abs().max(expected.abs());
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual:.15}, expected={expected:.15}"
    );
}

#[test]
fn reference_prices_match_three_independent_cases() {
    let cases = [
        (
            100.0,
            100.0,
            1.0,
            0.20,
            0.05,
            0.00,
            10.450583572185567,
            5.573526022256968,
        ),
        (
            100.0,
            110.0,
            0.5,
            0.25,
            0.03,
            0.01,
            3.7230100451832606,
            12.584075482251922,
        ),
        (
            42.0,
            40.0,
            0.5,
            0.20,
            0.10,
            0.02,
            4.438273938289693,
            0.9053579008531949,
        ),
    ];
    for (spot, strike, time, sigma, rate, carry, expected_call, expected_put) in cases {
        let actual = black_scholes(spot, strike, time, sigma, rate, carry).unwrap();
        assert_close(actual.call, expected_call);
        assert_close(actual.put, expected_put);
    }
}

#[test]
fn put_call_parity_holds_with_nonzero_rates() {
    let (spot, strike, time, sigma, rate, carry) =
        (101.0_f64, 97.0_f64, 0.37_f64, 0.63_f64, 0.07_f64, -0.02_f64);
    let prices = black_scholes(spot, strike, time, sigma, rate, carry).unwrap();
    let expected = spot * (-carry * time).exp() - strike * (-rate * time).exp();
    assert_close(prices.call - prices.put, expected);
}

#[test]
fn expiry_returns_exact_intrinsic_for_itm_otm_and_atm() {
    assert_eq!(
        black_scholes(120.0, 100.0, 0.0, 0.3, 0.1, 0.2).unwrap(),
        expiry_payoff(120.0, 100.0).unwrap()
    );
    assert_eq!(expiry_payoff(120.0, 100.0).unwrap().call, 20.0);
    assert_eq!(expiry_payoff(80.0, 100.0).unwrap().put, 20.0);
    assert_eq!(expiry_payoff(100.0, 100.0).unwrap().call, 0.0);
    assert_eq!(expiry_payoff(100.0, 100.0).unwrap().put, 0.0);
}

#[test]
fn atm_and_near_expiry_prices_are_finite_and_nonnegative() {
    let time = 1.0 / SECONDS_PER_YEAR as f64;
    let prices = black_scholes(50_000.0, 50_000.0, time, 0.8, 0.03, 0.01).unwrap();
    assert!(prices.call.is_finite() && prices.call >= 0.0);
    assert!(prices.put.is_finite() && prices.put >= 0.0);
    assert_eq!(SECONDS_PER_YEAR, 31_536_000);
}

#[test]
fn deep_otm_24_hour_rounding_tail_is_never_negative() {
    let spot = 50_000.0_f64;
    let time = 1.0_f64 / 365.0;
    let sigma = 0.55_f64;
    let strike = spot * (0.5 * sigma * sigma * time + 8.0 * sigma * time.sqrt()).exp();
    let prices = black_scholes(spot, strike, time, sigma, 0.0, 0.0).unwrap();
    assert!(prices.call >= 0.0);
    assert!(prices.put >= 0.0);
    assert_close(prices.call - prices.put, spot - strike);
}

#[test]
fn extreme_finite_rounding_tail_is_nonnegative() {
    let spot = 1.0e62_f64;
    let time = 1.0_f64 / 365.0;
    let sigma = 0.55_f64;
    let strike = spot * (0.5 * sigma * sigma * time + 8.0 * sigma * time.sqrt()).exp();
    let prices = black_scholes(spot, strike, time, sigma, 0.0, 0.0).unwrap();
    assert!(prices.call >= 0.0);
    assert!(prices.put >= 0.0);
    let parity = spot - strike;
    let error = ((prices.call - prices.put) - parity).abs();
    assert!(error <= PRICE_ABSOLUTE_TOLERANCE + PRICE_RELATIVE_TOLERANCE * spot);
}

#[test]
fn invalid_pricing_inputs_return_typed_errors() {
    assert_invalid(
        (0.0, 100.0, 1.0, 0.2, 0.0, 0.0),
        "spot",
        InvalidValue::MustBePositive,
    );
    assert_invalid(
        (100.0, -1.0, 1.0, 0.2, 0.0, 0.0),
        "strike",
        InvalidValue::MustBePositive,
    );
    assert_invalid(
        (100.0, 100.0, -0.1, 0.2, 0.0, 0.0),
        "time_years",
        InvalidValue::MustBeNonNegative,
    );
    assert_invalid(
        (100.0, 100.0, 1.0, 0.0, 0.0, 0.0),
        "sigma",
        InvalidValue::MustBePositive,
    );
    assert_invalid(
        (f64::NAN, 100.0, 1.0, 0.2, 0.0, 0.0),
        "spot",
        InvalidValue::NotFinite,
    );
    assert_invalid(
        (100.0, f64::INFINITY, 1.0, 0.2, 0.0, 0.0),
        "strike",
        InvalidValue::NotFinite,
    );
    assert_invalid(
        (100.0, 100.0, f64::NAN, 0.2, 0.0, 0.0),
        "time_years",
        InvalidValue::NotFinite,
    );
    assert_invalid(
        (100.0, 100.0, 1.0, f64::NAN, 0.0, 0.0),
        "sigma",
        InvalidValue::NotFinite,
    );
    assert_invalid(
        (100.0, 100.0, 1.0, 0.2, f64::INFINITY, 0.0),
        "risk_free_rate",
        InvalidValue::NotFinite,
    );
    assert_invalid(
        (100.0, 100.0, 1.0, 0.2, 0.0, f64::NEG_INFINITY),
        "carry_rate",
        InvalidValue::NotFinite,
    );
}

fn assert_invalid(
    input: (f64, f64, f64, f64, f64, f64),
    field: &'static str,
    reason: InvalidValue,
) {
    let (spot, strike, time, sigma, rate, carry) = input;
    assert_eq!(
        black_scholes(spot, strike, time, sigma, rate, carry),
        Err(DomainError::InvalidField { field, reason })
    );
}

#[test]
fn extreme_finite_inputs_cannot_leak_nonfinite_price() {
    assert_eq!(
        black_scholes(
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::MAX,
            -f64::MAX,
            -f64::MAX
        ),
        Err(DomainError::InvalidField {
            field: "pricing_result",
            reason: InvalidValue::NotFinite
        })
    );
}

#[test]
fn repeated_pricing_is_bit_deterministic() {
    let first = black_scholes(49_123.5, 50_000.0, 1.0 / 365.0, 0.55, 0.04, 0.01).unwrap();
    let second = black_scholes(49_123.5, 50_000.0, 1.0 / 365.0, 0.55, 0.04, 0.01).unwrap();
    assert_eq!(first.call.to_bits(), second.call.to_bits());
    assert_eq!(first.put.to_bits(), second.put.to_bits());
}

#[test]
fn bulk_pricing_prices_rows_and_propagates_typed_errors() {
    let result = black_scholes_many(
        &[100.0, 120.0],
        &[100.0, 100.0],
        &[1.0, 0.0],
        &[0.2, 0.3],
        0.05,
        0.01,
    )
    .unwrap();
    assert_eq!(result.calls.len(), 2);
    assert_eq!(result.puts.len(), 2);
    assert_eq!(result.calls[1], 20.0);

    assert!(matches!(
        black_scholes_many(&[100.0], &[], &[1.0], &[0.2], 0.0, 0.0),
        Err(DomainError::LengthMismatch {
            field: "strike",
            expected: 1,
            actual: 0
        })
    ));
    assert!(black_scholes_many(&[0.0], &[100.0], &[1.0], &[0.2], 0.0, 0.0).is_err());
}

#[test]
fn bulk_pricing_checks_every_column_length() {
    for result in [
        black_scholes_many(&[100.0], &[100.0], &[], &[0.2], 0.0, 0.0),
        black_scholes_many(&[100.0], &[100.0], &[1.0], &[], 0.0, 0.0),
    ] {
        assert!(matches!(result, Err(DomainError::LengthMismatch { .. })));
    }
}

#[test]
fn bulk_pricing_validates_shared_rates_and_accepts_an_empty_batch() {
    let empty = black_scholes_many(&[], &[], &[], &[], 0.03, 0.01).unwrap();
    assert!(empty.calls.is_empty());
    assert!(empty.puts.is_empty());

    for (risk_free_rate, carry_rate, field) in [
        (f64::NAN, 0.0, "risk_free_rate"),
        (0.0, f64::INFINITY, "carry_rate"),
    ] {
        assert!(matches!(
            black_scholes_many(
                &[100.0],
                &[100.0],
                &[1.0],
                &[0.2],
                risk_free_rate,
                carry_rate
            ),
            Err(DomainError::InvalidField {
                field: actual_field,
                reason: InvalidValue::NotFinite,
            }) if actual_field == field
        ));
    }
}
