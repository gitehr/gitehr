//! The calculator implementations.
//!
//! Each module exposes a strongly-typed `Input` struct, a pure `compute`
//! function, a `build_response` adapter to [`crate::CalculationResponse`], and a
//! unit struct implementing [`crate::Calculator`]. Register new calculators in
//! [`crate::all`].

pub mod amts;
pub mod asrs;
pub mod audit;
pub mod auditc;
pub mod cha2ds2vasc;
pub mod egfr;
pub mod epds;
pub mod feverpain;
pub mod fib4;
pub mod gad7;
pub mod ipss;
pub mod mrc_dyspnoea;
pub mod phq9;
