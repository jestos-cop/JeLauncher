use md5::{Digest, Md5};

use super::{Account, AccountKind};
use crate::{Error, Result};

/// Creates an offline account with the UUID vanilla servers assign to `username`.
pub fn offline_account(username: &str) -> Result<Account> {
    let username = username.trim();
    let valid_length = (3..=16).contains(&username.len());
    if !valid_length
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(Error::InvalidInput(
            "username must be 3–16 characters of letters, digits or underscores".into(),
        ));
    }
    Ok(Account {
        id: offline_uuid(username),
        username: username.to_owned(),
        kind: AccountKind::Offline,
    })
}

/// Same as Java's `UUID.nameUUIDFromBytes("OfflinePlayer:" + name)`.
fn offline_uuid(username: &str) -> String {
    let mut bytes: [u8; 16] = Md5::digest(format!("OfflinePlayer:{username}")).into();
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_vanilla_offline_uuid() -> Result<()> {
        assert_eq!(
            offline_account("Notch")?.id,
            "b50ad385829d3141a2167e7d7539ba7f"
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_names() {
        for name in ["ab", "this_name_is_way_too_long", "bad name", "über"] {
            assert!(offline_account(name).is_err(), "{name} should be rejected");
        }
    }
}
