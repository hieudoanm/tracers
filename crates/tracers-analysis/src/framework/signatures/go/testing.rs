//! Go testing framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const GO_TESTING_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "testify",
        display_name: "Testify",
        family: FrameworkFamily::Go,
        import_prefixes: &["github.com/stretchr/testify"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["go.mod"],
        manifest_deps: &["github.com/stretchr/testify"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "ginkgo",
        display_name: "Ginkgo",
        family: FrameworkFamily::Go,
        import_prefixes: &["github.com/onsi/ginkgo"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["go.mod"],
        manifest_deps: &["github.com/onsi/ginkgo"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "gomock",
        display_name: "GoMock",
        family: FrameworkFamily::Go,
        import_prefixes: &["github.com/golang/mock", "go.uber.org/mock"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["go.mod"],
        manifest_deps: &["github.com/golang/mock", "go.uber.org/mock"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
