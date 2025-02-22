use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use color_eyre::eyre::{bail, Result};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct Config {
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct Profile {
    pub server_base: Option<String>,
    pub server_api: Option<String>,
    pub server_identity: Option<String>,
}

// TODO: This could probably be derived with a macro if we start adding more fields
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum ProfileKey {
    server_base,
    server_api,
    server_identity,
}

impl ProfileKey {
    fn update_profile_value(&self, p: &mut Profile, value: String) {
        match self {
            ProfileKey::server_base => p.server_base = Some(value),
            ProfileKey::server_api => p.server_api = Some(value),
            ProfileKey::server_identity => p.server_identity = Some(value),
        }
    }
}

pub(crate) const FILENAME: &str = "config";
pub(crate) const DIRECTORY: &str = ".bws";

fn get_config_path(config_file: Option<&Path>, ensure_folder_exists: bool) -> PathBuf {
    let config_file = config_file.map(ToOwned::to_owned).unwrap_or_else(|| {
        let base_dirs = BaseDirs::new().unwrap();
        base_dirs.home_dir().join(DIRECTORY).join(FILENAME)
    });

    if ensure_folder_exists {
        if let Some(parent_folder) = config_file.parent() {
            std::fs::create_dir_all(parent_folder).unwrap();
        }
    }

    config_file
}

pub(crate) fn load_config(config_file: Option<&Path>, must_exist: bool) -> Result<Config> {
    let file = get_config_path(config_file, false);

    let content = match file.exists() {
        true => read_to_string(file),
        false if must_exist => bail!("Config file doesn't exist"),
        false => return Ok(Config::default()),
    };

    let config: Config = toml::from_str(&content?)?;
    Ok(config)
}

fn write_config(config: Config, config_file: Option<&Path>) -> Result<()> {
    let file = get_config_path(config_file, true);

    let content = toml::to_string_pretty(&config)?;

    std::fs::write(file, content)?;
    Ok(())
}

pub(crate) fn update_profile(
    config_file: Option<&Path>,
    profile: String,
    name: ProfileKey,
    value: String,
) -> Result<()> {
    let mut config = load_config(config_file, false)?;

    let p = config.profiles.entry(profile).or_default();
    name.update_profile_value(p, value);

    write_config(config, config_file)?;
    Ok(())
}

pub(crate) fn delete_profile(config_file: Option<&Path>, profile: String) -> Result<()> {
    let mut config = load_config(config_file, true)?;

    if !config.profiles.contains_key(&profile) {
        bail!("Profile does not exist");
    }

    config.profiles.remove(&profile);

    write_config(config, config_file)?;
    Ok(())
}

impl Profile {
    pub(crate) fn from_url(url: &str) -> Result<Profile> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            bail!("Server URL must start with http:// or https://, the provided URL is: `{url}`");
        }

        Ok(Profile {
            server_base: Some(url.to_string()),
            server_api: None,
            server_identity: None,
        })
    }
    pub(crate) fn api_url(&self) -> Result<String> {
        if let Some(api) = &self.server_api {
            return Ok(api.clone());
        }

        if let Some(base) = &self.server_base {
            return Ok(format!("{base}/api"));
        }

        bail!("Profile has no `server_base` or `server_api`")
    }

    pub(crate) fn identity_url(&self) -> Result<String> {
        if let Some(identity) = &self.server_identity {
            return Ok(identity.clone());
        }

        if let Some(base) = &self.server_base {
            return Ok(format!("{base}/identity"));
        }

        bail!("Profile has no `server_base` or `server_identity`")
    }
}

impl Config {
    pub(crate) fn select_profile(
        &self,
        profile: &str,
        profile_defined: bool,
    ) -> Result<Option<Profile>> {
        if let Some(profile) = self.profiles.get(profile) {
            return Ok(Some(profile.clone()));
        }

        if profile_defined {
            bail!("The specified profile does not exist");
        }

        if let Some(profile) = self.profiles.get("default") {
            return Ok(Some(profile.clone()));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn config_doesnt_exist() {
        let c = load_config(Some(Path::new("non_existing")), true);
        assert!(c.is_err());

        let c = load_config(None, false);
        assert!(c.is_ok());
    }

    #[test]
    fn config_exist() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile.as_file(), "[profiles]").unwrap();

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true);
        let config = c.unwrap();
        assert_eq!(0, config.profiles.len());
    }

    #[test]
    fn config_exist_with_profile() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(
            tmpfile.as_file(),
            "[profiles.default]
        server_base = \"https://bitwarden.com\"
        "
        )
        .unwrap();

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true);
        assert_eq!(
            "https://bitwarden.com",
            c.unwrap().profiles["default"].server_base.as_ref().unwrap()
        );
    }
}
