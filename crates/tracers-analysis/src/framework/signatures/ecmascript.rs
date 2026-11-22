//! EcmaScript framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod backend;
mod client;
mod data;
mod platforms;
mod services;
mod tooling;
mod ui;
mod web;

/// All EcmaScript signatures, in deterministic category order.
pub const ECMASCRIPT_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::ECMASCRIPT_WEB_SIGNATURES,
    backend::ECMASCRIPT_BACKEND_SIGNATURES,
    data::ECMASCRIPT_DATA_SIGNATURES,
    ui::ECMASCRIPT_UI_SIGNATURES,
    platforms::ECMASCRIPT_PLATFORMS_SIGNATURES,
    services::ECMASCRIPT_SERVICES_SIGNATURES,
    client::ECMASCRIPT_CLIENT_SIGNATURES,
    tooling::ECMASCRIPT_TOOLING_SIGNATURES,
];
