use backtest_core::{
    BacktestConfig, DatasetMetadata, DomainError, InvalidValue, IvScenario, validate_scenarios,
};

fn valid_metadata() -> DatasetMetadata {
    DatasetMetadata {
        dataset_id: "fixture-1".into(),
        source: "synthetic".into(),
        symbol: "BTCUSDT".into(),
        interval_seconds: 60,
        timezone: "UTC".into(),
    }
}

#[test]
fn dataset_metadata_accepts_complete_utc_minute_identity() {
    assert_eq!(valid_metadata().validate(), Ok(()));
}

#[test]
fn dataset_metadata_rejects_empty_identity_unsupported_interval_and_timezone() {
    for field in ["dataset_id", "source", "symbol"] {
        let mut metadata = valid_metadata();
        match field {
            "dataset_id" => metadata.dataset_id = " ".into(),
            "source" => metadata.source.clear(),
            _ => metadata.symbol.clear(),
        }
        assert!(matches!(
            metadata.validate(),
            Err(DomainError::InvalidField {
                reason: InvalidValue::Empty,
                ..
            })
        ));
    }
    let mut metadata = valid_metadata();
    metadata.interval_seconds = 300;
    assert!(matches!(
        metadata.validate(),
        Err(DomainError::InvalidField {
            field: "dataset.interval_seconds",
            ..
        })
    ));
    let mut metadata = valid_metadata();
    metadata.timezone = "Europe/Moscow".into();
    assert!(matches!(
        metadata.validate(),
        Err(DomainError::InvalidField {
            field: "dataset.timezone",
            ..
        })
    ));
}

#[test]
fn backtest_config_validates_every_numeric_field() {
    let valid = BacktestConfig {
        initial_capital_usd: 1_000.0,
        base_iv: 0.55,
        risk_free_rate: 0.02,
        carry_rate: -0.01,
        margin_per_straddle_usd: 100.0,
        quantity_step: 0.1,
    };
    assert_eq!(valid.validate(), Ok(()));
    let invalid = [
        BacktestConfig {
            initial_capital_usd: 0.0,
            ..valid
        },
        BacktestConfig {
            base_iv: f64::NAN,
            ..valid
        },
        BacktestConfig {
            risk_free_rate: f64::INFINITY,
            ..valid
        },
        BacktestConfig {
            carry_rate: f64::NEG_INFINITY,
            ..valid
        },
        BacktestConfig {
            margin_per_straddle_usd: -1.0,
            ..valid
        },
        BacktestConfig {
            quantity_step: 0.0,
            ..valid
        },
    ];
    assert!(invalid.into_iter().all(|config| config.validate().is_err()));
}

#[test]
fn scenario_collection_is_validated_and_canonicalized() {
    let canonical = validate_scenarios(&[
        IvScenario::Stress3x { after_minutes: 720 },
        IvScenario::Baseline,
        IvScenario::Stress2x { after_minutes: 60 },
    ])
    .unwrap();
    assert_eq!(
        canonical
            .iter()
            .map(|item| item.scenario_id())
            .collect::<Vec<_>>(),
        ["baseline", "stress_2x", "stress_3x"]
    );
    assert_eq!(canonical[0].multiplier(), 1.0);
    assert_eq!(canonical[1].multiplier(), 2.0);
    assert_eq!(canonical[2].multiplier(), 3.0);
}

#[test]
fn scenario_collection_rejects_empty_duplicate_and_invalid_shock() {
    assert!(validate_scenarios(&[]).is_err());
    assert!(matches!(
        validate_scenarios(&[IvScenario::Baseline, IvScenario::Baseline]),
        Err(DomainError::InvalidScenario {
            reason: InvalidValue::Duplicate,
            ..
        })
    ));
    assert!(validate_scenarios(&[IvScenario::Stress2x { after_minutes: 0 }]).is_err());
    assert!(
        validate_scenarios(&[IvScenario::Stress3x {
            after_minutes: 1440
        }])
        .is_err()
    );
}

#[test]
fn scenario_parser_rejects_custom_and_malformed_variants() {
    assert_eq!(
        IvScenario::parse("baseline", None),
        Ok(IvScenario::Baseline)
    );
    assert!(IvScenario::parse("custom", Some(12)).is_err());
    assert!(IvScenario::parse("baseline", Some(12)).is_err());
    assert!(IvScenario::parse("stress_2x", None).is_err());
    assert_eq!(
        IvScenario::parse("stress_2x", Some(1)).unwrap(),
        IvScenario::Stress2x { after_minutes: 1 }
    );
    assert_eq!(
        IvScenario::parse("stress_3x", Some(1439))
            .unwrap()
            .shock_after_minutes(),
        Some(1439)
    );
}

#[test]
fn every_typed_error_reason_has_actionable_text() {
    for reason in [
        InvalidValue::NotFinite,
        InvalidValue::MustBePositive,
        InvalidValue::MustBeNonNegative,
        InvalidValue::Empty,
        InvalidValue::Unsupported,
        InvalidValue::Duplicate,
    ] {
        let error = DomainError::InvalidField {
            field: "example",
            reason,
        };
        assert!(error.to_string().contains("example"));
        assert!(!reason.to_string().is_empty());
    }
    let scenario_error = DomainError::InvalidScenario {
        index: 2,
        field: "scenario_id",
        reason: InvalidValue::Duplicate,
    };
    assert!(scenario_error.to_string().contains("index 2"));
    let length_error = DomainError::LengthMismatch {
        field: "close",
        expected: 2,
        actual: 1,
    };
    assert!(length_error.to_string().contains("expected 2, got 1"));
}
