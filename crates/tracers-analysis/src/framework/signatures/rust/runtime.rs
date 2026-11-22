//! Rust runtime framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const RUST_RUNTIME_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "tokio",
        display_name: "Tokio",
        family: FrameworkFamily::Rust,
        import_prefixes: &["tokio"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["tokio"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "futures",
        display_name: "futures-rs",
        family: FrameworkFamily::Rust,
        import_prefixes: &["futures", "futures_util"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["futures", "futures-util"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
