# How to install the tracers CLI on Windows

For 64-bit Windows. The repository analysis CLI

The release binary is a native `.exe`. The install script is bash, so on Windows
run it from **Git Bash** or **WSL** — or download the binary manually.

## Option 1: install script (recommended)

```sh
curl -fsSL https://github.com/hieudoanm/tracers/master/scripts/install.sh | bash
```

The script detects Windows and downloads
`cli-tracers-x86_64-pc-windows-msvc.exe`, installing it to
`~/.local/bin/tracers` (or the path you pass as `INSTALL_DIR`).

## Option 2: manual download

1. Open the [downloads page](https://hieudoanm.github.io/tracers/downloads/) and
   download **Windows** under **CLI** —
   `cli-tracers-x86_64-pc-windows-msvc.exe`.
2. Rename it to `tracers.exe` and place it in a directory on your `PATH` (for
   example `%LOCALAPPDATA%\Programs\tracers\` or `C:\tools\`).
3. Open a new terminal and verify:

```powershell
tracers --version
```

## Troubleshooting

- `tracers is not recognized` — the directory is not on `PATH`. Add it via
  Settings → Environment Variables, or call the binary by full path.
- SmartScreen warning — the binary is unsigned. Choose **More info → Run
  anyway**.
