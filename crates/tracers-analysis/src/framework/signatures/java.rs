//! Java framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod data;
mod infra;
mod misc;
mod net;
mod testing;
mod web;

/// All Java signatures, in deterministic category order.
pub const JAVA_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::JAVA_WEB_SIGNATURES,
    data::JAVA_DATA_SIGNATURES,
    testing::JAVA_TESTING_SIGNATURES,
    infra::JAVA_INFRA_SIGNATURES,
    net::JAVA_NET_SIGNATURES,
    misc::JAVA_MISC_SIGNATURES,
];
