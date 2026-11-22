//! Rust frontend framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const RUST_FRONTEND_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "leptos",
        display_name: "Leptos",
        family: FrameworkFamily::Rust,
        import_prefixes: &["leptos"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["leptos"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "dioxus",
        display_name: "Dioxus",
        family: FrameworkFamily::Rust,
        import_prefixes: &["dioxus"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["dioxus"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "yew",
        display_name: "Yew",
        family: FrameworkFamily::Rust,
        import_prefixes: &["yew"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["yew"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
