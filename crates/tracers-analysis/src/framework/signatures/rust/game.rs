//! Rust game framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const RUST_GAME_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "bevy",
        display_name: "Bevy",
        family: FrameworkFamily::Rust,
        import_prefixes: &["bevy"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["bevy"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "macroquad",
        display_name: "Macroquad",
        family: FrameworkFamily::Rust,
        import_prefixes: &["macroquad"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["macroquad"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
