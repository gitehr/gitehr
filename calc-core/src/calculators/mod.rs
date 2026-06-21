//! The calculator implementations.
//!
//! Each module exposes a strongly-typed `Input` struct, a pure `compute`
//! function, a `build_response` adapter to [`crate::CalculationResponse`], and a
//! unit struct implementing [`crate::Calculator`]. Register new calculators in
//! [`crate::all`].

pub mod asrs;
pub mod feverpain;
pub mod gad7;
pub mod phq9;
