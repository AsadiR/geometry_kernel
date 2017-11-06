// use primitives::number_trait;
// use primitives::number_impl_big_rational;
use primitives::number_impl_gmp;


pub use primitives::number_trait::NumberTrait;

/// An alias for a number type, used in the geomety kernel.
//pub type Number = number_impl_big_rational::Number;
pub type Number = number_impl_gmp::Number;