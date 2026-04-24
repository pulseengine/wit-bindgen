include!(env!("BINDINGS"));

use exports::test::resource_floats_opaque::chain::{Guest as ChainGuest, GuestFloat as ChainGuestFloat};
use exports::test::resource_floats_opaque::test::{Guest as TestGuest, GuestFloat as TestGuestFloat};

struct Component;

export!(Component);

// Leaf uses the standard (non-opaque) resource pattern — it owns the data.
#[derive(Default)]
pub struct MyFloat(f64);

impl TestGuest for Component {
    type Float = MyFloat;
}

impl TestGuestFloat for MyFloat {
    fn new(v: f64) -> MyFloat {
        MyFloat(v + 1.0)
    }

    fn get(&self) -> f64 {
        self.0 + 3.0
    }
}

impl ChainGuest for Component {
    type Float = MyFloat;
}

impl ChainGuestFloat for MyFloat {
    fn new(v: f64) -> MyFloat {
        MyFloat(v + 2.0)
    }
}
