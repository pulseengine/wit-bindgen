# RFC: opaque-rep method support

## Status

Draft — design discussion before implementation.

## Background

The `--opaque-export-resources <name>...` opt-in (commit `c7550718`) makes the
Rust generator emit a stripped-down wrapper for re-exporter resources whose
representation is treated as opaque (a `u32` rather than a `Box<Option<T>>`
pointer). This sidesteps the `Box::into_raw` + `assume(ptr.is_aligned())`
debug-assertion chain that triggers `wasm unreachable` traps under
cross-component static fusion (e.g. with [meld](https://github.com/pulseengine/meld)).

The current opt-in covers:

- Constructors (`new(args) -> u32`)
- Drop (no-op `dtor`)

It does NOT cover:

- Methods (`fn get(&self) -> f64`)
- Static methods that return `own<Self>` from a different code path

The `resource_floats_opaque` test fixture is constructor-only because of this.

## The problem

Standard wit-bindgen-rust emits this trait shape for an exported resource
with a method:

```rust
pub trait GuestFloat: 'static + Sized {
    fn new(v: f64) -> Self;
    fn get(&self) -> f64;
}
```

The `&self` has type `&Self` — for non-opaque resources, `Self` is the user's
struct (e.g. `MyFloat`) and `&self` directly accesses fields. The cabi-export
shim does `<MyFloat as GuestFloat>::get(borrow.get::<MyFloat>())` where
`Borrow::get` calls `_resource_rep(handle)` and casts the resulting pointer
to `&MyFloat`.

For opaque-rep, the user picks `Self = ()` (or any unit type). Then `&self`
is `&()` and contains no usable information. There is nowhere for the user
code to obtain the inner-component handle to forward the method call.

## Design space

### Option A — raw `u32` borrow

Trait method takes a raw `u32` handle as the first parameter:

```rust
pub trait GuestFloat: 'static + Sized {
    fn new(v: f64) -> u32;
    fn get(handle: u32) -> f64;
}

impl GuestFloat for () {
    fn new(v: f64) -> u32 {
        let inner = ImportFloat::new(v + 1.0);
        inner.take_handle()
    }
    fn get(handle: u32) -> f64 {
        let inner = unsafe { ImportFloat::from_handle(handle) };
        let result = inner.get();
        inner.take_handle();   // suppress drop
        result
    }
}
```

**Pros:** simple generator change (just emit `u32` instead of `&self` for
opaque resources). User has full control. Maps cleanly to the canonical
ABI (handles ARE u32s).

**Cons:** unsafe required to materialise `ImportFloat::from_handle`. Easy
to leak the inner handle if user forgets `.take_handle()`. Doesn't compose
with arbitrary user types as `Self`.

### Option B — typed `Borrow<Self>` with handle access

Trait method keeps `&self` shape but the generator emits a borrow type that
exposes the handle:

```rust
pub trait GuestFloat: 'static + Sized {
    fn new(v: f64) -> u32;
    fn get(borrow: &OpaqueBorrow<Self>) -> f64;
}

pub struct OpaqueBorrow<T> { handle: u32, _ph: PhantomData<T> }
impl<T> OpaqueBorrow<T> {
    pub fn handle(&self) -> u32 { self.handle }
}
```

**Pros:** typed, explicit. No `unsafe` needed at user code if combined with
a safe `ImportFloat::borrow_from_handle(b.handle())` helper.

**Cons:** requires a new generator type. Diverges from the standard
`Borrow<'_, T>` API. Two parallel borrow types in the same crate may
confuse users.

### Option C — `&self` with internal handle accessor

Keep the standard `Borrow<'_, T>` API but add a public `.handle()` method
that returns the raw `u32` handle. For opaque-rep, the user code would
manually use this:

```rust
pub trait GuestFloat: 'static + Sized {
    fn new(v: f64) -> u32;   // (already opaque — returns u32 not Self)
    fn get(&self) -> f64;
}

impl GuestFloat for () {
    fn new(v: f64) -> u32 { ... }
    fn get(&self) -> f64 {
        let h = OpaqueRepHandle::current();    // ???
        // Problem: how does &self → handle?
    }
}
```

**Cons:** doesn't actually work. `&self` for `Self = ()` literally has no
runtime information.

### Recommendation

**Option A** is the simplest to implement and reason about. The unsafe is
localised and the trait shape is mechanically reproducible. The user is
explicitly opting into opaque-rep so they're already accepting some loss
of safety in exchange for fusion compatibility.

## Implementation sketch (Option A)

In `crates/rust/src/interface.rs`, when emitting the trait method signature
for a resource that is in `opts.opaque_export_resources`:

1. Replace the `&self` first parameter with `handle: u32`.
2. (Static methods and constructors are unchanged — they don't have `&self`.)

In `crates/rust/src/bindgen.rs::Instruction::CallExport` (around line 1036):

1. When the resource is opaque-rep, emit `T_::method(handle, ...)` instead
   of `T_::method(borrow.get::<T>(), ...)` — the operand is already the
   raw u32 handle (no `Borrow` wrapper produced).
2. Skip the `Borrow::new` wrap that produces `borrow` in the first place.

In the test fixture `tests/runtime/resource_floats_opaque/`:

1. Extend `chain` interface in `test.wit` to include `get: func() -> f64`.
2. Update `intermediate.rs` to implement `fn get(handle: u32) -> f64`.
3. Update `runner.rs` to call `f1.get()`.

## Validation

- `cargo run --bin wit-bindgen test --languages rust tests/runtime/resource_floats_opaque` passes.
- `meld fuse <composed>.wasm -o <fused>.wasm --component` produces a valid component.
- `wasmtime --invoke='run()' <fused>.wasm` runs without trap.

## Drop semantics (orthogonal but related)

Opaque-rep currently has a no-op `dtor`. For correctness, the inner-
component handle owned by the rep must be explicitly dropped when the
outer handle drops. A symmetric design question: should the opt-in also
allow user-supplied `dtor` impls?

```rust
impl GuestFloat for () {
    fn dtor(handle: u32) {
        let inner = unsafe { ImportFloat::from_handle(handle) };
        drop(inner);  // calls leaf's [resource-drop]float
    }
}
```

Suggested as a follow-up RFC after methods land.

## Open questions for upstream

1. Is `Option A` (raw `u32`) acceptable, or would `Option B` (typed borrow)
   be preferred for safety?
2. Should the opt-in flag affect imported resources too (currently only
   exported)? Would symmetric opacity simplify any patterns?
3. How does this interact with the async resource pattern?
