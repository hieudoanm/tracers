//! PHP data framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const PHP_DATA_SIGNATURES: &[FrameworkSignature] = &[FrameworkSignature {
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
}];
