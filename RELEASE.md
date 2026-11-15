## tracers — latest build

High-performance, multi-language code intelligence and graph analysis:
parses 15 languages with Tree-sitter, builds a unified dependency/call
graph, and answers structural queries (callers, callees, impact,
cycles, paths). Includes an embedded D3 graph explorer and an MCP
server.

### Download

| Platform | Asset |
| --- | --- |
| Linux x86_64 | `tracers-x86_64-unknown-linux-gnu` |
| macOS Apple Silicon | `tracers-aarch64-apple-darwin` |
| Windows x86_64 | `tracers-x86_64-pc-windows-msvc.exe` |

### Quick start

```sh
tracers scan .
tracers stats
tracers callers src/main.rs:main
tracers mcp
```

### Documentation

- [README](https://github.com/hieudoanm/tracers#readme)
