use std::path::Path;

use serde::{Deserialize, Serialize};

use super::MicrosoftSession;
use crate::{Error, Result, fs};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// Minecraft profile UUID without dashes.
    pub id: String,
    pub username: String,
    pub kind: AccountKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microsoft: Option<MicrosoftSession>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AccountKind {
    Offline,
    Microsoft,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub id: String,
    pub username: String,
    pub kind: AccountKind,
    pub is_default: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AccountStore {
    accounts: Vec<Account>,
    default_id: Option<String>,
}

impl AccountStore {
    pub fn load(path: &Path) -> Result<Self> {
        Ok(fs::read_json(path)?.unwrap_or_default())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        fs::write_json(path, self)
    }

    pub fn summaries(&self) -> Vec<AccountSummary> {
        self.accounts
            .iter()
            .map(|account| AccountSummary {
                id: account.id.clone(),
                username: account.username.clone(),
                kind: account.kind.clone(),
                is_default: self.default_id.as_deref() == Some(&account.id),
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|account| account.id == id)
    }

    pub fn default_account(&self) -> Option<&Account> {
        self.default_id.as_deref().and_then(|id| self.get(id))
    }

    /// Adds or replaces an account; the first account becomes the default.
    pub fn upsert(&mut self, account: Account) {
        if self.default_id.is_none() {
            self.default_id = Some(account.id.clone());
        }
        match self
            .accounts
            .iter_mut()
            .find(|existing| existing.id == account.id)
        {
            Some(existing) => *existing = account,
            None => self.accounts.push(account),
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.accounts.retain(|account| account.id != id);
        if self.default_id.as_deref() == Some(id) {
            self.default_id = self.accounts.first().map(|account| account.id.clone());
        }
    }

    pub fn set_default(&mut self, id: &str) -> Result<()> {
        if self.get(id).is_none() {
            return Err(Error::UnknownAccount(id.to_owned()));
        }
        self.default_id = Some(id.to_owned());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offline(id: &str) -> Account {
        Account {
            id: id.into(),
            username: format!("player-{id}"),
            kind: AccountKind::Offline,
            microsoft: None,
        }
    }

    #[test]
    fn first_account_becomes_default_and_default_moves_on_removal() -> Result<()> {
        let mut store = AccountStore::default();
        store.upsert(offline("a"));
        store.upsert(offline("b"));
        assert_eq!(store.default_account().map(|a| a.id.as_str()), Some("a"));

        store.set_default("b")?;
        store.remove("b");
        assert_eq!(store.default_account().map(|a| a.id.as_str()), Some("a"));
        assert!(store.set_default("missing").is_err());
        Ok(())
    }

    #[test]
    fn upsert_replaces_existing() {
        let mut store = AccountStore::default();
        store.upsert(offline("a"));
        store.upsert(Account {
            username: "renamed".into(),
            ..offline("a")
        });
        assert_eq!(store.summaries().len(), 1);
        assert_eq!(store.summaries()[0].username, "renamed");
    }
}
