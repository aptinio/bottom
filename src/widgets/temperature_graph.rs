//! Code around a temperature graph widget.

use std::time::Instant;

use crate::components::time_series::{AutoYAxisTimeGraph, TimeseriesConfig};

/// A time series graph widget displaying temperature usage over time.
pub struct TempGraphWidgetState {
    pub graph: AutoYAxisTimeGraph,
    pub min_temp: Option<f64>,
    pub max_temp: Option<f64>,
}

impl TempGraphWidgetState {
    pub fn new(
        config: TimeseriesConfig, autohide_timer: Option<Instant>, min_temp: Option<f64>,
        max_temp: Option<f64>,
    ) -> Self {
        TempGraphWidgetState {
            graph: AutoYAxisTimeGraph::new(config, autohide_timer),
            min_temp,
            max_temp,
        }
    }
}
