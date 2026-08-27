use crate::error::{DomainError, InvalidValue, require_finite, require_positive};

/// Calendar-year convention frozen by the MVP pricing contract.
pub const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

/// Absolute tolerance used only to remove negative floating-point round-off.
pub const PRICE_ABSOLUTE_TOLERANCE: f64 = 1.0e-12;

/// Relative tolerance used only to remove negative floating-point round-off.
pub const PRICE_RELATIVE_TOLERANCE: f64 = 1.0e-12;

/// Call and put values in model USD for one unit of each option.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptionPrices {
    pub call: f64,
    pub put: f64,
}

/// Columnar prices returned by one deterministic bulk Rust call.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchOptionPrices {
    pub calls: Vec<f64>,
    pub puts: Vec<f64>,
}

/// Price equally sized parameter columns without crossing a language boundary per row.
pub fn black_scholes_many(
    spot: &[f64],
    strike: &[f64],
    time_years: &[f64],
    sigma: &[f64],
    risk_free_rate: f64,
    carry_rate: f64,
) -> Result<BatchOptionPrices, DomainError> {
    require_finite("risk_free_rate", risk_free_rate)?;
    require_finite("carry_rate", carry_rate)?;
    let length = spot.len();
    for (field, actual) in [
        ("strike", strike.len()),
        ("time_years", time_years.len()),
        ("sigma", sigma.len()),
    ] {
        if actual != length {
            return Err(DomainError::LengthMismatch {
                field,
                expected: length,
                actual,
            });
        }
    }

    let mut calls = Vec::with_capacity(length);
    let mut puts = Vec::with_capacity(length);
    for index in 0..length {
        let prices = black_scholes(
            spot[index],
            strike[index],
            time_years[index],
            sigma[index],
            risk_free_rate,
            carry_rate,
        )?;
        calls.push(prices.call);
        puts.push(prices.put);
    }
    Ok(BatchOptionPrices { calls, puts })
}

/// Exact European call and put intrinsic values at expiry.
pub fn expiry_payoff(spot: f64, strike: f64) -> Result<OptionPrices, DomainError> {
    require_positive("spot", spot)?;
    require_positive("strike", strike)?;
    Ok(OptionPrices {
        call: (spot - strike).max(0.0),
        put: (strike - spot).max(0.0),
    })
}

/// Black--Scholes call and put prices with continuous rates `r` and `q`.
pub fn black_scholes(
    spot: f64,
    strike: f64,
    time_years: f64,
    sigma: f64,
    risk_free_rate: f64,
    carry_rate: f64,
) -> Result<OptionPrices, DomainError> {
    require_positive("spot", spot)?;
    require_positive("strike", strike)?;
    require_finite("time_years", time_years)?;
    if time_years < 0.0 {
        return Err(DomainError::InvalidField {
            field: "time_years",
            reason: InvalidValue::MustBeNonNegative,
        });
    }
    require_positive("sigma", sigma)?;
    require_finite("risk_free_rate", risk_free_rate)?;
    require_finite("carry_rate", carry_rate)?;

    if time_years == 0.0 {
        return expiry_payoff(spot, strike);
    }

    let sigma_sqrt_time = sigma * time_years.sqrt();
    let d1 = ((spot / strike).ln()
        + (risk_free_rate - carry_rate + 0.5 * sigma * sigma) * time_years)
        / sigma_sqrt_time;
    let d2 = d1 - sigma_sqrt_time;
    let discounted_spot = spot * (-carry_rate * time_years).exp();
    let discounted_strike = strike * (-risk_free_rate * time_years).exp();
    let parity = discounted_spot - discounted_strike;
    if !discounted_spot.is_finite() || !discounted_strike.is_finite() || !parity.is_finite() {
        return Err(DomainError::InvalidField {
            field: "pricing_result",
            reason: InvalidValue::NotFinite,
        });
    }

    // Compute the out-of-the-money leg directly and derive the other through
    // parity. This avoids subtracting two large almost-equal ITM terms.
    let scale = discounted_spot.abs().max(discounted_strike.abs()).max(1.0);
    let (call, put) = if parity >= 0.0 {
        let put = normalize_price(
            discounted_strike * normal_cdf(-d2) - discounted_spot * normal_cdf(-d1),
            scale,
        )?;
        (normalize_price(put + parity, scale)?, put)
    } else {
        let call = normalize_price(
            discounted_spot * normal_cdf(d1) - discounted_strike * normal_cdf(d2),
            scale,
        )?;
        (call, normalize_price(call - parity, scale)?)
    };
    Ok(OptionPrices { call, put })
}

fn normal_cdf(value: f64) -> f64 {
    0.5 * libm::erfc(-value / std::f64::consts::SQRT_2)
}

fn normalize_price(value: f64, scale: f64) -> Result<f64, DomainError> {
    if !value.is_finite() {
        return Err(DomainError::InvalidField {
            field: "pricing_result",
            reason: InvalidValue::NotFinite,
        });
    }
    if value >= 0.0 {
        return Ok(value);
    }
    let tolerance = PRICE_ABSOLUTE_TOLERANCE + PRICE_RELATIVE_TOLERANCE * scale;
    if value >= -tolerance {
        Ok(0.0)
    } else {
        Err(DomainError::InvalidField {
            field: "pricing_result",
            reason: InvalidValue::MustBeNonNegative,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PRICE_ABSOLUTE_TOLERANCE, normalize_price};
    use crate::{DomainError, InvalidValue};

    #[test]
    fn price_roundoff_normalization_clamps_only_within_tolerance() {
        assert_eq!(
            normalize_price(-0.5 * PRICE_ABSOLUTE_TOLERANCE, 1.0),
            Ok(0.0)
        );
        assert_eq!(normalize_price(1.25, 1.0), Ok(1.25));
        assert_eq!(
            normalize_price(-1.0, 1.0),
            Err(DomainError::InvalidField {
                field: "pricing_result",
                reason: InvalidValue::MustBeNonNegative,
            })
        );
        assert_eq!(
            normalize_price(f64::INFINITY, 1.0),
            Err(DomainError::InvalidField {
                field: "pricing_result",
                reason: InvalidValue::NotFinite,
            })
        );
    }
}
