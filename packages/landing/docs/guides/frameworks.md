# Supported frameworks & libraries

Tracers recognises frameworks, libraries, and major packages through a signature
database of **303 entries** spanning nine language families. When a scan finds
evidence of one, it adds a `framework` node to the graph and `uses_framework` /
`extends_framework` edges from the matching symbols.

## How detection works

Each signature declares the evidence it looks for:

- **Import prefixes** — e.g. `import { useState } from 'react'`,
  `use Illuminate\Support\Facades\Route`, or
  `import "github.com/gin-gonic/gin"`.
- **Decorators / annotations** — e.g. `@Component`, `@app.get`,
  `@RestController`.
- **Base classes** — e.g. `class Foo extends Express`.
- **Call names** — e.g. `describe(...)` for test runners (only qualified or
  distinctive calls are used to avoid false positives).
- **Manifest files** — `package.json`, `requirements.txt`, `pyproject.toml`,
  `Cargo.toml`, `go.mod`, `pom.xml`, `build.gradle`, `*.csproj`,
  `composer.json`, `Gemfile`, etc. Dependency names in manifests are matched
  with boundary checks, so `aws-sdk-s3` does not match `aws-sdk` accidentally.

Every match records a **confidence level** (`high`, `medium`, `low`) and a
**signal kind**, so the graph always shows _why_ a framework was detected —
import evidence is stronger than a manifest-only hit. Multiple signals for the
same framework are merged into one node.

Non-manifest signals are scoped to the signature's language family, so a Rust
`test()` call will not be mistaken for Jest. Java-family signatures also apply
to Kotlin files, since Kotlin/JVM projects share `org.springframework` and
`io.ktor` import conventions.

## By language

| Language family         | Signatures | Page                                                         |
| ----------------------- | ---------: | ------------------------------------------------------------ |
| TypeScript / JavaScript |         64 | [typescript-javascript](frameworks/typescript-javascript.md) |
| Python                  |         64 | [python](frameworks/python.md)                               |
| Go                      |         40 | [go](frameworks/go.md)                                       |
| Rust                    |         42 | [rust](frameworks/rust.md)                                   |
| Java                    |         14 | [java](frameworks/java.md)                                   |
| C#                      |         15 | [csharp](frameworks/csharp.md)                               |
| PHP                     |         13 | [php](frameworks/php.md)                                     |
| Ruby                    |         16 | [ruby](frameworks/ruby.md)                                   |
| Kotlin                  |         35 | [kotlin](frameworks/kotlin.md)                               |

## Adding a framework

Signatures live in `crates/tracers-analysis/src/framework/signatures/` — one
module per language family, split into category files (e.g. `ecmascript/web.rs`,
`python/ml.rs`). To add an entry, provide at least one of:

- `import_prefixes` — distinctive module paths (use trailing `/` for scoped
  packages like `@nestjs/`).
- `manifest_deps` — package names as they appear in manifests.
- `decorators`, `base_classes`, `calls` — distinctive pattern names only; avoid
  generic names that would match unrelated code.

These pages are generated from the signature database — regenerate them with:

```bash
cargo run -p tracers-analysis --example list_frameworks
```

and update the per-language pages under `docs/guides/frameworks/`.
