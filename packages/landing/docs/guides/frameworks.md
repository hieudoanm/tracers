# Framework & library detection

Tracers recognises frameworks, libraries, and packages — not just languages.
Each signature in the database describes how to spot one framework from
**imports**, **decorators/annotations**, **base classes**, **calls**, and
**package manifests** (package.json, go.mod, Cargo.toml, pyproject.toml,
build.gradle, pom.xml, composer.json, Gemfile, *.csproj, ...).

The database currently holds **584 entries** spanning nine language families.
When a scan finds a match it emits a `framework` node and a `uses_framework`
edge with a confidence level:

- **High** — import/decorator/base-class/call evidence.
- **Medium** — weaker pattern evidence.
- **Low** — manifest-only dependency match.

Non-manifest signals are scoped to the file's language family (with
cross-matching inside the JVM families, so Kotlin files detect Java frameworks
and vice versa), and manifest dependency matching is boundary-aware to avoid
substring false positives.

## Supported frameworks by language

| Language                | Signatures | Page                                                         |
| ----------------------- | ---------: | ------------------------------------------------------------ |
| TypeScript / JavaScript |         64 | [typescript-javascript](frameworks/typescript-javascript.md) |
| Python                  |         64 | [python](frameworks/python.md)                               |
| Go                      |         66 | [go](frameworks/go.md)                                       |
| Rust                    |         64 | [rust](frameworks/rust.md)                                   |
| Kotlin                  |         64 | [kotlin](frameworks/kotlin.md)                               |
| Java                    |         64 | [java](frameworks/java.md)                                   |
| C#                      |         66 | [csharp](frameworks/csharp.md)                               |
| PHP                     |         64 | [php](frameworks/php.md)                                     |
| Ruby                    |         68 | [ruby](frameworks/ruby.md)                                   |

Each per-language page lists every signature with its `name`, display name,
matched import prefixes, decorators, base classes, calls, and manifest
dependencies.

---

_Generated from the signature database — regenerate with
`cargo run -p tracers-analysis --example list_frameworks`._
