//! # Examples
//! ```
//! ///
//! /// Use to return staticly known errors:
//! fn static_error(n: i32) -> Result<(), StrErr> {
//!     match n {
//!         0 => {
//!             let err = format!("separate error message with param: {}", n);
//!             Err(StrErr::from(err))
//!         }
//!         1 => {
//!             let err_str = "separate error message with no params";
//!             Err(StrErr::from(err_str))
//!         }
//!         2 => Err(StrErr::from(format!("inline error message with params: {}", n))),
//!         3 => Err(StrErr::from("inline error message as string literal")),
//!         // calling of another fn, which may return Err(StrErr)
//!         4 => static_error(n),
//!         // ...
//!         _ => todo!(),
//!     }
//! }
//! ///
//! /// Use to return boxed errors:
//! fn dynamic_error(n: i32) -> Result<(), Box<dyn std::error::Error>> {
//!     match n {
//!         0 => {
//!             let err = format!("separate error message with param: {}", n);
//!             Err(StrErr::from(err).into())
//!         }
//!         1 => {
//!             let err_str = "separate error message w/o params";
//!             Err(StrErr::from(err_str).into())
//!         }
//!         2 => Err(StrErr::from(format!("inline with param: {}", n)).into()),
//!         3 => Err(StrErr::from("inline witn no params").into()),
//!         // calling of another fn, which may return Box<dyn std::error::Error>
//!         4 => dynamic_error(n),
//!         // note that it needs to be coerced as usual
//!         5 => Ok(static_error(n)?),
//!         // ...
//!         _ => todo!(),
//!     }
//! }
//! ```
///
/// Error wrapper for owned string and string literal.
#[repr(transparent)]
pub struct StrErr(std::borrow::Cow<'static, str>);
//
//
impl std::fmt::Debug for StrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
//
//
impl std::fmt::Display for StrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
//
//
impl std::error::Error for StrErr {}
//
//
impl<T: Into<std::borrow::Cow<'static, str>>> From<T> for StrErr {
    fn from(err_str: T) -> Self {
        Self(err_str.into())
    }
}
