use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::{Error, Result};

const DEFAULT_PORT: u16 = 25565;
const TIMEOUT: Duration = Duration::from_secs(5);
/// Servers answer status requests from any client version; -1 means "not logging in".
const STATUS_PROTOCOL: i32 = -1;
const MAX_PACKET: usize = 1 << 21;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub version: String,
    pub players_online: u32,
    pub players_max: u32,
    pub motd: String,
    /// `data:image/png;base64,...` URL.
    pub favicon: Option<String>,
    pub latency_ms: u32,
    /// Names of some of the players online, when the server shares them.
    pub players: Vec<String>,
}

#[derive(Deserialize)]
struct StatusResponse {
    version: Option<VersionInfo>,
    players: Option<Players>,
    description: Option<Value>,
    favicon: Option<String>,
}

#[derive(Deserialize)]
struct VersionInfo {
    name: String,
}

#[derive(Deserialize)]
struct Players {
    online: u32,
    max: u32,
    #[serde(default)]
    sample: Vec<PlayerSample>,
}

#[derive(Deserialize)]
struct PlayerSample {
    name: String,
}

/// Queries a server with the Server List Ping protocol.
pub async fn ping_server(address: &str) -> Result<ServerStatus> {
    let (host, port) = parse_address(address)?;
    tokio::time::timeout(TIMEOUT, query(&host, port))
        .await
        .map_err(|_| Error::InvalidInput(format!("{address} did not respond")))?
        .map_err(|err| Error::io(address, err))
}

async fn query(host: &str, port: u16) -> std::io::Result<ServerStatus> {
    let started = Instant::now();
    let mut stream = TcpStream::connect((host, port)).await?;

    let mut handshake = Vec::new();
    write_varint(&mut handshake, 0);
    write_varint(&mut handshake, STATUS_PROTOCOL);
    write_string(&mut handshake, host);
    handshake.extend_from_slice(&port.to_be_bytes());
    write_varint(&mut handshake, 1);
    send_packet(&mut stream, &handshake).await?;
    send_packet(&mut stream, &[0]).await?;

    let length = read_varint(&mut stream).await?;
    let length = usize::try_from(length)
        .ok()
        .filter(|&length| length <= MAX_PACKET)
        .ok_or_else(|| invalid("bad packet length"))?;
    let mut packet = vec![0; length];
    stream.read_exact(&mut packet).await?;
    let latency_ms = u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);

    let mut cursor = packet.as_slice();
    if read_varint(&mut cursor).await? != 0 {
        return Err(invalid("unexpected packet"));
    }
    let json_length = usize::try_from(read_varint(&mut cursor).await?)
        .map_err(|_| invalid("bad string length"))?;
    let json = cursor
        .get(..json_length)
        .ok_or_else(|| invalid("truncated response"))?;
    let response: StatusResponse =
        serde_json::from_slice(json).map_err(|err| invalid(&err.to_string()))?;
    Ok(ServerStatus {
        version: response
            .version
            .map(|version| version.name)
            .unwrap_or_default(),
        players_online: response
            .players
            .as_ref()
            .map_or(0, |players| players.online),
        players_max: response.players.as_ref().map_or(0, |players| players.max),
        motd: response
            .description
            .as_ref()
            .map(|description| strip_formatting(&plain_text(description)))
            .unwrap_or_default(),
        players: response
            .players
            .map(|players| player_names(players.sample))
            .unwrap_or_default(),
        favicon: response.favicon,
        latency_ms,
    })
}

/// Names from the player sample. Some servers fill it with text lines instead; those are kept as text.
fn player_names(sample: Vec<PlayerSample>) -> Vec<String> {
    sample
        .into_iter()
        .map(|player| strip_formatting(&player.name).trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect()
}

pub fn parse_address(address: &str) -> Result<(String, u16)> {
    let address = address.trim();
    let invalid_address = || Error::InvalidInput(format!("invalid server address: {address}"));
    let (host, port) = if let Some(rest) = address.strip_prefix('[') {
        let (host, rest) = rest.split_once(']').ok_or_else(invalid_address)?;
        (host, rest.strip_prefix(':'))
    } else {
        match address.rsplit_once(':') {
            Some((host, port)) if !host.contains(':') => (host, Some(port)),
            _ => (address, None),
        }
    };
    if host.is_empty() {
        return Err(invalid_address());
    }
    let port = match port {
        Some(port) => port.parse().map_err(|_| invalid_address())?,
        None => DEFAULT_PORT,
    };
    Ok((host.to_owned(), port))
}

/// Flattens a chat component (string, object with `text`/`extra`, or array) to plain text.
fn plain_text(component: &Value) -> String {
    match component {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts.iter().map(plain_text).collect(),
        Value::Object(object) => {
            let mut text = object
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if let Some(Value::Array(extra)) = object.get("extra") {
                text.extend(extra.iter().map(plain_text));
            }
            text
        }
        _ => String::new(),
    }
}

/// Removes legacy `§x` formatting codes.
fn strip_formatting(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else {
            plain.push(c);
        }
    }
    plain
}

async fn send_packet(stream: &mut TcpStream, payload: &[u8]) -> std::io::Result<()> {
    let mut packet = Vec::with_capacity(payload.len() + 5);
    write_varint(
        &mut packet,
        i32::try_from(payload.len()).unwrap_or(i32::MAX),
    );
    packet.extend_from_slice(payload);
    stream.write_all(&packet).await
}

fn write_varint(buffer: &mut Vec<u8>, value: i32) {
    let mut value = u32::from_ne_bytes(value.to_ne_bytes());
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            buffer.push(byte);
            return;
        }
        buffer.push(byte | 0x80);
    }
}

fn write_string(buffer: &mut Vec<u8>, value: &str) {
    write_varint(buffer, i32::try_from(value.len()).unwrap_or(i32::MAX));
    buffer.extend_from_slice(value.as_bytes());
}

async fn read_varint(reader: &mut (impl AsyncReadExt + Unpin)) -> std::io::Result<i32> {
    let mut value: u32 = 0;
    for shift in (0..35).step_by(7) {
        let byte = reader.read_u8().await?;
        value |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(i32::from_ne_bytes(value.to_ne_bytes()));
        }
    }
    Err(invalid("varint is too long"))
}

fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_addresses() -> Result<()> {
        assert_eq!(
            parse_address("mc.example.com")?,
            ("mc.example.com".into(), 25565)
        );
        assert_eq!(
            parse_address("mc.example.com:25570")?,
            ("mc.example.com".into(), 25570)
        );
        assert_eq!(parse_address("[::1]:25570")?, ("::1".into(), 25570));
        assert_eq!(parse_address("::1")?, ("::1".into(), 25565));
        assert!(parse_address("host:notaport").is_err());
        assert!(parse_address(":25565").is_err());
        Ok(())
    }

    #[test]
    fn flattens_motd_components() -> std::result::Result<(), serde_json::Error> {
        let motd: Value = serde_json::from_str(
            r#"{"text":"§aHello ","extra":[{"text":"World","bold":true},"!"]}"#,
        )?;
        assert_eq!(strip_formatting(&plain_text(&motd)), "Hello World!");
        Ok(())
    }

    #[tokio::test]
    async fn varints_round_trip() -> std::io::Result<()> {
        for value in [0, 1, 127, 128, 25565, -1, i32::MAX] {
            let mut buffer = Vec::new();
            write_varint(&mut buffer, value);
            assert_eq!(read_varint(&mut buffer.as_slice()).await?, value);
        }
        Ok(())
    }

    #[tokio::test]
    async fn pings_a_status_server() -> std::io::Result<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await?;
            for _ in 0..2 {
                let length = read_varint(&mut socket).await?;
                let mut packet = vec![0; usize::try_from(length).unwrap_or(0)];
                socket.read_exact(&mut packet).await?;
            }
            let mut payload = vec![0];
            write_string(
                &mut payload,
                r#"{"version":{"name":"1.21.4","protocol":769},"players":{"online":3,"max":20,"sample":[{"name":"Steve","id":"0"},{"name":"§aAlex","id":"1"},{"name":" ","id":"2"}]},"description":"A §bserver"}"#,
            );
            let mut packet = Vec::new();
            write_varint(&mut packet, i32::try_from(payload.len()).unwrap_or(0));
            packet.extend_from_slice(&payload);
            socket.write_all(&packet).await
        });

        let status = ping_server(&format!("127.0.0.1:{port}"))
            .await
            .map_err(|err| invalid(&err.to_string()))?;
        server.await.map_err(|err| invalid(&err.to_string()))??;
        assert_eq!(status.version, "1.21.4");
        assert_eq!((status.players_online, status.players_max), (3, 20));
        assert_eq!(status.motd, "A server");
        assert_eq!(status.players, ["Steve", "Alex"]);
        Ok(())
    }
}
