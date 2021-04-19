// This code (in fact, this library) is heavily inspired by the dotnet Decimal number library
// implementation. Consequently, a huge thank you for to all the contributors to that project
// whose work has also inspired the solutions found here.

//#[cfg(feature = "legacy-ops")]
mod legacy;
#[cfg(feature = "legacy-ops")]
pub(crate) use legacy::{add_impl, div_impl, mul_impl, rem_impl, sub_impl};

#[cfg(not(feature = "legacy-ops"))]
mod div;
#[cfg(not(feature = "legacy-ops"))]
mod mul;

#[cfg(not(feature = "legacy-ops"))]
pub(crate) use div::div_impl;
#[cfg(not(feature = "legacy-ops"))]
pub(crate) use legacy::{add_impl, rem_impl, sub_impl};
#[cfg(not(feature = "legacy-ops"))]
pub(crate) use mul::mul_impl;
