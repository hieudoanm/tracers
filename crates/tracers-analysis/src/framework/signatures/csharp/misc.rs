//! C# misc signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const CSHARP_MISC_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "orleans",
        display_name: "Orleans",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Orleans"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Microsoft.Orleans"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "umbraco",
        display_name: "Umbraco CMS",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Umbraco"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Umbraco.Cms"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
