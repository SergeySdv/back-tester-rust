use crate::{BacktestConfig, DatasetMetadata, IvScenario};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnSchema {
    pub name: &'static str,
    pub logical_dtype: &'static str,
    pub nullable: bool,
}

macro_rules! column {
    ($name:literal, $dtype:literal) => {
        ColumnSchema {
            name: $name,
            logical_dtype: $dtype,
            nullable: false,
        }
    };
    ($name:literal, $dtype:literal, nullable) => {
        ColumnSchema {
            name: $name,
            logical_dtype: $dtype,
            nullable: true,
        }
    };
}

pub const EQUITY_SCHEMA: &[ColumnSchema] = &[
    column!("timestamp_ns", "int64"),
    column!("scenario_id", "string"),
    column!("spot", "float64"),
    column!("active_trade_id", "uint64", nullable),
    column!("active_iv", "float64", nullable),
    column!("cash", "float64"),
    column!("option_liability", "float64"),
    column!("locked_margin", "float64"),
    column!("available_margin", "float64"),
    column!("equity", "float64"),
    column!("pnl", "float64"),
    column!("running_peak", "float64"),
    column!("drawdown_usd", "float64"),
    column!("drawdown_pct", "float64"),
    column!("margin_breached", "bool"),
];

pub const COMPLETED_TRADES_SCHEMA: &[ColumnSchema] = &[
    column!("scenario_id", "string"),
    column!("trade_id", "uint64"),
    column!("entry_timestamp_ns", "int64"),
    column!("expiry_timestamp_ns", "int64"),
    column!("strike", "float64"),
    column!("entry_equity", "float64"),
    column!("initial_quantity_steps", "uint64"),
    column!("initial_quantity", "float64"),
    column!("reserve_attempted", "bool"),
    column!("reserve_executed_quantity_steps", "uint64"),
    column!("reserve_executed_quantity", "float64"),
    column!("total_received_premium", "float64"),
    column!("settlement_spot", "float64"),
    column!("total_expiry_payoff", "float64"),
    column!("realized_pnl", "float64"),
    column!("margin_breached_during_trade", "bool"),
];

pub const EXECUTED_TRANCHES_SCHEMA: &[ColumnSchema] = &[
    column!("scenario_id", "string"),
    column!("trade_id", "uint64"),
    column!("tranche_id", "uint64"),
    column!("tranche_kind", "string"),
    column!("execution_timestamp_ns", "int64"),
    column!("expiry_timestamp_ns", "int64"),
    column!("strike", "float64"),
    column!("quantity_steps", "uint64"),
    column!("quantity", "float64"),
    column!("active_iv", "float64"),
    column!("call_premium_per_unit", "float64"),
    column!("put_premium_per_unit", "float64"),
    column!("total_premium_per_unit", "float64"),
    column!("received_premium", "float64"),
    column!("locked_margin", "float64"),
];

pub const RESERVE_ATTEMPTS_SCHEMA: &[ColumnSchema] = &[
    column!("scenario_id", "string"),
    column!("trade_id", "uint64"),
    column!("attempt_timestamp_ns", "int64"),
    column!("requested_quantity_steps", "uint64"),
    column!("executed_quantity_steps", "uint64"),
    column!("requested_quantity", "float64"),
    column!("executed_quantity", "float64"),
    column!("available_margin_before", "float64"),
    column!("reserve_budget_remaining", "float64"),
    column!("outcome", "string"),
    column!("reason", "string"),
];

pub const SUMMARY_SCHEMA: &[ColumnSchema] = &[
    column!("scenario_id", "string"),
    column!("terminal_status", "string"),
    column!("terminal_timestamp_ns", "int64"),
    column!("initial_equity", "float64"),
    column!("final_equity", "float64"),
    column!("total_pnl", "float64"),
    column!("return_pct", "float64"),
    column!("maximum_drawdown_usd", "float64"),
    column!("maximum_drawdown_pct", "float64"),
    column!("minimum_equity", "float64"),
    column!("minimum_available_margin", "float64"),
    column!("maximum_locked_margin", "float64"),
    column!("completed_trade_count", "uint64"),
    column!("reserve_attempt_count", "uint64"),
    column!("full_reserve_count", "uint64"),
    column!("reduced_reserve_count", "uint64"),
    column!("rejected_reserve_count", "uint64"),
    column!("skipped_incomplete_window_count", "uint8"),
    column!("input_row_count", "uint64"),
    column!("processed_row_count", "uint64"),
    column!("ignored_input_row_count", "uint64"),
    column!("any_margin_breach", "bool"),
];

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioCode {
    Baseline = 0,
    Stress2x = 1,
    Stress3x = 2,
}

impl ScenarioCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Stress2x => "stress_2x",
            Self::Stress3x => "stress_3x",
        }
    }
}

impl From<IvScenario> for ScenarioCode {
    fn from(value: IvScenario) -> Self {
        match value {
            IvScenario::Baseline => Self::Baseline,
            IvScenario::Stress2x { .. } => Self::Stress2x,
            IvScenario::Stress3x { .. } => Self::Stress3x,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrancheKind {
    Initial = 0,
    Reserve = 1,
}

impl TrancheKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Reserve => "reserve",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReserveOutcome {
    Full = 0,
    Reduced = 1,
    Rejected = 2,
}

impl ReserveOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Reduced => "reduced",
            Self::Rejected => "rejected",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReserveReason {
    None = 0,
    LimitedByAvailableMargin = 1,
    BelowQuantityStep = 2,
    NoAvailableMargin = 3,
}

impl ReserveReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LimitedByAvailableMargin => "limited_by_available_margin",
            Self::BelowQuantityStep => "below_quantity_step",
            Self::NoAvailableMargin => "no_available_margin",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalStatus {
    Completed = 0,
    CapitalExhausted = 1,
}

impl TerminalStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::CapitalExhausted => "capital_exhausted",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioMetadata {
    pub scenario_id: ScenarioCode,
    pub multiplier: f64,
    pub shock_after_minutes: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunMetadata {
    pub dataset: DatasetMetadata,
    pub config: BacktestConfig,
    pub initial_allocation: f64,
    pub reserve_allocation: f64,
    pub reserve_trigger_multiple: f64,
    pub scenarios: Vec<ScenarioMetadata>,
    pub pricing_model: &'static str,
    pub seconds_per_year: u64,
    pub software_version: &'static str,
    pub software_commit: Option<&'static str>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EquitySeries {
    pub timestamp_ns: Vec<i64>,
    pub scenario_id: Vec<ScenarioCode>,
    pub spot: Vec<f64>,
    pub active_trade_id: Vec<u64>,
    pub active_trade_id_valid: Vec<bool>,
    pub active_iv: Vec<f64>,
    pub active_iv_valid: Vec<bool>,
    pub cash: Vec<f64>,
    pub option_liability: Vec<f64>,
    pub locked_margin: Vec<f64>,
    pub available_margin: Vec<f64>,
    pub equity: Vec<f64>,
    pub pnl: Vec<f64>,
    pub running_peak: Vec<f64>,
    pub drawdown_usd: Vec<f64>,
    pub drawdown_pct: Vec<f64>,
    pub margin_breached: Vec<bool>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompletedTrades {
    pub scenario_id: Vec<ScenarioCode>,
    pub trade_id: Vec<u64>,
    pub entry_timestamp_ns: Vec<i64>,
    pub expiry_timestamp_ns: Vec<i64>,
    pub strike: Vec<f64>,
    pub entry_equity: Vec<f64>,
    pub initial_quantity_steps: Vec<u64>,
    pub initial_quantity: Vec<f64>,
    pub reserve_attempted: Vec<bool>,
    pub reserve_executed_quantity_steps: Vec<u64>,
    pub reserve_executed_quantity: Vec<f64>,
    pub total_received_premium: Vec<f64>,
    pub settlement_spot: Vec<f64>,
    pub total_expiry_payoff: Vec<f64>,
    pub realized_pnl: Vec<f64>,
    pub margin_breached_during_trade: Vec<bool>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecutedTranches {
    pub scenario_id: Vec<ScenarioCode>,
    pub trade_id: Vec<u64>,
    pub tranche_id: Vec<u64>,
    pub tranche_kind: Vec<TrancheKind>,
    pub execution_timestamp_ns: Vec<i64>,
    pub expiry_timestamp_ns: Vec<i64>,
    pub strike: Vec<f64>,
    pub quantity_steps: Vec<u64>,
    pub quantity: Vec<f64>,
    pub active_iv: Vec<f64>,
    pub call_premium_per_unit: Vec<f64>,
    pub put_premium_per_unit: Vec<f64>,
    pub total_premium_per_unit: Vec<f64>,
    pub received_premium: Vec<f64>,
    pub locked_margin: Vec<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReserveAttempts {
    pub scenario_id: Vec<ScenarioCode>,
    pub trade_id: Vec<u64>,
    pub attempt_timestamp_ns: Vec<i64>,
    pub requested_quantity_steps: Vec<u64>,
    pub executed_quantity_steps: Vec<u64>,
    pub requested_quantity: Vec<f64>,
    pub executed_quantity: Vec<f64>,
    pub available_margin_before: Vec<f64>,
    pub reserve_budget_remaining: Vec<f64>,
    pub outcome: Vec<ReserveOutcome>,
    pub reason: Vec<ReserveReason>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScenarioSummary {
    pub scenario_id: Vec<ScenarioCode>,
    pub terminal_status: Vec<TerminalStatus>,
    pub terminal_timestamp_ns: Vec<i64>,
    pub initial_equity: Vec<f64>,
    pub final_equity: Vec<f64>,
    pub total_pnl: Vec<f64>,
    pub return_pct: Vec<f64>,
    pub maximum_drawdown_usd: Vec<f64>,
    pub maximum_drawdown_pct: Vec<f64>,
    pub minimum_equity: Vec<f64>,
    pub minimum_available_margin: Vec<f64>,
    pub maximum_locked_margin: Vec<f64>,
    pub completed_trade_count: Vec<u64>,
    pub reserve_attempt_count: Vec<u64>,
    pub full_reserve_count: Vec<u64>,
    pub reduced_reserve_count: Vec<u64>,
    pub rejected_reserve_count: Vec<u64>,
    pub skipped_incomplete_window_count: Vec<u8>,
    pub input_row_count: Vec<u64>,
    pub processed_row_count: Vec<u64>,
    pub ignored_input_row_count: Vec<u64>,
    pub any_margin_breach: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BacktestResult {
    pub metadata: RunMetadata,
    pub equity: EquitySeries,
    pub completed_trades: CompletedTrades,
    pub executed_tranches: ExecutedTranches,
    pub reserve_attempts: ReserveAttempts,
    pub summary: ScenarioSummary,
}
