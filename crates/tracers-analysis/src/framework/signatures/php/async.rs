//! PHP async signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const PHP_ASYNC_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "reactphp",
        display_name: "ReactPHP",
        family: FrameworkFamily::Php,
        import_prefixes: &["React"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["react/event-loop"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "amphp",
        display_name: "Amphp",
        family: FrameworkFamily::Php,
        import_prefixes: &["Amp"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["amphp/amp"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
