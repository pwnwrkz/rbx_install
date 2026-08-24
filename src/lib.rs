use std::{
    env, fs, io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use thiserror::Error;

#[cfg(target_os = "windows")]
use winreg::RegKey;

/// A wrapper for [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html) that
/// contains [`Error`] in the `Err` type.
pub type Result<T> = std::result::Result<T, Error>;

const ROBLOX_STUDIO_PATH_VARIABLE: &str = "ROBLOX_STUDIO_PATH";

/// Cache for the expensive platform-specific lookup, including failures so
/// that repeated calls never redo the filesystem scan.
static PLATFORM_STUDIO: OnceLock<Result<RobloxStudio>> = OnceLock::new();

#[derive(Debug, Error)]
#[non_exhaustive]
/// Everything that can go wrong while using roblox-install.
pub enum Error {
    #[error("Couldn't find Documents directory")]
    DocumentsDirectoryNotFound,

    #[error(
        "The values of the registry keys used to find Roblox are malformed, maybe your Roblox installation is corrupt?"
    )]
    MalformedRegistry,

    #[error("Your platform is not currently supported")]
    PlatformNotSupported,

    #[error("Couldn't find Plugins directory")]
    PluginsDirectoryNotFound,

    #[error("Couldn't find registry keys, Roblox might not be installed.")]
    RegistryError(#[source] io::Error),

    #[error("Environment variable misconfigured: {0}")]
    EnvironmentVariableError(String),

    #[error("Couldn't find Roblox Studio")]
    NotInstalled,

    #[error("Couldn't find home directory")]
    HomeDirectoryNotFound,

    #[error("Vinegar installation not found")]
    VinegarInstallationNotFound,
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Error::RegistryError(e) => {
                Error::RegistryError(io::Error::new(e.kind(), e.to_string()))
            }
            Error::DocumentsDirectoryNotFound => Error::DocumentsDirectoryNotFound,
            Error::MalformedRegistry => Error::MalformedRegistry,
            Error::PlatformNotSupported => Error::PlatformNotSupported,
            Error::PluginsDirectoryNotFound => Error::PluginsDirectoryNotFound,
            Error::EnvironmentVariableError(message) => {
                Error::EnvironmentVariableError(message.clone())
            }
            Error::NotInstalled => Error::NotInstalled,
            Error::HomeDirectoryNotFound => Error::HomeDirectoryNotFound,
            Error::VinegarInstallationNotFound => Error::VinegarInstallationNotFound,
        }
    }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct RobloxStudio {
    content: PathBuf,
    application: PathBuf,
    built_in_plugins: PathBuf,
    plugins: PathBuf,
}

/// Attempts to find the Roblox Studio executable.
///
/// This is a convenience wrapper that preserves the semantics of the standalone
/// `locate` function from earlier versions of this crate:
/// * The platform-specific executable is located and cached after the first
///   call (including failures), making subsequent calls O(1).
/// * If `ROBLOX_STUDIO_PATH` is set and the cache is cold, it must point
///   directly to the executable; this value is never cached.
pub fn locate() -> Result<PathBuf> {
    if let Some(cached) = PLATFORM_STUDIO.get() {
        return cached.clone().map(|studio| studio.application);
    }

    if let Some(result) = locate_executable_from_env() {
        return result;
    }

    RobloxStudio::platform_locate().map(|studio| studio.application)
}

fn locate_executable_from_env() -> Option<Result<PathBuf>> {
    let variable_value = env::var_os(ROBLOX_STUDIO_PATH_VARIABLE)?;
    let path = PathBuf::from(variable_value);

    if path.is_file() {
        Some(Ok(path))
    } else {
        Some(Err(Error::EnvironmentVariableError(format!(
            "environment variable `{ROBLOX_STUDIO_PATH_VARIABLE}` is not a file: {}",
            path.display()
        ))))
    }
}

impl RobloxStudio {
    /// Attempts to find a Roblox Studio installation. It will start by looking up
    /// into the environment variable `ROBLOX_STUDIO_PATH`. If the variable is not
    /// defined, it will find the usual installation on Windows, MacOS, and Linux
    /// (via Vinegar).
    ///
    /// On Windows (or WSL), the environment variable can point to a specific version (where
    /// the `RobloxStudioBeta.exe` file and `content` directory are located) or it
    /// can also point to the Roblox directory in AppData (`$APPDATA\Local\Roblox`)
    /// and it will find the latest version by itself.
    ///
    /// The result of platform-specific detection (success or failure) is cached
    /// after the first call, making subsequent calls O(1). Environment overrides
    /// are never cached.
    pub fn locate() -> Result<RobloxStudio> {
        if let Some(cached) = PLATFORM_STUDIO.get() {
            return cached.clone();
        }

        if let Some(result) = Self::locate_from_env() {
            return result;
        }

        Self::platform_locate()
    }

    fn platform_locate() -> Result<RobloxStudio> {
        let result = Self::locate_target_specific();
        let _ = PLATFORM_STUDIO.set(result.clone());
        result
    }

    #[cfg(target_os = "windows")]
    fn locate_target_specific() -> Result<RobloxStudio> {
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

        let roblox_studio_reg = hkcu
            .open_subkey(r"Software\Roblox\RobloxStudio")
            .map_err(Error::RegistryError)?;

        let content_folder_value: String = roblox_studio_reg
            .get_value("ContentFolder")
            .map_err(Error::RegistryError)?;

        let content = PathBuf::from(content_folder_value);
        let root = content
            .parent()
            .ok_or(Error::MalformedRegistry)?
            .to_path_buf();

        let plugins = Self::locate_plugins_on_windows()?;

        Ok(RobloxStudio {
            content,
            application: root.join("RobloxStudioBeta.exe"),
            built_in_plugins: root.join("BuiltInPlugins"),
            plugins,
        })
    }

    #[cfg(target_os = "macos")]
    fn locate_target_specific() -> Result<RobloxStudio> {
        let root = PathBuf::from("/Applications/RobloxStudio.app");
        Self::locate_from_directory(root)
    }

    #[cfg(target_os = "linux")]
    fn locate_target_specific() -> Result<RobloxStudio> {
        Self::locate_vinegar()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    #[inline]
    fn locate_target_specific() -> Result<RobloxStudio> {
        Err(Error::PlatformNotSupported)
    }

    #[cfg(not(target_os = "macos"))]
    fn locate_plugins_on_windows() -> Result<PathBuf> {
        let mut plugin_dir = dirs::home_dir().ok_or(Error::PluginsDirectoryNotFound)?;
        plugin_dir.push("AppData");
        plugin_dir.push("Local");
        plugin_dir.push("Roblox");
        plugin_dir.push("Plugins");
        Ok(plugin_dir)
    }

    #[cfg(target_os = "windows")]
    fn locate_from_directory(root: PathBuf) -> Result<RobloxStudio> {
        Self::locate_from_windows_directory(root)
    }

    #[cfg(target_os = "macos")]
    fn locate_from_directory(root: PathBuf) -> Result<RobloxStudio> {
        let contents = root.join("Contents");
        let application = contents.join("MacOS").join("RobloxStudio");
        let built_in_plugins = contents.join("Resources").join("BuiltInPlugins");
        let documents = dirs::document_dir().ok_or(Error::DocumentsDirectoryNotFound)?;
        let plugins = documents.join("Roblox").join("Plugins");
        let content = contents.join("Resources").join("content");

        Ok(RobloxStudio {
            content,
            application,
            built_in_plugins,
            plugins,
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    fn locate_from_directory(root: PathBuf) -> Result<RobloxStudio> {
        Self::locate_from_windows_directory(root).map_err(|_| Error::PlatformNotSupported)
    }

    #[cfg(not(target_os = "macos"))]
    fn locate_from_windows_directory(root: PathBuf) -> Result<RobloxStudio> {
        let content_folder_path = root.join("content");

        if content_folder_path.is_dir() {
            let plugins = Self::locate_plugins_on_windows()?;
            return Ok(RobloxStudio {
                content: content_folder_path,
                application: root.join("RobloxStudioBeta.exe"),
                built_in_plugins: root.join("BuiltInPlugins"),
                plugins,
            });
        }

        let versions = root.join("Versions");
        if !versions.is_dir() {
            return Err(Error::NotInstalled);
        }

        let version_dir = fs::read_dir(&versions)
            .map_err(|_| Error::NotInstalled)?
            .filter_map(|entry| entry.ok())
            .find_map(|entry| {
                let path = entry.path();
                path.join("RobloxStudioBeta.exe").is_file().then_some(path)
            })
            .ok_or(Error::NotInstalled)?;

        let plugins = Self::locate_plugins_on_windows()?;

        Ok(RobloxStudio {
            content: version_dir.join("content"),
            application: version_dir.join("RobloxStudioBeta.exe"),
            built_in_plugins: version_dir.join("BuiltInPlugins"),
            plugins,
        })
    }

    #[cfg(target_os = "linux")]
    fn locate_vinegar() -> Result<RobloxStudio> {
        let home = dirs::home_dir().ok_or(Error::HomeDirectoryNotFound)?;

        let versions_dir = home.join(".var/app/org.vinegarhq.Vinegar/data/vinegar/versions");
        if !versions_dir.is_dir() {
            return Err(Error::VinegarInstallationNotFound);
        }

        let root = fs::read_dir(&versions_dir)
            .map_err(|_| Error::VinegarInstallationNotFound)?
            .filter_map(|e| e.ok())
            .find_map(|e| {
                let path = e.path();
                path.join("RobloxStudioBeta.exe").is_file().then_some(path)
            })
            .ok_or(Error::VinegarInstallationNotFound)?;

        let user = env::var("USER").unwrap_or_default();
        let plugins = home
            .join(".var/app/org.vinegarhq.Vinegar/data/vinegar/prefix")
            .join("drive_c/users")
            .join(&user)
            .join("AppData/Local/Roblox/Plugins");

        Ok(RobloxStudio {
            content: root.join("content"),
            application: root.join("RobloxStudioBeta.exe"),
            built_in_plugins: root.join("BuiltInPlugins"),
            plugins,
        })
    }

    #[must_use]
    #[inline]
    /// Path to the Roblox Studio executable
    pub fn application_path(&self) -> &Path {
        &self.application
    }

    #[must_use]
    #[inline]
    /// Path to the content directory
    pub fn content_path(&self) -> &Path {
        &self.content
    }

    #[must_use]
    #[inline]
    /// Path to built-in plugins directory
    pub fn built_in_plugins_path(&self) -> &Path {
        &self.built_in_plugins
    }

    #[must_use]
    #[inline]
    /// Path to the user's plugin directory. This directory may NOT exist if the Roblox Studio
    /// user has never opened it from Roblox Studio `Plugins Folder` button.
    pub fn plugins_path(&self) -> &Path {
        &self.plugins
    }

    fn locate_from_env() -> Option<Result<RobloxStudio>> {
        let variable_value = env::var_os(ROBLOX_STUDIO_PATH_VARIABLE)?;
        let path = PathBuf::from(variable_value);
        Some(Self::locate_from_directory(path))
    }
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn content_directory_layout_is_detected() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("content")).unwrap();

        let studio =
            RobloxStudio::locate_from_windows_directory(root.path().to_path_buf()).unwrap();

        assert_eq!(studio.content_path(), &root.path().join("content"));
        assert_eq!(
            studio.application_path(),
            &root.path().join("RobloxStudioBeta.exe")
        );
        assert_eq!(
            studio.built_in_plugins_path(),
            &root.path().join("BuiltInPlugins")
        );
    }

    #[test]
    fn versions_layout_finds_executable_and_skips_decoys() {
        let root = TempDir::new().unwrap();
        let decoy = root.path().join("Versions").join("version-0");
        let version = root.path().join("Versions").join("version-1");
        fs::create_dir_all(&decoy).unwrap();
        fs::create_dir_all(version.join("content")).unwrap();
        fs::write(version.join("RobloxStudioBeta.exe"), b"stub").unwrap();

        let studio =
            RobloxStudio::locate_from_windows_directory(root.path().to_path_buf()).unwrap();

        assert_eq!(
            studio.application_path(),
            &version.join("RobloxStudioBeta.exe")
        );
        assert_eq!(studio.content_path(), &version.join("content"));
        assert_eq!(
            studio.built_in_plugins_path(),
            &version.join("BuiltInPlugins")
        );
    }

    #[test]
    fn versions_layout_without_executable_is_not_installed() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("Versions").join("version-0")).unwrap();

        let result = RobloxStudio::locate_from_windows_directory(root.path().to_path_buf());

        assert!(matches!(result, Err(Error::NotInstalled)));
    }

    #[test]
    fn empty_root_is_not_installed() {
        let root = TempDir::new().unwrap();

        let result = RobloxStudio::locate_from_windows_directory(root.path().to_path_buf());

        assert!(matches!(result, Err(Error::NotInstalled)));
    }
}
