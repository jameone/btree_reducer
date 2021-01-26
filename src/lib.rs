#![no_std]
extern crate alloc;

mod arrangement;
mod breducer;
mod contact;
mod xor;

#[cfg(test)]
mod unit_tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
