#![cfg_attr(not(test), no_std)]

#[cfg(feature = "std")]
extern crate std;

pub mod q2_32;

pub fn testing() -> usize {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        // panic!();
        assert_eq!(testing(), 0);
    }
}
