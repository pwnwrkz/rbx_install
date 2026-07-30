# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0]

### Added

- Initial release of `rbx_install`
- Cross-platform Roblox Studio location:
  - Windows: Registry-based detection via `HKCU\Software\Roblox\RobloxStudio`
  - macOS: Standard `/Applications/RobloxStudio.app` bundle
  - Linux: Vinegar Flatpak support (optional `vinegar` feature)
- Caching via `OnceLock` for O(1) subsequent lookups
- Environment variable override: `RBX_INSTALL_PATH`
- Public API: `RobloxStudio::locate()` and free `rbx_install::locate()`
- Path accessors: `application_path()`, `content_path()`, `built_in_plugins_path()`, `plugins_path()`
- Comprehensive error types for all failure modes
- CI workflow (Linux/Windows/macOS) with fmt, clippy, tests
- Release workflow publishing to crates.io

### Features

- `vinegar` (optional): Linux Vinegar support with `reqwest` + `serde_json`

[Unreleased]: https://github.com/pwnwrkz/rbx_install/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/pwnwrkz/rbx_install/releases/tag/v0.1.0
