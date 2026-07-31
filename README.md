<img align="right" width="200" src="https://raw.githubusercontent.com/pwnwrkz/rbx_install/refs/heads/main/assets/RBXInstall.png" alt="Logo" />

<h1><code>rbx_install</code></h1>

Rust crate to locate Roblox Studio installations.

It is technically a fork of [`roblox_install`](https://github.com/Kampfkarren/roblox-install), but supports [Vinegar](https://vinegarhq.org/Vinegar/Installation).

And it's also pretty optimized, with cache lookups for repeated calls.

Built for [Tungsten](https://pwnwrkz.github.io/tungsten).

## Installation

Run this command to add the crate:

```bash
cargo add rbx_install
```

Alternatively, just add a new entry in your `Cargo.toml` dependencies:

```toml
[dependencies]
rbx_install = "0.2.0"
```

## Example

```rs
use rbx_install::{self, RobloxStudio};

fn main() -> rbx_install::Result<()> {
    // Quick one-liner: just the executable path.
    // This is cached after the first call.
    let exe = rbx_install::locate()?;
    println!("Studio executable: {}", exe.display());

    // Full installation details.
    let studio = RobloxStudio::locate()?;
    println!("Application:  {}", studio.application_path().display());
    println!("Content:      {}", studio.content_path().display());
    println!("Plugins:      {}", studio.plugins_path().display());
    println!("Built-ins:    {}", studio.built_in_plugins_path().display());

    // Subsequent calls are O(1) — they hit the cache.
    let _ = RobloxStudio::locate()?;
    let _ = rbx_install::locate()?;

    Ok(())
}
```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](https://github.com/pwnwrkz/rbx_install/blob/main/LICENSE) file for details.
