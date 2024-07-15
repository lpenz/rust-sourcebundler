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
pub mod submod3 {
pub fn hello_world3__long_identifier_name_for_multiline() {
    println!("Hello, world 3!");
}
}
pub mod submod4 {
use super::{    submod1,    submod3::hello_world3__long_identifier_name_for_multiline,};
pub fn hello_world4() {
    submod1::hello_world1();
    hello_world3__long_identifier_name_for_multiline();
    println!("Hello, world 4!");
}
}
}
use self::usecrate::submod2::hello_world2;
use self::usecrate::submod4::hello_world4;
fn main() {
    hello_world2();
    hello_world4();
}
