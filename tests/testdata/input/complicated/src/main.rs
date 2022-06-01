extern crate complicated;

use complicated::{a, b, c};

fn main() {
    a::a();
    self::b::b();
    self::c::d::d();
}
