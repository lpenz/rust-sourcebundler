extern crate basic_test;
use basic_test::hello_external_world;
use basic_test::internal::hello_world;

fn main() {
    hello_world();
    hello_external_world();
}
