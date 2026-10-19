mod ping;

use std::path::Path;

use serde::{Deserialize, Serialize};

pub use ping::{ServerStatus, ping_server};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ResourcePackPolicy {
    Prompt,
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub name: String,
    pub address: String,
    /// Base64-encoded PNG, as stored by the game.
    pub icon: Option<String>,
    pub resource_packs: ResourcePackPolicy,
}

#[derive(Default, Serialize, Deserialize)]
struct ServersDat {
    #[serde(default)]
    servers: Vec<ServerNbt>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerNbt {
    #[serde(default)]
    name: String,
    #[serde(default)]
    ip: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accept_textures: Option<bool>,
}

impl From<ServerNbt> for Server {
    fn from(server: ServerNbt) -> Self {
        Self {
            name: server.name,
            address: server.ip,
            icon: server.icon.filter(|icon| !icon.is_empty()),
            resource_packs: match server.accept_textures {
                None => ResourcePackPolicy::Prompt,
                Some(true) => ResourcePackPolicy::Enabled,
                Some(false) => ResourcePackPolicy::Disabled,
            },
        }
    }
}

impl From<Server> for ServerNbt {
    fn from(server: Server) -> Self {
        Self {
            name: server.name,
            ip: server.address,
            icon: server.icon,
            accept_textures: match server.resource_packs {
                ResourcePackPolicy::Prompt => None,
                ResourcePackPolicy::Enabled => Some(true),
                ResourcePackPolicy::Disabled => Some(false),
            },
        }
    }
}

/// Reads the multiplayer server list (`servers.dat`, uncompressed NBT).
pub fn read_servers(path: &Path) -> Result<Vec<Server>> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(Error::io(path, err)),
    };
    let dat: ServersDat = fastnbt::from_bytes(&bytes)
        .map_err(|err| Error::InvalidInput(format!("unreadable {}: {err}", path.display())))?;
    Ok(dat.servers.into_iter().map(Into::into).collect())
}

pub fn write_servers(path: &Path, servers: &[Server]) -> Result<()> {
    let dat = ServersDat {
        servers: servers.iter().cloned().map(Into::into).collect(),
    };
    let bytes = fastnbt::to_bytes(&dat).map_err(|err| Error::InvalidInput(err.to_string()))?;
    crate::fs::write_atomic(path, &bytes)
}

/// Applies `edit` to the stored server list and writes it back.
pub fn edit_servers(path: &Path, edit: impl FnOnce(&mut Vec<Server>) -> Result<()>) -> Result<()> {
    let mut servers = read_servers(path)?;
    edit(&mut servers)?;
    for server in &servers {
        if server.address.trim().is_empty() {
            return Err(Error::InvalidInput(
                "server address must not be empty".into(),
            ));
        }
    }
    write_servers(path, &servers)
}

pub fn server_index(servers: &[Server], index: usize) -> Result<usize> {
    if index < servers.len() {
        Ok(index)
    } else {
        Err(Error::InvalidInput(format!(
            "no server at position {index}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(name: &str, policy: ResourcePackPolicy) -> Server {
        Server {
            name: name.into(),
            address: format!("{name}.example.com"),
            icon: None,
            resource_packs: policy,
        }
    }

    #[test]
    fn round_trips_servers_dat() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let path = dir.path().join("servers.dat");
        assert!(read_servers(&path)?.is_empty());

        let servers = vec![
            server("a", ResourcePackPolicy::Prompt),
            server("b", ResourcePackPolicy::Disabled),
        ];
        write_servers(&path, &servers)?;
        assert_eq!(read_servers(&path)?, servers);

        edit_servers(&path, |servers| {
            servers.swap(0, 1);
            Ok(())
        })?;
        assert_eq!(read_servers(&path)?[0].name, "b");
        assert!(
            edit_servers(&path, |servers| {
                servers[0].address.clear();
                Ok(())
            })
            .is_err()
        );
        Ok(())
    }
}
