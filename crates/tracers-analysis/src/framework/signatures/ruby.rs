//! Ruby framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod auth;
mod data;
mod jobs;
mod testing;
mod tools;
mod ui;
mod web;

/// All Ruby signatures, in deterministic category order.
pub const RUBY_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::RUBY_WEB_SIGNATURES,
    jobs::RUBY_JOBS_SIGNATURES,
    auth::RUBY_AUTH_SIGNATURES,
    data::RUBY_DATA_SIGNATURES,
    testing::RUBY_TESTING_SIGNATURES,
    ui::RUBY_UI_SIGNATURES,
    tools::RUBY_TOOLS_SIGNATURES,
];
