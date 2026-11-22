//! Rust framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod cli;
mod data;
mod frontend;
mod game;
mod rpc;
mod runtime;
mod util;
mod web;

/// All Rust signatures, in deterministic category order.
pub const RUST_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::RUST_WEB_SIGNATURES,
    frontend::RUST_FRONTEND_SIGNATURES,
    game::RUST_GAME_SIGNATURES,
    runtime::RUST_RUNTIME_SIGNATURES,
    rpc::RUST_RPC_SIGNATURES,
    data::RUST_DATA_SIGNATURES,
    cli::RUST_CLI_SIGNATURES,
    util::RUST_UTIL_SIGNATURES,
];
