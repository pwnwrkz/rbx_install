# rbx_install Optimization and Fix Handoff Report

This report consolidates all recently identified optimization opportunities and the WSL compatibility gap, so another agent can implement them in one pass.

## Scope

- Repository: `pwnwrkz/rbx_install`
- Primary file impacted: `/home/runner/work/rbx_install/rbx_install/src/lib.rs`

## Consolidated Findings

### 1) Add WSL fallback auto-detection (functional fix)

**Issue**
- Linux currently always resolves through Vinegar (`locate_vinegar()`), and does not auto-detect Windows Roblox Studio installations when running under WSL.
- This mirrors the issue addressed in `Kampfkarren/roblox-install` PR #36.

**Current behavior**
- If Vinegar is missing, Linux returns `VinegarInstallationNotFound`, even on WSL systems that have Roblox installed on Windows.
- WSL users only work today when manually setting `ROBLOX_STUDIO_PATH`.

**Desired behavior**
- On Linux, attempt Vinegar first.
- If Vinegar lookup fails and runtime is WSL, attempt Windows-style location fallback (under mounted Windows filesystem).
- Keep explicit env override behavior unchanged and highest priority.

---

### 2) Unify top-level executable lookup through `RobloxStudio::locate()` (optimization + simplification)

**Issue**
- `rbx_install::locate()` duplicates parts of detection and cache population instead of reusing `RobloxStudio::locate()`.

**Optimization**
- Route executable-only lookup through the full lookup path and return `application_path`.
- Reduces branching duplication and avoids repeated clone/set logic.

---

### 3) Cache negative platform detection outcomes (high-impact for repeated polling)

**Issue**
- Only successful platform detection is cached (`OnceLock<RobloxStudio>`).
- Repeated calls in not-installed states repeatedly perform I/O (registry/filesystem scans).

**Optimization**
- Cache failed platform lookup outcomes (e.g., `NotInstalled`, Vinegar missing) where safe.
- Prevents repeated expensive work in callers that poll for availability.

---

### 4) Memoize environment variable resolution (including “unset”)

**Issue**
- `locate_executable_from_env()` and `RobloxStudio::locate_from_env()` call `env::var_os` every invocation.

**Optimization**
- Memoize the env resolution result for process lifetime (including absent variable state) to avoid repeated lookups on hot paths.

---

### 5) Reduce `PathBuf` churn during directory iteration

**Issue**
- In version scanning flows (`locate_from_windows_directory`, `locate_vinegar`), `entry.path()` is recomputed multiple times.

**Optimization**
- Compute each entry path once and reuse it within the closure.
- Cuts per-entry temporary allocations in large `Versions` directories.

---

### 6) Cache stable derived plugin/home-based paths

**Issue**
- Plugin paths are rebuilt repeatedly via chained `PathBuf::push`/`join`.

**Optimization**
- Cache stable derived paths (e.g., plugin directory path) where they are invariant for process lifetime.

---

### 7) Use OS-string user env handling for filesystem path assembly

**Issue**
- Linux Vinegar path code uses UTF-8 `env::var("USER").unwrap_or_default()`.

**Optimization**
- Prefer `env::var_os("USER")` for path assembly to avoid unnecessary UTF-8 conversion/allocations and better match filesystem semantics.

## Recommended Implementation Order

1. WSL fallback auto-detection (functional compatibility fix)
2. Unify `locate()` path through `RobloxStudio::locate()`
3. Cache negative platform lookup outcomes
4. Reduce `PathBuf` churn in iteration closures
5. Cache derived plugin/home paths
6. Memoize env variable resolution
7. Switch USER handling to `var_os`

## Validation Targets for Implementing Agent

- Existing behavior preserved:
  - `ROBLOX_STUDIO_PATH` override remains top-priority.
  - Windows/macOS behavior unchanged.
  - Vinegar-first behavior on Linux remains.
- New behavior added:
  - WSL can locate Windows installation without requiring env override.
- Performance checks:
  - Repeated lookups should avoid repeated filesystem/registry scans in both success and expected failure states.

## Notes

- This report intentionally contains no code patch, only a consolidated implementation handoff.
