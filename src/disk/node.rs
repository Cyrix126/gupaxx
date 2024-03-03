use crate::disk::TomlError;
use crate::disk::*;
use serde::{Deserialize, Serialize};
//---------------------------------------------------------------------------------------------------- [Node] Impl
impl Node {
    pub fn localhost() -> Self {
        Self {
            ip: "localhost".to_string(),
            rpc: "18081".to_string(),
            zmq: "18083".to_string(),
        }
    }

    pub fn new_vec() -> Vec<(String, Self)> {
        vec![("Local Monero Node".to_string(), Self::localhost())]
    }

    pub fn new_tuple() -> (String, Self) {
        ("Local Monero Node".to_string(), Self::localhost())
    }

    // Convert [String] to [Node] Vec
    pub fn from_str_to_vec(string: &str) -> Result<Vec<(String, Self)>, TomlError> {
        let nodes: toml::map::Map<String, toml::Value> = match toml::de::from_str(string) {
            Ok(map) => {
                info!("Node | Parse ... OK");
                map
            }
            Err(err) => {
                error!("Node | String parse ... FAIL ... {}", err);
                return Err(TomlError::Deserialize(err));
            }
        };
        let size = nodes.keys().len();
        let mut vec = Vec::with_capacity(size);
        for (key, values) in nodes.iter() {
            let ip = match values.get("ip") {
                Some(ip) => match ip.as_str() {
                    Some(ip) => ip.to_string(),
                    None => {
                        error!("Node | [None] at [ip] parse");
                        return Err(TomlError::Parse("[None] at [ip] parse"));
                    }
                },
                None => {
                    error!("Node | [None] at [ip] parse");
                    return Err(TomlError::Parse("[None] at [ip] parse"));
                }
            };
            let rpc = match values.get("rpc") {
                Some(rpc) => match rpc.as_str() {
                    Some(rpc) => rpc.to_string(),
                    None => {
                        error!("Node | [None] at [rpc] parse");
                        return Err(TomlError::Parse("[None] at [rpc] parse"));
                    }
                },
                None => {
                    error!("Node | [None] at [rpc] parse");
                    return Err(TomlError::Parse("[None] at [rpc] parse"));
                }
            };
            let zmq = match values.get("zmq") {
                Some(zmq) => match zmq.as_str() {
                    Some(zmq) => zmq.to_string(),
                    None => {
                        error!("Node | [None] at [zmq] parse");
                        return Err(TomlError::Parse("[None] at [zmq] parse"));
                    }
                },
                None => {
                    error!("Node | [None] at [zmq] parse");
                    return Err(TomlError::Parse("[None] at [zmq] parse"));
                }
            };
            let node = Node { ip, rpc, zmq };
            vec.push((key.clone(), node));
        }
        Ok(vec)
    }

    // Convert [Vec<(String, Self)>] into [String]
    // that can be written as a proper TOML file
    pub fn to_string(vec: &[(String, Self)]) -> Result<String, TomlError> {
        let mut toml = String::new();
        for (key, value) in vec.iter() {
            write!(
                toml,
                "[\'{}\']\nip = {:#?}\nrpc = {:#?}\nzmq = {:#?}\n\n",
                key, value.ip, value.rpc, value.zmq,
            )?;
        }
        Ok(toml)
    }

    // Combination of multiple functions:
    //   1. Attempt to read file from path into [String]
    //      |_ Create a default file if not found
    //   2. Deserialize [String] into a proper [Struct]
    //      |_ Attempt to merge if deserialization fails
    pub fn get(path: &PathBuf) -> Result<Vec<(String, Self)>, TomlError> {
        // Read
        let file = File::Node;
        let string = match read_to_string(file, path) {
            Ok(string) => string,
            // Create
            _ => {
                Self::create_new(path)?;
                read_to_string(file, path)?
            }
        };
        // Deserialize, attempt merge if failed
        Self::from_str_to_vec(&string)
    }

    // Completely overwrite current [node.toml]
    // with a new default version, and return [Vec<String, Self>].
    pub fn create_new(path: &PathBuf) -> Result<Vec<(String, Self)>, TomlError> {
        info!("Node | Creating new default...");
        let new = Self::new_vec();
        let string = Self::to_string(&Self::new_vec())?;
        fs::write(path, string)?;
        info!("Node | Write ... OK");
        Ok(new)
    }

    // Save [Node] onto disk file [node.toml]
    pub fn save(vec: &[(String, Self)], path: &PathBuf) -> Result<(), TomlError> {
        info!("Node | Saving to disk ... [{}]", path.display());
        let string = Self::to_string(vec)?;
        match fs::write(path, string) {
            Ok(_) => {
                info!("Node | Save ... OK");
                Ok(())
            }
            Err(err) => {
                error!("Node | Couldn't overwrite file");
                Err(TomlError::Io(err))
            }
        }
    }

    //	pub fn merge(old: &String) -> Result<Self, TomlError> {
    //		info!("Node | Starting TOML merge...");
    //		let default = match toml::ser::to_string(&Self::new()) {
    //			Ok(string) => { info!("Node | Default TOML parse ... OK"); string },
    //			Err(err) => { error!("Node | Couldn't parse default TOML into string"); return Err(TomlError::Serialize(err)) },
    //		};
    //		let mut new: Self = match Figment::new().merge(Toml::string(&old)).merge(Toml::string(&default)).extract() {
    //			Ok(new) => { info!("Node | TOML merge ... OK"); new },
    //			Err(err) => { error!("Node | Couldn't merge default + old TOML"); return Err(TomlError::Merge(err)) },
    //		};
    //		// Attempt save
    //		Self::save(&mut new)?;
    //		Ok(new)
    //	}
}
//---------------------------------------------------------------------------------------------------- [Node] Struct
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Node {
    pub ip: String,
    pub rpc: String,
    pub zmq: String,
}
