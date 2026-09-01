pub mod autocomplete;
pub mod history;
pub mod macro_engine;
pub mod router;
pub mod telemetry;

#[allow(unused_imports)]
pub use autocomplete::{AutocompleteEngine, SuggestionItem, SuggestionKind};
#[allow(unused_imports)]
pub use history::HistoryManager;
#[allow(unused_imports)]
pub use macro_engine::MacroEngine;
#[allow(unused_imports)]
pub use router::{CommandRouter, CommandType, ParsedCommand};
#[allow(unused_imports)]
pub use telemetry::{MetricsHistory, TelemetryMetrics, TelemetryParser};
