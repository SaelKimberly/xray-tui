//! First-match routing engine primitives (addr/error foundation).

pub mod addr;
pub mod compiler;
pub mod engine;
pub mod error;
pub mod events;
pub mod ir;
pub mod matchers;

pub use addr::{Cidr, NetAddr, NetHost, PortRange};
pub use compiler::{CompileOutput, compile_xray};
pub use engine::{ConnMeta, Decision, Engine};
pub use error::RouteError;
pub use events::RouteEvent;
