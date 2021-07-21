use std::env;
use config::{ConfigError, Config, Environment, File};
use serde::Deserialize;

pub fn load_settings<'a, T:Deserialize<'a>>(app_env_prefix: &str, app_name: &str) -> Result<T, ConfigError> {
    let mut s = Config::default();
    let base_path = format!("config/{}", app_name);
    // Start off by merging in the "default.toml" configuration file
    s.merge(File::with_name(&format!("{}/default.toml", base_path)))?;

    // Add in the current environment file
    // Default to 'development' env
    // Note that this file is _optional_
    let env_name = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
    s.merge(File::with_name(&format!("{}/{}", base_path, env_name)).required(false))?;
    s.set("environment.name", env_name)?;
    // Add in a local configuration file
    // This file shouldn't be checked in to git
    s.merge(File::with_name(&format!("{}/local", base_path)).required(false))?;

    // Add in settings from the environment (with a prefix of APP)
    // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
    s.merge(Environment::with_prefix(app_env_prefix))?;
    // Now that we're done, let's access our configuration

    // You can deserialize (and thus freeze) the entire configuration as
    s.try_into()
}
