//! PHP framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod cms;
mod data;
mod libs;
mod testing;
mod ui;
mod web;

/// All PHP signatures, in deterministic category order.
pub const PHP_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::PHP_WEB_SIGNATURES,
    cms::PHP_CMS_SIGNATURES,
    data::PHP_DATA_SIGNATURES,
    testing::PHP_TESTING_SIGNATURES,
    ui::PHP_UI_SIGNATURES,
    libs::PHP_LIBS_SIGNATURES,
];
