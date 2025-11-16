fn testing_tests() -> usize {
    10
}

use core::fmt::{Display, Formatter};
use core::ops::{Add, Mul};

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone)]
struct Q2_30 {
    value: i32,
}

impl TryFrom<f32> for Q2_30 {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        let scale_value = (1 << 30_u32) as f32; // Scale input value to fit in

        let scaled = (value * scale_value) as i32;

        if scaled.abs() <= (1 << 30_i32) {
            return Ok(Q2_30 { value: scaled });
        } else {
            return Err("Bad value when converting from float.");
        }
    }
}

// How to implement something like this for references at the same time?
impl Add for Q2_30 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

// impl Mul for Q2_30 {
//     type Output = Self;
// }

#[cfg(test)]
impl Display for Q2_30 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} ≈ {:.7})",
            self.value,
            self.value as f64 / (1 << 30_u32) as f64
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let a: Q2_30 = (-0.1_f32).try_into().unwrap();
        let b = 0.1_f32.try_into().unwrap();

        std::println!("{}", a + b);
        panic!("{}", a);
    }

    #[test]
    fn test1() {
        // panic!();
        assert_eq!(testing_tests(), 10);
    }
}
