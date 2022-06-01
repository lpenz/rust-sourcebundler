pub mod usecrate {
pub mod submod1 {
pub fn hello_world1() {
    println!("Hello, world 1!");
}
}
pub mod submod2 {
use super::submod1;
pub fn hello_world2() {
    submod1::hello_world1();
    println!("Hello, world 2!");
}
}
}
use usecrate::submod2::hello_world2;
fn main() {
    hello_world2();
}
