//! First-match routing engine primitives (addr/error foundation).

pub mod addr;
pub mod engine;
pub mod error;
pub mod events;
pub mod ir;
pub mod matchers;

pub use engine::{ConnMeta, Decision, Engine};
pub use events::RouteEvent;
pub use addr::{Cidr, NetAddr, NetHost, PortRange};
pub use error::RouteError;
