//! Rust rpc framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const RUST_RPC_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "tonic",
        display_name: "Tonic",
        family: FrameworkFamily::Rust,
        import_prefixes: &["tonic"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["tonic"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "prost",
        display_name: "prost",
        family: FrameworkFamily::Rust,
        import_prefixes: &["prost"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["prost"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
