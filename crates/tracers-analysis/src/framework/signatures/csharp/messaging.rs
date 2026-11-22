//! C# messaging framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const CSHARP_MESSAGING_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "grpc-dotnet",
        display_name: "gRPC for .NET",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Grpc"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Grpc"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "masstransit",
        display_name: "MassTransit",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["MassTransit"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["MassTransit"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "rabbitmq-dotnet",
        display_name: "RabbitMQ.Client",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["RabbitMQ"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["RabbitMQ.Client"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
