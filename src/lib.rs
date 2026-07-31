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

/// Cache for the expensive platform-specific lookup.
static PLATFORM_STUDIO: OnceLock<RobloxStudio> = OnceLock::new();

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
/// * If `ROBLOX_STUDIO_PATH` is set, it must point directly to the executable.
/// * Otherwise the platform-specific executable is located and cached.
pub fn locate() -> Result<PathBuf> {
    if let Some(result) = locate_executable_from_env() {
        return result;
    }

    if let Some(studio) = PLATFORM_STUDIO.get() {
        return Ok(studio.application.clone());
    }

    let studio = RobloxStudio::locate_target_specific()?;
    let application = studio.application.clone();
    let _ = PLATFORM_STUDIO.set(studio);
    Ok(application)
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
    /// The result of platform-specific detection is cached after the first successful
    /// call, making subsequent calls O(1).
    pub fn locate() -> Result<RobloxStudio> {
        if let Some(result) = Self::locate_from_env() {
            return result;
        }

        if let Some(studio) = PLATFORM_STUDIO.get() {
            return Ok(studio.clone());
        }

        let studio = Self::locate_target_specific()?;
        let _ = PLATFORM_STUDIO.set(studio.clone());
        Ok(studio)
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
        let plugins = Self::locate_plugins_on_windows()?;

        if content_folder_path.is_dir() {
            Ok(RobloxStudio {
                content: content_folder_path,
                application: root.join("RobloxStudioBeta.exe"),
                built_in_plugins: root.join("BuiltInPlugins"),
                plugins,
            })
        } else {
            let versions = root.join("Versions");

            if versions.is_dir() {
                let version_dir = fs::read_dir(&versions)
                    .map_err(|_| Error::NotInstalled)?
                    .filter_map(|entry| entry.ok())
                    .find_map(|entry| {
                        let version = entry.path();
                        let application = version.join("RobloxStudioBeta.exe");
                        application.is_file().then_some(version)
                    })
                    .ok_or(Error::NotInstalled)?;

                Ok(RobloxStudio {
                    content: version_dir.join("content"),
                    application: version_dir.join("RobloxStudioBeta.exe"),
                    built_in_plugins: version_dir.join("BuiltInPlugins"),
                    plugins,
                })
            } else {
                Err(Error::NotInstalled)
            }
        }
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
            .filter(|e| e.path().is_dir())
            .find_map(|e| {
                let path = e.path().join("RobloxStudioBeta.exe");
                path.is_file().then(|| e.path())
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
