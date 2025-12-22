//! Post-action execution module

mod executor;
mod input;

pub use executor::execute_post_actions;
pub use executor::execute_with_post_actions;
