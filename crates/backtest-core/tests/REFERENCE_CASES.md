# Black–Scholes pricing reference cases

The three values in `pricing.rs` were calculated independently from the Rust
implementation with `mpmath==1.3.0` at 80 decimal digits. The reproducible
command is:

```bash
uv run --with mpmath==1.3.0 python - <<'PY'
import mpmath as mp

mp.mp.dps = 80
cases = [
    ("100", "100", "1", ".20", ".05", "0"),
    ("100", "110", ".5", ".25", ".03", ".01"),
    ("42", "40", ".5", ".20", ".10", ".02"),
]
normal_cdf = lambda value: (1 + mp.erf(value / mp.sqrt(2))) / 2
for values in cases:
    spot, strike, time, sigma, rate, carry = map(mp.mpf, values)
    d1 = (
        mp.log(spot / strike)
        + (rate - carry + sigma * sigma / 2) * time
    ) / (sigma * mp.sqrt(time))
    d2 = d1 - sigma * mp.sqrt(time)
    call = (
        spot * mp.exp(-carry * time) * normal_cdf(d1)
        - strike * mp.exp(-rate * time) * normal_cdf(d2)
    )
    put = (
        strike * mp.exp(-rate * time) * normal_cdf(-d2)
        - spot * mp.exp(-carry * time) * normal_cdf(-d1)
    )
    print(mp.nstr(call, 30), mp.nstr(put, 30))
PY
```

The test uses
`absolute_tolerance + relative_tolerance * max(abs(actual), abs(expected))`,
with both tolerances fixed at `1e-12`. Production negative-roundoff handling
uses the separately exported `PRICE_ABSOLUTE_TOLERANCE` and
`PRICE_RELATIVE_TOLERANCE`, also fixed at `1e-12`.
