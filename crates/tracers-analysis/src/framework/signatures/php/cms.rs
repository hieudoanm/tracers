//! PHP cms framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const PHP_CMS_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "drupal",
        display_name: "Drupal",
        family: FrameworkFamily::Php,
        import_prefixes: &["Drupal"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["drupal/core"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "magento",
        display_name: "Magento",
        family: FrameworkFamily::Php,
        import_prefixes: &["Magento"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["magento/product-community-edition"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "craftcms",
        display_name: "Craft CMS",
        family: FrameworkFamily::Php,
        import_prefixes: &["craft\\cms\\", "Craft\\Cms"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["craftcms/cms"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
