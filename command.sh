
//Add these settings to your VSCode settings.json or .vscode/settings.json in your project:
{
  "rust-analyzer.server.extraEnv": {
    "RUSTUP_TOOLCHAIN": "stable"
  },
  "rust-analyzer.cargo.target": "riscv32imac-unknown-none-elf",
  "rust-analyzer.checkOnSave": false,
  "rust-analyzer.check.allTargets": false
}

rustup +stable component add rust-src
rustup +stable component add rust-analyzer


