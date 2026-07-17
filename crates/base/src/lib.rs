#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::return_self_not_must_use,
    clippy::wildcard_enum_match_arm,
    clippy::inline_always,
    clippy::struct_excessive_bools
)]

pub mod audio;
pub mod error;
pub mod frame;
