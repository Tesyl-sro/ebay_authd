use crate::error::{Error, Result};
use configuration::Configuration;
use std::path::PathBuf;

pub mod configuration;

pub fn location() -> Result<PathBuf> {
    let mut home_dir = homedir::my_home().ok().flatten().ok_or(Error::NoHome)?;

    home_dir.push(".config/");
    home_dir.push("ebay_authd.yml");

    Ok(home_dir)
}

pub fn config_exists() -> Result<bool> {
    location().map(|path| path.is_file())
}

pub fn create_config() -> Result<()> {
    Ok(confy::store_path(location()?, Configuration::default())?)
}
