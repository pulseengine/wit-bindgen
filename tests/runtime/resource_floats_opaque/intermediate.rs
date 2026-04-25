//@ args = '--opaque-export-resources float'

include!(env!("BINDINGS"));

use exports::test::resource_floats_opaque::chain::{Guest, GuestFloat};
use test::resource_floats_opaque::chain::Float as ImportFloat;

struct Component;

export!(Component);

impl Guest for Component {
    // Opaque-rep associated type: `()` because we never construct a real
    // wrapper struct. The trait method `new(v: f64) -> u32` returns just
    // the rep (here, the inner-component handle). Methods receive a
    // raw `u32` handle as the first parameter (instead of `&self`).
    type Float = ();
}

impl GuestFloat for () {
    /// Re-exporter constructor in the opaque-rep style.
    ///
    /// Builds an inner `Float` in the leaf component and forwards its
    /// handle as the exported rep. No `Box::new`, no `*const Self`,
    /// no `assume(ptr.is_aligned())` debug check — the rep is just
    /// the leaf's handle as a `u32`.
    ///
    /// After meld fuses the chain, the rep stored in intermediate's
    /// per-component handle table is exactly this `u32`. No
    /// dereference happens anywhere in user code, so meld's
    /// existing handle-table machinery is sufficient.
    fn new(v: f64) -> u32 {
        let inner = ImportFloat::new(v + 1.0);
        inner.take_handle()
    }

    /// Re-exporter method in the opaque-rep style.
    ///
    /// The trait signature for opaque-rep takes a raw `u32` handle as
    /// the first parameter instead of `&self`. The user code wraps it
    /// in an `ImportFloat` (no clone — `from_handle` takes ownership of
    /// the handle), forwards the call to the inner component, then
    /// `take_handle()` the inner Float to suppress its `Drop` so the
    /// inner-component handle table isn't decremented (the outer handle
    /// is still alive on the caller side).
    fn get(handle: u32) -> f64 {
        let inner = unsafe { ImportFloat::from_handle(handle) };
        let result = inner.get();
        let _ = inner.take_handle();
        result
    }
}
