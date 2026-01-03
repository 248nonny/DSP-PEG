#![cfg_attr(not(test), no_std)]

mod q;

pub type Q1_31 = q::Q<31>;
pub type Q16_16 = q::Q<16>;

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
