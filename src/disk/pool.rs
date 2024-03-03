use super::*;
//---------------------------------------------------------------------------------------------------- [Pool] impl
impl Pool {
    pub fn p2pool() -> Self {
        Self {
            rig: GUPAX_VERSION_UNDERSCORE.to_string(),
            ip: "localhost".to_string(),
            port: "3333".to_string(),
        }
    }

    pub fn new_vec() -> Vec<(String, Self)> {
        vec![("Local P2Pool".to_string(), Self::p2pool())]
    }

    pub fn new_tuple() -> (String, Self) {
        ("Local P2Pool".to_string(), Self::p2pool())
    }

    pub fn from_str_to_vec(string: &str) -> Result<Vec<(String, Self)>, TomlError> {
        let pools: toml::map::Map<String, toml::Value> = match toml::de::from_str(string) {
            Ok(map) => {
                info!("Pool | Parse ... OK");
                map
            }
            Err(err) => {
                error!("Pool | String parse ... FAIL ... {}", err);
                return Err(TomlError::Deserialize(err));
            }
        };
        let size = pools.keys().len();
        let mut vec = Vec::with_capacity(size);
        // We have to do [.as_str()] -> [.to_string()] to get rid of the \"...\" that gets added on.
        for (key, values) in pools.iter() {
            let rig = match values.get("rig") {
                Some(rig) => match rig.as_str() {
                    Some(rig) => rig.to_string(),
                    None => {
                        error!("Pool | [None] at [rig] parse");
                        return Err(TomlError::Parse("[None] at [rig] parse"));
                    }
                },
                None => {
                    error!("Pool | [None] at [rig] parse");
                    return Err(TomlError::Parse("[None] at [rig] parse"));
                }
            };
            let ip = match values.get("ip") {
                Some(ip) => match ip.as_str() {
                    Some(ip) => ip.to_string(),
                    None => {
                        error!("Pool | [None] at [ip] parse");
                        return Err(TomlError::Parse("[None] at [ip] parse"));
                    }
                },
                None => {
                    error!("Pool | [None] at [ip] parse");
                    return Err(TomlError::Parse("[None] at [ip] parse"));
                }
            };
            let port = match values.get("port") {
                Some(port) => match port.as_str() {
                    Some(port) => port.to_string(),
                    None => {
                        error!("Pool | [None] at [port] parse");
                        return Err(TomlError::Parse("[None] at [port] parse"));
                    }
                },
                None => {
                    error!("Pool | [None] at [port] parse");
                    return Err(TomlError::Parse("[None] at [port] parse"));
                }
            };
            let pool = Pool { rig, ip, port };
            vec.push((key.clone(), pool));
        }
        Ok(vec)
    }

    pub fn to_string(vec: &[(String, Self)]) -> Result<String, TomlError> {
        let mut toml = String::new();
        for (key, value) in vec.iter() {
            write!(
                toml,
                "[\'{}\']\nrig = {:#?}\nip = {:#?}\nport = {:#?}\n\n",
                key, value.rig, value.ip, value.port,
            )?;
        }
        Ok(toml)
    }

    pub fn get(path: &PathBuf) -> Result<Vec<(String, Self)>, TomlError> {
        // Read
        let file = File::Pool;
        let string = match read_to_string(file, path) {
            Ok(string) => string,
            // Create
            _ => {
                Self::create_new(path)?;
                read_to_string(file, path)?
            }
        };
        // Deserialize
        Self::from_str_to_vec(&string)
    }

    pub fn create_new(path: &PathBuf) -> Result<Vec<(String, Self)>, TomlError> {
        info!("Pool | Creating new default...");
        let new = Self::new_vec();
        let string = Self::to_string(&Self::new_vec())?;
        fs::write(path, string)?;
        info!("Pool | Write ... OK");
        Ok(new)
    }

    pub fn save(vec: &[(String, Self)], path: &PathBuf) -> Result<(), TomlError> {
        info!("Pool | Saving to disk ... [{}]", path.display());
        let string = Self::to_string(vec)?;
        match fs::write(path, string) {
            Ok(_) => {
                info!("Pool | Save ... OK");
                Ok(())
            }
            Err(err) => {
                error!("Pool | Couldn't overwrite file");
                Err(TomlError::Io(err))
            }
        }
    }
}
//---------------------------------------------------------------------------------------------------- [Pool] Struct
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Pool {
    pub rig: String,
    pub ip: String,
    pub port: String,
}
