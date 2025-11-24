//! Data Transfer Objects (DTOs) module.
//!
//! DTOs serve as contracts between layers, preventing tight coupling and allowing
//! each layer to evolve independently. DTOs are simple data structures without
//! business logic, designed for efficient data transfer.
//!
//! Module organization:
//! - `input_action`: Input event DTOs (decouples from crossterm::Event)
//! - `loop_state`: Loop state DTOs (decouples UI from domain LoopState)
//! - `ui_state`: UI-specific state DTOs (for presentation layer consumption)
//!
//! DTOs should:
//! - Be serializable/cloneable
//! - Have conversion logic to/from domain types
//! - Not contain business logic
//! - Not depend on presentation or infrastructure layers

pub mod input_action;
pub mod loop_state;
pub mod ui_state;

