use num::PrimInt;
use std::fmt::Display;

pub fn validate_argument<T: PrimInt + Display>(value : T, mask: T) -> T{
    if value.bitand(mask) != value {
        panic!("Argument {} is outside of mask {}!", value, mask);
    }
    value
}