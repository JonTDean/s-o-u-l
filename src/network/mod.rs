//! Networking domain (stub).
//!
//! Split into *server* and *client* until we figure out authoritive vs
//! peer-to-peer sync.  Each sub-plugin will tag its own systems with the
//! correct `SystemSet` (most likely `MainSet::Input` and/or `Logic`).

pub mod server;
pub mod client;
