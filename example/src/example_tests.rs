#[cfg(test)]
use example_core::example_hello;

#[test]
fn test_example() {
    assert_eq!(example_hello(), "Hello example!");
}
