//! Java testing framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const JAVA_TESTING_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "junit",
        display_name: "JUnit",
        family: FrameworkFamily::Java,
        import_prefixes: &["org.junit", "junit"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["pom.xml", "build.gradle", "build.gradle.kts"],
        manifest_deps: &["junit", "junit-jupiter"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "mockito",
        display_name: "Mockito",
        family: FrameworkFamily::Java,
        import_prefixes: &["org.mockito"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["pom.xml", "build.gradle", "build.gradle.kts"],
        manifest_deps: &["org.mockito", "mockito"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "assertj",
        display_name: "AssertJ",
        family: FrameworkFamily::Java,
        import_prefixes: &["org.assertj"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["pom.xml", "build.gradle", "build.gradle.kts"],
        manifest_deps: &["org.assertj"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
