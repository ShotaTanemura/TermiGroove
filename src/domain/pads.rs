//! Pad domain logic module.
//!
//! This module will contain domain entities, value objects, and business logic
//! related to pad mapping and sample slots.
//!
//! Domain concepts:
//! - Pad key mappings
//! - Sample slot assignments
//! - Pad activation and debouncing logic

/// Default pad keys for mapping samples.
pub fn default_pad_keys() -> &'static [char] {
    &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i']
}

