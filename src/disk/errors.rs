use super::*;
//---------------------------------------------------------------------------------------------------- Custom Error [TomlError]
#[derive(Debug)]
pub enum TomlError {
    Io(std::io::Error),
    Path(String),
    Serialize(toml::ser::Error),
    Deserialize(toml::de::Error),
    Merge(figment::Error),
    Format(std::fmt::Error),
    Parse(&'static str),
}

impl Display for TomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use TomlError::*;
        match self {
            Io(err) => write!(f, "{}: IO | {}", ERROR, err),
            Path(err) => write!(f, "{}: Path | {}", ERROR, err),
            Serialize(err) => write!(f, "{}: Serialize | {}", ERROR, err),
            Deserialize(err) => write!(f, "{}: Deserialize | {}", ERROR, err),
            Merge(err) => write!(f, "{}: Merge | {}", ERROR, err),
            Format(err) => write!(f, "{}: Format | {}", ERROR, err),
            Parse(err) => write!(f, "{}: Parse | {}", ERROR, err),
        }
    }
}

impl From<std::io::Error> for TomlError {
    fn from(err: std::io::Error) -> Self {
        TomlError::Io(err)
    }
}

impl From<std::fmt::Error> for TomlError {
    fn from(err: std::fmt::Error) -> Self {
        TomlError::Format(err)
    }
}
