//! PHP ui framework signatures.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const PHP_UI_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "livewire",
        display_name: "Livewire",
        family: FrameworkFamily::Php,
        import_prefixes: &["Livewire"],
        decorators: &[],
        base_classes: &["Component"],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["livewire/livewire"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "inertia-laravel",
        display_name: "Inertia Laravel",
        family: FrameworkFamily::Php,
        import_prefixes: &["Inertia"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["inertiajs/inertia-laravel"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
