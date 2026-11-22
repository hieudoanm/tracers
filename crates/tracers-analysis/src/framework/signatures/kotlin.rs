//! Kotlin framework signatures, grouped by category.

use crate::framework::types::FrameworkSignature;

mod android;
mod coroutines;
mod data;
mod di;
mod misc;
mod net;
mod testing;
mod web;

/// All Kotlin signatures, in deterministic category order.
pub const KOTLIN_SIGNATURES: &[&[FrameworkSignature]] = &[
    web::KOTLIN_WEB_SIGNATURES,
    android::KOTLIN_ANDROID_SIGNATURES,
    net::KOTLIN_NET_SIGNATURES,
    data::KOTLIN_DATA_SIGNATURES,
    coroutines::KOTLIN_COROUTINES_SIGNATURES,
    testing::KOTLIN_TESTING_SIGNATURES,
    di::KOTLIN_DI_SIGNATURES,
    misc::KOTLIN_MISC_SIGNATURES,
];
