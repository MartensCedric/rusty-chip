use num::PrimInt;
use std::fmt::Display;

pub fn validate_argument<T: PrimInt + Display>(value : T, mask: T) -> T{
    if value.bitand(mask) != value {
        panic!("Argument {} is outside of mask {}!", value, mask);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn validate_argument_test() {
        validate_argument(0x54, 0xFF);
        validate_argument(0x132, 0xFFF);
        validate_argument(0x5, 0xFF);
        validate_argument(0x3, 0xF);
        validate_argument(0xFFF, 0xFFF);
    }

    #[test]
    #[should_panic]
    pub fn validate_argument_outside_test() {
        validate_argument(0x254, 0xFF);
    }

    #[test]
    #[should_panic]
    pub fn validate_argument_barely_bad_test() {
        validate_argument(0x54, 0x53);
    }
}