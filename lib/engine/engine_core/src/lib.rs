//! Core simulation logic for **SOUL** – world representation, CPU
//! steppers, high‑level state resources, and GPU back‑end re‑exports.
pub mod core;

pub mod state;
pub mod engine;

pub mod schedule;
pub mod events;
pub mod systems;