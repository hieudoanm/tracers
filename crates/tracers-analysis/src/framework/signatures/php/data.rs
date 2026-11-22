//! PHP data framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const PHP_DATA_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "doctrine",
        display_name: "Doctrine",
        family: FrameworkFamily::Php,
        import_prefixes: &["Doctrine\\ORM", "Doctrine\\DBAL", "Doctrine\\Common"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["doctrine/orm", "doctrine/dbal"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "predis",
        display_name: "Predis",
        family: FrameworkFamily::Php,
        import_prefixes: &["Predis"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["predis/predis"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "phpredis",
        display_name: "phpredis",
        family: FrameworkFamily::Php,
        import_prefixes: &[],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["ext-redis"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
