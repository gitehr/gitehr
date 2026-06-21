//! The calculator implementations.
//!
//! Each module exposes a strongly-typed `Input` struct, a pure `compute`
//! function, a `build_response` adapter to [`crate::CalculationResponse`], and a
//! unit struct implementing [`crate::Calculator`]. Register new calculators in
//! [`crate::all`].

pub mod abcd2;
pub mod amts;
pub mod asrs;
pub mod audit;
pub mod auditc;
pub mod cha2ds2vasc;
pub mod curb65;
pub mod egfr;
pub mod epds;
pub mod feverpain;
pub mod fib4;
pub mod fourat;
pub mod gad7;
pub mod hasbled;
pub mod ipss;
pub mod mrc_dyspnoea;
pub mod news2;
pub mod phq9;
pub mod qsofa;
pub mod wells_dvt;
pub mod wells_pe;
