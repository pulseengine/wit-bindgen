include!(env!("BINDINGS"));

use exports::test::resource_floats_opaque::chain::{Guest as ChainGuest, GuestFloat as ChainGuestFloat};

struct Component;

export!(Component);

// Leaf uses the standard (non-opaque) resource pattern — it owns the data.
pub struct MyFloat(f64);

impl ChainGuest for Component {
    type Float = MyFloat;
}

impl ChainGuestFloat for MyFloat {
    fn new(v: f64) -> MyFloat {
        MyFloat(v + 2.0)
    }

    fn get(&self) -> f64 {
        self.0 + 3.0
    }
}
