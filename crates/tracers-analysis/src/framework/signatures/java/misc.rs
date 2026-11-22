//! Java misc framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const JAVA_MISC_SIGNATURES: &[FrameworkSignature] = &[FrameworkSignature {
    name: "immutables",
    display_name: "Immutables",
    family: FrameworkFamily::Java,
    import_prefixes: &["org.immutables"],
    decorators: &[],
    base_classes: &[],
    calls: &[],
    manifest_files: &["pom.xml", "build.gradle", "build.gradle.kts"],
    manifest_deps: &["org.immutables"],
    import_confidence: FrameworkConfidence::Medium,
    pattern_confidence: FrameworkConfidence::High,
    manifest_confidence: FrameworkConfidence::Medium,
}];
