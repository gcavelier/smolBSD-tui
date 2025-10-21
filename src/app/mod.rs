mod args;
mod state;

pub use state::State;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
