//! C# framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod auth;
mod data;
mod infra;
mod jobs;
mod messaging;
mod misc;
mod testing;
mod web;

/// All C# signatures, in deterministic category order.
pub const CSHARP_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::CSHARP_WEB_SIGNATURES,
    data::CSHARP_DATA_SIGNATURES,
    messaging::CSHARP_MESSAGING_SIGNATURES,
    jobs::CSHARP_JOBS_SIGNATURES,
    testing::CSHARP_TESTING_SIGNATURES,
    infra::CSHARP_INFRA_SIGNATURES,
    auth::CSHARP_AUTH_SIGNATURES,
    misc::CSHARP_MISC_SIGNATURES,
];
