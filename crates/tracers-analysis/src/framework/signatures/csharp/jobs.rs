//! C# jobs framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const CSHARP_JOBS_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "hangfire",
        display_name: "Hangfire",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Hangfire"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Hangfire"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "quartz",
        display_name: "Quartz.NET",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Quartz"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Quartz"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
