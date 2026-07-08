pub mod proto_spec;
pub mod urlx;
pub(crate) mod utils;

pub use urlx::SchemeX;
pub(crate) use urlx::PortSpec;

macro_rules! nom_bail {
    ($input: expr, $code: ident) => {{
        return Err(nom::Err::Error(nom::error::Error::new(
            $input,
            nom::error::ErrorKind::$code,
        )));
    }};
}
pub(crate) use nom_bail;
