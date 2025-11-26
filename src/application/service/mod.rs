//! Application service layer.
//!
//! This module contains stateless application services that orchestrate use cases
//! by coordinating domain logic and producing effects. Services receive state as
//! parameters and return new state along with side effects, ensuring they remain
//! reusable and testable.
//!
//! Key principles:
//! - Services are stateless (no internal state)
//! - Services receive state as parameters
//! - Services return new state and effects
//! - Services coordinate domain logic but don't contain business logic
//! - Services produce DTOs for presentation layer consumption

pub mod app_service;
pub mod effect;

pub use app_service::AppService;
pub use effect::Effect;
