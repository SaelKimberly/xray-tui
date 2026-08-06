mod port_spec;
mod schemex;
mod split_url;

pub type TinyText = smartstring::SmartString<smartstring::LazyCompact>;
pub(crate) type HostSpec = rustls::pki_types::ServerName<'static>;

pub(crate) use port_spec::PortSpec;

pub use schemex::SchemeX;
pub use split_url::RawUrlX;
