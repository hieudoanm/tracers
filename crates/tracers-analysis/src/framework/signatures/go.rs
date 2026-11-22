//! Go framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod data;
mod infra;
mod rpc;
mod testing;
mod web;

/// All Go signatures, in deterministic category order.
pub const GO_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::GO_WEB_SIGNATURES,
    rpc::GO_RPC_SIGNATURES,
    data::GO_DATA_SIGNATURES,
    infra::GO_INFRA_SIGNATURES,
    testing::GO_TESTING_SIGNATURES,
];
