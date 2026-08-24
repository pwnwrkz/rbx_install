# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 24-08-2026

### Changed

- Failed platform detections are now cached alongside successes, so repeated calls never redo the filesystem/registry scan. A process restart is required to re-detect.
- `Error` now implements `Clone` (the `RegistryError` source is reconstructed from kind and message).
- The free `locate()` checks the detection cache before consulting `ROBLOX_STUDIO_PATH`; the environment variable must be set before the first call.
- Plugin-directory lookup is skipped when no Roblox Studio installation is found on Windows-style layouts.
- Reduced redundant path allocations in version-directory scanning (Windows fallback and Vinegar).

### Added

- Unit tests covering the Windows-style directory layout detection (`content` directory and `Versions` scan).

## [0.2.0] - 31-07-2026

### Changed

- **Vinegar support is now unconditional.** The optional `vinegar` feature has been removed. Linux now always attempts to detect a Vinegar (Flatpak) installation.
- Vinegar detection no longer queries the Roblox version endpoint. It now scans the local `versions/` directory directly, eliminating the `reqwest` and `serde_json` dependencies entirely.
- `RobloxStudio` no longer contains a `root` field. The deprecated `root_path()` and `exe_path()` methods have been removed. Use `application_path()`, `content_path()`, and the other dedicated accessors instead.

### Removed

- `vinegar` feature flag and its associated dependencies (`reqwest`, `serde_json`)
- `VersionEndpointError` and `VersionNotFoundInEndpoint` error variants
- Deprecated `RobloxStudio::root_path()` method
- Deprecated `RobloxStudio::exe_path()` method

### Added

- `VinegarInstallationNotFound` error variant for when the Vinegar versions directory exists but contains no valid Roblox Studio executable

## [0.1.0] - 30-07-2026

### Added

- Initial release of `rbx_install`
- Cross-platform Roblox Studio location:
  - Windows: Registry-based detection via `HKCU\Software\Roblox\RobloxStudio`
  - macOS: Standard `/Applications/RobloxStudio.app` bundle
  - Linux: Vinegar Flatpak support (optional `vinegar` feature)
- Caching via `OnceLock` for O(1) subsequent lookups
- Environment variable override: `ROBLOX_STUDIO_PATH`
- Public API: `RobloxStudio::locate()` and free `rbx_install::locate()`
- Path accessors: `application_path()`, `content_path()`, `built_in_plugins_path()`, `plugins_path()`
- Comprehensive error types for all failure modes
- CI workflow (Linux/Windows/macOS) with fmt, clippy, tests
- Release workflow publishing to crates.io

### Features

- `vinegar` (optional): Linux Vinegar support with `reqwest` + `serde_json`

[Unreleased]: https://github.com/pwnwrkz/rbx_install/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/pwnwrkz/rbx_install/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/pwnwrkz/rbx_install/releases/tag/v0.1.0
