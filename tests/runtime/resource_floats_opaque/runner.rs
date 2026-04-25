include!(env!("BINDINGS"));

use test::resource_floats_opaque::chain::Float as ReExportedFloat;

struct Component;

export!(Component);

impl Guest for Component {
    fn run() {
        // Exercise opaque-rep through a 3-component chain:
        //   1. runner -> intermediate.chain.float-constructor
        //   2. intermediate -> leaf chain.float-constructor (forwards handle)
        //   3. runner -> intermediate.chain.float.get (borrow forwarding)
        //   4. intermediate -> leaf chain.float.get
        //   5. drop fires (no-op opaque dtor; leaf cleans up at teardown)
        //
        // Each constructor adds 1.0 (intermediate) + 2.0 (leaf) = +3.0 to v.
        // Each get() adds 3.0 in leaf. So input 42.0 -> get -> 48.0.
        let f1 = ReExportedFloat::new(42.0);
        let v1 = f1.get();
        assert_eq!(v1, 48.0);

        let f2 = ReExportedFloat::new(7.0);
        let v2 = f2.get();
        assert_eq!(v2, 13.0);

        drop(f1);
        drop(f2);
    }
}
