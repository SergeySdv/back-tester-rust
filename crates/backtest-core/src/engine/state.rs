use crate::result::{ScenarioCode, TerminalStatus};

#[derive(Debug, Clone)]
pub(super) struct ActiveTranche {
    pub(super) quantity_steps: u64,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveTrade {
    pub(super) trade_id: u64,
    pub(super) entry_index: usize,
    pub(super) expiry_index: usize,
    pub(super) strike: f64,
    pub(super) entry_equity: f64,
    pub(super) initial_quantity_steps: u64,
    pub(super) reserve_attempted: bool,
    pub(super) reserve_executed_steps: u64,
    pub(super) total_received_premium: f64,
    pub(super) margin_breached: bool,
    pub(super) tranches: Vec<ActiveTranche>,
}

pub(super) struct ScenarioState {
    pub(super) code: ScenarioCode,
    pub(super) cash: f64,
    pub(super) liability: f64,
    pub(super) locked_margin: f64,
    pub(super) running_peak: f64,
    pub(super) maximum_drawdown_usd: f64,
    pub(super) maximum_drawdown_pct: f64,
    pub(super) minimum_equity: f64,
    pub(super) minimum_available_margin: f64,
    pub(super) maximum_locked_margin: f64,
    pub(super) any_margin_breach: bool,
    pub(super) next_trade_id: u64,
    pub(super) next_tranche_id: u64,
    pub(super) active_trade: Option<ActiveTrade>,
    pub(super) terminal_status: TerminalStatus,
    pub(super) processed_rows: usize,
    pub(super) skipped_tail: u8,
    pub(super) full_reserves: u64,
    pub(super) reduced_reserves: u64,
    pub(super) rejected_reserves: u64,
}

impl ScenarioState {
    pub(super) fn new(code: ScenarioCode, initial_capital: f64) -> Self {
        Self {
            code,
            cash: initial_capital,
            liability: 0.0,
            locked_margin: 0.0,
            running_peak: initial_capital,
            maximum_drawdown_usd: 0.0,
            maximum_drawdown_pct: 0.0,
            minimum_equity: initial_capital,
            minimum_available_margin: initial_capital,
            maximum_locked_margin: 0.0,
            any_margin_breach: false,
            next_trade_id: 0,
            next_tranche_id: 0,
            active_trade: None,
            terminal_status: TerminalStatus::Completed,
            processed_rows: 0,
            skipped_tail: 0,
            full_reserves: 0,
            reduced_reserves: 0,
            rejected_reserves: 0,
        }
    }

    pub(super) fn equity(&self) -> f64 {
        self.cash - self.liability
    }

    pub(super) fn available_margin(&self) -> f64 {
        self.equity() - self.locked_margin
    }
}
