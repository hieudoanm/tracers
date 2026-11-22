//! C# data framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const CSHARP_DATA_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "efcore",
        display_name: "Entity Framework Core",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Microsoft.EntityFrameworkCore"],
        decorators: &[],
        base_classes: &["DbContext"],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Microsoft.EntityFrameworkCore"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "dapper",
        display_name: "Dapper",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Dapper"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Dapper"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
