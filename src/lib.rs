#![no_std]
extern crate alloc;

/// `Error` type is re-exported from the separate btree_error crate.
pub type Error = btree_error::Error;

mod reducer;

#[cfg(test)]
mod unit_tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
