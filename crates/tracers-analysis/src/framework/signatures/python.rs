//! Python framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod ai;
mod data;
mod infra;
mod ml;
mod tools;
mod web;

/// All Python signatures, in deterministic category order.
pub const PYTHON_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::PYTHON_WEB_SIGNATURES,
    data::PYTHON_DATA_SIGNATURES,
    ml::PYTHON_ML_SIGNATURES,
    infra::PYTHON_INFRA_SIGNATURES,
    ai::PYTHON_AI_SIGNATURES,
    tools::PYTHON_TOOLS_SIGNATURES,
];
