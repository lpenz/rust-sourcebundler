pub mod basic_test {
pub mod internal {
pub fn hello_world() {
    println!("Hello, world!");
}
}
pub fn hello_external_world() {
    println!("Hello, external world!");
}
}
use ::basic_test::hello_external_world;
use ::basic_test::internal::hello_world;
fn main() {
    hello_world();
    hello_external_world();
}
