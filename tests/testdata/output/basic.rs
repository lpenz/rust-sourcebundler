pub mod internal {
pub fn hello_world() {
    println!("Hello, world!");
}
}
use internal::hello_world;
fn main() {
    hello_world();
}
