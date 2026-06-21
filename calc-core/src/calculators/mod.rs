//! The calculator implementations.
//!
//! Each module exposes a strongly-typed `Input` struct, a pure `compute`
//! function, a `build_response` adapter to [`crate::CalculationResponse`], and a
//! unit struct implementing [`crate::Calculator`]. Register new calculators in
//! [`crate::all`].

pub mod abcd2;
pub mod abpi;
pub mod amts;
pub mod asrs;
pub mod audit;
pub mod auditc;
pub mod bode;
pub mod cha2ds2vasc;
pub mod chalice;
pub mod child_pugh;
pub mod ckd_risk;
pub mod curb65;
pub mod das28;
pub mod egfr;
pub mod epds;
pub mod euroscore2;
pub mod feverpain;
pub mod fib4;
pub mod fourat;
pub mod gleason;
pub mod grace;
pub mod gad7;
pub mod hasbled;
pub mod heart;
pub mod ipss;
pub mod meld;
pub mod mrc_dyspnoea;
pub mod news2;
pub mod nhfs;
pub mod npi;
pub mod padua;
pub mod phq9;
pub mod qsofa;
pub mod sofa;
pub mod timi;
pub mod uacr;
pub mod ukeld;
pub mod waterlow;
pub mod wells_dvt;
pub mod wells_pe;
