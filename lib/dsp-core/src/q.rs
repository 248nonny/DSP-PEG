use const_format::formatcp;
#[cfg(test)]
use core::fmt::{Display, Formatter};
use core::ops::{Add, Mul, Neg, Sub};

macro_rules! q_impl {
    ($trait:ident, { $($body:tt)* }) => {
        impl<const F: u32> $trait for Q<F> {
            $($body)*
        }
    };
}

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Q<const FRAC_BITS: u32> {
    value: i32,
}

impl<const F: u32> Q<F> {
    const _CHECK_BOUNDS: () = {
        if F > 31 {
            panic!("Q format error: F cannot be greater than 31 bits!");
        } else if F == 0 {
            panic!("Q format error: F should not be 0, use i32 instead.");
        }
    };

    // Didn't use From trait since technically we are
    // changing the meaning of the number, i.e. when we
    // feed "1" into this, the output does not represent 1.
    #[inline(always)]
    pub fn new(raw: i32) -> Self {
        #[allow(clippy::let_unit_value)]
        let _ = Self::_CHECK_BOUNDS;

        Self { value: raw }
    }

    pub fn multiply_by<const F_OTHER: u32>(self, other: Q<F_OTHER>) -> Self {
        // Assumption is that we are using a 64 bit processor,
        // so this shouldn't be too inefficient.
        let x = self.value as i64;
        let y = other.value as i64;

        let mut product = x * y;

        // Add 0.5 for rounding purposes.
        if F_OTHER > 0 {
            product = (product).saturating_add(1 << (F_OTHER - 1));
        }

        // Clamp to allowed values.
        // If we need better performance later, maybe
        // we can hand optimize using NEON ARM assembly,
        // and have clamped and unclamped options.
        let value = (product >> F_OTHER).clamp(i32::MIN as i64, i32::MAX as i64) as i32;

        Self { value }
    }
}

impl<const F: u32> TryFrom<f32> for Q<F> {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        let scale_value = (1_i64 << F) as f32; // Scale input value to fit in

        let scaled = value * scale_value;

        if scaled < (i32::MIN as f32) || scaled > (i32::MAX as f32) || scaled.is_nan() {
            return Err(formatcp!("Value out of range for Q format."));
        }

        Ok(Self::new(scaled as i32))
    }
}

// How to implement something like this for references at the same time?
q_impl!(Add, {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value.saturating_add(rhs.value),
        }
    }
});

q_impl!(Neg, {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self { value: -self.value }
    }
});

q_impl!(Sub, {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value.saturating_sub(rhs.value),
        }
    }
});

q_impl!(Mul, {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply_by(rhs)
    }
});

#[cfg(test)]
q_impl!(Display, {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} ≈ {:.7})",
            self.value,
            self.value as f64 / (1 << 30_u32) as f64
        )
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_31() {
        let a: Q<31> = Q::new(10);
        let b: Q<31> = Q::new(100);

        std::println!("{}", a + b);
        std::assert!(a + b == Q::new(110));
    }

    #[test]
    fn test_mul_31() {
        let a: Q<31> = Q::new(1 << 30);
        let b: Q<31> = Q::new((1 << 29) + 8);

        std::assert_eq!(a * a, Q::new(1 << 29));

        std::assert_eq!(a * b, Q::new((1 << 28) + 4));

        std::assert_eq!(b * b, Q::new((1 << 27) + 2 + 2));
    }
}
