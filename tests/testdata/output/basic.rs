pub mod basic {
pub mod internal {
pub fn hello_world() {
    println!("Hello, world!");
}
}
pub fn hello_external_world() {
    println!("Hello, external world!");
}
}
use ::basic::hello_external_world;
use ::basic::internal::hello_world;
fn main() {
    hello_world();
    hello_external_world();
}
