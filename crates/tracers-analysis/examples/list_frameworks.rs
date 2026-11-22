//! Prints the framework signature database as a Markdown table grouped by
//! language family. Used to generate `packages/landing/docs/guides/frameworks.md`.
//!
//! Run: `cargo run -p tracers-analysis --example list_frameworks`

use tracers_analysis::framework::signatures::all_signatures;
use tracers_analysis::framework::types::FrameworkFamily;

const FAMILIES: [FrameworkFamily; 9] = [
    FrameworkFamily::EcmaScript,
    FrameworkFamily::Python,
    FrameworkFamily::Go,
    FrameworkFamily::Rust,
    FrameworkFamily::Kotlin,
    FrameworkFamily::Java,
    FrameworkFamily::CSharp,
    FrameworkFamily::Php,
    FrameworkFamily::Ruby,
];

const FAMILY_TITLES: [(&str, &str); 9] = [
    ("ecmascript", "TypeScript / JavaScript"),
    ("python", "Python"),
    ("go", "Go"),
    ("rust", "Rust"),
    ("kotlin", "Kotlin"),
    ("java", "Java"),
    ("csharp", "C#"),
    ("php", "PHP"),
    ("ruby", "Ruby"),
];

fn main() {
    let signatures = all_signatures();
    println!("Total signatures: {}", signatures.len());
    println!();

    for (family_key, title) in FAMILY_TITLES {
        let family = FAMILIES
            .iter()
            .copied()
            .find(|f| f.as_str() == family_key)
            .unwrap();
        let entries: Vec<_> = signatures.iter().filter(|s| s.family == family).collect();
        println!("## {title} ({})", entries.len());
        println!();
        println!("| Name | Display name |");
        println!("|------|--------------|");
        for sig in entries {
            println!("| `{}` | {} |", sig.name, sig.display_name);
        }
        println!();
    }
}
