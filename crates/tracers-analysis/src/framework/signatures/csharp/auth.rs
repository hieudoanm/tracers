//! C# auth signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const CSHARP_AUTH_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "duende",
        display_name: "Duende IdentityServer",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Duende"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Duende.IdentityServer"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "openiddict",
        display_name: "OpenIddict",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["OpenIddict"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["OpenIddict"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
