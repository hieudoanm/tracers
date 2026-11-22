//! Kotlin di framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const KOTLIN_DI_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "hilt",
        display_name: "Hilt / Dagger",
        family: FrameworkFamily::Kotlin,
        import_prefixes: &["dagger", "com.google.dagger"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &[
            "build.gradle",
            "build.gradle.kts",
            "pom.xml",
            "libs.versions.toml",
        ],
        manifest_deps: &["com.google.dagger"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "koin",
        display_name: "Koin",
        family: FrameworkFamily::Kotlin,
        import_prefixes: &["org.koin"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &[
            "build.gradle",
            "build.gradle.kts",
            "pom.xml",
            "libs.versions.toml",
        ],
        manifest_deps: &["org.koin", "io.insert-koin"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
