//! PHP testing framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const PHP_TESTING_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "phpunit",
        display_name: "PHPUnit",
        family: FrameworkFamily::Php,
        import_prefixes: &["PHPUnit"],
        decorators: &[],
        base_classes: &["TestCase"],
        calls: &[],
        manifest_files: &["composer.json", "phpunit.xml"],
        manifest_deps: &["phpunit/phpunit"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "phpstan",
        display_name: "PHPStan",
        family: FrameworkFamily::Php,
        import_prefixes: &[],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["phpstan/phpstan"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
