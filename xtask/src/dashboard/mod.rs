pub mod state;
pub mod ui;

pub use state::{DashboardState, ErrorStats, EventSeverity, RecentEvent, StateManager};
pub use ui::render_dashboard; 