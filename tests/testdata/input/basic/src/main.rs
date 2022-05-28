extern crate basic;
use basic::hello_external_world;
use basic::internal::hello_world;

fn main() {
    hello_world();
    hello_external_world();
}
