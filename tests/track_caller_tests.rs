//! Verifies that panics originating from `Decimal`'s arithmetic operators are
//! attributed to the caller's source location.
//!
//! The operators (and their `*Assign` counterparts) are annotated with
//! `#[track_caller]`, so a panic such as a division by zero should report the
//! location of the offending expression in user code rather than a line inside
//! `rust_decimal`'s own source. See <https://github.com/paupino/rust-decimal/issues/733>.
//!
//! This lives in its own integration test binary because it installs a process
//! wide panic hook; keeping it isolated avoids interfering with the panic
//! behaviour of any other test.

use rust_decimal::Decimal;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Mutex, OnceLock};

fn captured_location() -> &'static Mutex<Option<(String, u32)>> {
    static LOCATION: OnceLock<Mutex<Option<(String, u32)>>> = OnceLock::new();
    LOCATION.get_or_init(|| Mutex::new(None))
}

/// Runs `operation`, which is expected to panic, and returns the `(file, line)`
/// that the panic was attributed to.
fn location_of_panic(operation: impl FnOnce()) -> (String, u32) {
    *captured_location().lock().unwrap() = None;
    let result = panic::catch_unwind(AssertUnwindSafe(operation));
    assert!(result.is_err(), "expected the operation to panic, but it did not");
    captured_location()
        .lock()
        .unwrap()
        .clone()
        .expect("panic hook did not record a location")
}

#[test]
fn arithmetic_panics_point_at_the_caller() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        if let Some(location) = info.location() {
            *captured_location().lock().unwrap() = Some((location.file().to_string(), location.line()));
        }
    }));

    // Every operation checked below is expected to panic. Because the operators
    // carry `#[track_caller]`, the reported file must be this test file (the
    // caller), never `src/arithmetic_impls.rs` or `src/decimal.rs`.
    let mut misattributed = Vec::new();
    let mut check = |name: &str, operation: fn()| {
        let (file, line) = location_of_panic(operation);
        if !file.ends_with("track_caller_tests.rs") {
            misattributed.push(format!("{name}: attributed to {file}:{line}"));
        }
    };

    check("addition overflow", || {
        let _ = Decimal::MAX + Decimal::MAX;
    });
    check("subtraction overflow", || {
        let _ = Decimal::MIN - Decimal::MAX;
    });
    check("multiplication overflow", || {
        let _ = Decimal::MAX * Decimal::MAX;
    });
    check("division by zero", || {
        let _ = Decimal::ONE / Decimal::ZERO;
    });
    check("remainder by zero", || {
        let _ = Decimal::ONE % Decimal::ZERO;
    });
    check("add-assign overflow", || {
        let mut value = Decimal::MAX;
        value += Decimal::MAX;
    });
    check("div-assign by zero", || {
        let mut value = Decimal::ONE;
        value /= Decimal::ZERO;
    });

    panic::set_hook(default_hook);

    assert!(
        misattributed.is_empty(),
        "expected every arithmetic panic to be attributed to the caller, but the \
         following were not:\n{}",
        misattributed.join("\n"),
    );
}
