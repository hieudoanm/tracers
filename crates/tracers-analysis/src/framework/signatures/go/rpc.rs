//! Go rpc framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const GO_RPC_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "grpc",
        display_name: "gRPC",
        family: FrameworkFamily::Go,
        import_prefixes: &["google.golang.org/grpc"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["go.mod"],
        manifest_deps: &["google.golang.org/grpc"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "connectrpc",
        display_name: "Connect RPC",
        family: FrameworkFamily::Go,
        import_prefixes: &["connectrpc.com"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["go.mod"],
        manifest_deps: &["connectrpc.com"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "gqlgen",
        display_name: "gqlgen",
        family: FrameworkFamily::Go,
        import_prefixes: &[
            "github.com/99designs/gqlgen",
            "github.com/graph-gophers/graphql-go",
        ],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["go.mod"],
        manifest_deps: &[
            "github.com/99designs/gqlgen",
            "github.com/graph-gophers/graphql-go",
        ],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
