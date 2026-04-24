include!(env!("BINDINGS"));

use test::resource_floats_opaque::chain::Float as ReExportedFloat;

struct Component;

export!(Component);

impl Guest for Component {
    fn run() {
        // Construct an exported (opaque-rep re-exporter) Float, then drop
        // it. This exercises:
        //   1. runner -> intermediate.exports.float-constructor
        //   2. intermediate -> imports.float-constructor (= leaf)
        //   3. leaf returns inner handle, intermediate returns it as rep
        //   4. runner drops the float
        //   5. intermediate's drop fires (no-op `dtor` for opaque-rep)
        //   6. leaf's drop fires for the inner float
        //
        // No method calls — methods on opaque-rep resources need further
        // generator work (see intermediate.rs comments). Constructor +
        // drop is the load-bearing case for proving meld can fuse a
        // 3-component chain without trapping.
        let f1 = ReExportedFloat::new(42.0);
        let f2 = ReExportedFloat::new(7.0);
        drop(f1);
        drop(f2);
    }
}
