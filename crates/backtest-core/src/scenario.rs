use crate::{DomainError, InvalidValue};

pub const MIN_STRESS_MINUTE: u16 = 1;
pub const MAX_STRESS_MINUTE: u16 = 1439;

/// Closed scenario set supported by the MVP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IvScenario {
    Baseline,
    Stress2x { after_minutes: u16 },
    Stress3x { after_minutes: u16 },
}

impl IvScenario {
    pub fn parse(scenario_id: &str, shock_after_minutes: Option<u16>) -> Result<Self, DomainError> {
        match (scenario_id, shock_after_minutes) {
            ("baseline", None) => Ok(Self::Baseline),
            ("baseline", Some(_)) => Err(DomainError::InvalidField {
                field: "scenario.shock_after_minutes",
                reason: InvalidValue::Unsupported,
            }),
            ("stress_2x", Some(after_minutes)) => Ok(Self::Stress2x { after_minutes }),
            ("stress_3x", Some(after_minutes)) => Ok(Self::Stress3x { after_minutes }),
            ("stress_2x" | "stress_3x", None) => Err(DomainError::InvalidField {
                field: "scenario.shock_after_minutes",
                reason: InvalidValue::Empty,
            }),
            _ => Err(DomainError::InvalidField {
                field: "scenario.scenario_id",
                reason: InvalidValue::Unsupported,
            }),
        }
    }

    pub const fn scenario_id(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Stress2x { .. } => "stress_2x",
            Self::Stress3x { .. } => "stress_3x",
        }
    }

    pub const fn multiplier(self) -> f64 {
        match self {
            Self::Baseline => 1.0,
            Self::Stress2x { .. } => 2.0,
            Self::Stress3x { .. } => 3.0,
        }
    }

    pub const fn shock_after_minutes(self) -> Option<u16> {
        match self {
            Self::Baseline => None,
            Self::Stress2x { after_minutes } | Self::Stress3x { after_minutes } => {
                Some(after_minutes)
            }
        }
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Baseline => 0,
            Self::Stress2x { .. } => 1,
            Self::Stress3x { .. } => 2,
        }
    }
}

/// Validate and return scenarios in the canonical public order.
pub fn validate_scenarios(scenarios: &[IvScenario]) -> Result<Vec<IvScenario>, DomainError> {
    if scenarios.is_empty() {
        return Err(DomainError::InvalidField {
            field: "scenarios",
            reason: InvalidValue::Empty,
        });
    }

    let mut canonical = scenarios.to_vec();
    canonical.sort_by_key(|scenario| scenario.rank());
    for (index, scenario) in canonical.iter().enumerate() {
        if let Some(after_minutes) = scenario.shock_after_minutes()
            && !(MIN_STRESS_MINUTE..=MAX_STRESS_MINUTE).contains(&after_minutes)
        {
            return Err(DomainError::InvalidScenario {
                index,
                field: "shock_after_minutes",
                reason: InvalidValue::Unsupported,
            });
        }
        if index > 0 && canonical[index - 1].rank() == scenario.rank() {
            return Err(DomainError::InvalidScenario {
                index,
                field: "scenario_id",
                reason: InvalidValue::Duplicate,
            });
        }
    }
    Ok(canonical)
}
