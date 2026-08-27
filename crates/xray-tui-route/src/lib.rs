//! First-match routing engine primitives (addr/error foundation).

pub mod addr;
pub mod error;
pub mod ir;
pub mod matchers;

pub use addr::{Cidr, NetAddr, NetHost, PortRange};
pub use error::RouteError;
