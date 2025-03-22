// This code (in fact, this library) is heavily inspired by the dotnet Decimal number library
// implementation. Consequently, a huge thank you for to all the contributors to that project
// whose work has also inspired the solutions found here.

pub(crate) mod array;

mod add;
mod cmp;
pub(in crate::ops) mod common;
mod div;
mod mul;
mod rem;

pub(crate) use add::{add_impl, sub_impl};
pub(crate) use cmp::cmp_impl;
pub(crate) use div::div_impl;
pub(crate) use mul::mul_impl;
pub(crate) use rem::rem_impl;
