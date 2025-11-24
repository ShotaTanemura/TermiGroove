//! Application layer module.
//!
//! The application layer orchestrates domain logic and provides DTOs (Data Transfer Objects)
//! to decouple layers. It acts as an intermediary between the presentation layer (UI) and
//! the domain layer, translating between domain entities and presentation concerns.
//!
//! Key responsibilities:
//! - Define DTOs for data transfer between layers
//! - Provide application services (to be implemented in future tasks)
//! - Coordinate domain logic execution
//!
//! Layer boundaries:
//! - Depends on: Domain layer (`crate::domain`)
//! - Does not depend on: Presentation layer (`crate::ui`), Infrastructure layer (`crate::audio`)

pub mod dto;
