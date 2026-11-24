# How to install the tracers CLI on macOS (Intel)

For Intel-based Macs. The repository analysis CLI

## Option 1: install script (recommended)

```sh
curl -fsSL https://github.com/hieudoanm/tracers/master/scripts/install.sh | bash
```

The script detects `macos/x86_64` and downloads
`cli-tracers-x86_64-apple-darwin` from the
[latest release](https://github.com/hieudoanm/tracers/releases/latest). It
installs to `~/.local/bin/tracers` — override with `INSTALL_DIR=/path`.

## Option 2: manual download

1. Open the [downloads page](https://hieudoanm.github.io/tracers/downloads/) and
   download **macOS (Intel)** under **CLI** — `cli-tracers-x86_64-apple-darwin`.
2. Make it executable and move it onto your `PATH`:

```sh
chmod +x cli-tracers-x86_64-apple-darwin
mv cli-tracers-x86_64-apple-darwin ~/.local/bin/tracers
```

3. Verify:

```sh
tracers --version
```

## Troubleshooting

- `tracers: command not found` — the install directory is not on your `PATH`.
  Add `export PATH="$HOME/.local/bin:$PATH"` to your shell profile.
- Gatekeeper warning — the binary is unsigned. Right-click → Open, or run
  `xattr -d com.apple.quarantine` on the installed binary.
