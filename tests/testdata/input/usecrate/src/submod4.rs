use crate::{
    submod1,
    submod3::hello_world3__long_identifier_name_for_multiline,
};

pub fn hello_world4() {
    submod1::hello_world1();
    hello_world3__long_identifier_name_for_multiline();
    println!("Hello, world 4!");
}
