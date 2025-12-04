// Gupax
//
// Copyright (c) 2024-2025 Cyrix126
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
            Io(err) => write!(f, "{ERROR}: IO | {err}"),
            Path(err) => write!(f, "{ERROR}: Path | {err}"),
            Serialize(err) => write!(f, "{ERROR}: Serialize | {err}"),
            Deserialize(err) => write!(f, "{ERROR}: Deserialize | {err}"),
            Merge(err) => write!(f, "{ERROR}: Merge | {err}"),
            Format(err) => write!(f, "{ERROR}: Format | {err}"),
            Parse(err) => write!(f, "{ERROR}: Parse | {err}"),
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
