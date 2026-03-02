use std::{
    fs::{self, File},
    io::{self, Cursor, Read, Write},
    path::Path,
    sync::Arc,
};

use dashmap::DashMap;
use log::info;
use serde::{Deserialize, Serialize};
use srv_session::{
    AccountValue, DEFAULT_HASH_LEN, DEFAULT_SALT_LEN, DEFAULT_SESSION_LEN, KVTrait, SessionValue,
};

use crate::web::analyzer::Counter;

#[derive(Serialize, Deserialize)]
pub struct Storage {
    pub counter: Counter,
    pub accounts: AccountKV,
    pub sessions: SessionKV,
}

impl Storage {
    pub fn load_or_create(file_name: &Path) -> std::io::Result<Self> {
        if file_name.exists() {
            info!("Loading storage from file: {:?}", file_name);
            let mut file = std::fs::File::open(file_name)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let storage = from_cbor_bytes(&buf).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse storage file: {}", e),
                )
            })?;
            info!("Storage loaded successfully from file: {:?}", file_name);
            Ok(storage)
        } else {
            info!("Creating new storage file: {:?}", file_name);
            let storage = Self {
                accounts: AccountKV {
                    accounts: Arc::new(DashMap::new()),
                },
                sessions: SessionKV {
                    sessions: Arc::new(DashMap::new()),
                },
                counter: Counter::new(),
            };
            storage.save(file_name)?;
            info!("Storage file created successfully: {:?}", file_name);
            Ok(storage)
        }
    }

    pub fn save(&self, file_name: &Path) -> std::io::Result<()> {
        let bytes = to_cbor_bytes(self).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize storage file: {}", e),
            )
        })?;

        let tmp = file_name.with_added_extension("tmp");
        let old = file_name.with_added_extension("old");

        // 0. 前回の old が残ってたら消す
        if old.exists() {
            let _ = fs::remove_file(&old);
            // old の削除もディレクトリ更新なので fsync する
            let _ = fsync_parent_dir(file_name);
        }

        // 1. tmp に書く
        {
            let mut f = File::create(&tmp)?;
            f.write_all(&bytes)?;
            f.sync_all()?; // 2) tmp の内容を永続
        }

        // 3. file -> old
        if file_name.exists() {
            fs::rename(file_name, &old)?;
            fsync_parent_dir(file_name)?; // 4. rename 永続
        }

        // 5. tmp -> file
        fs::rename(&tmp, file_name)?;
        fsync_parent_dir(file_name)?; // 6. rename 永続

        Ok(())
    }
}

fn fsync_parent_dir(path: &Path) -> io::Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    #[cfg(windows)]
    {
        let _ = dir;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let d = File::open(dir)?;
        d.sync_all()
    }
}

fn to_cbor_bytes<T: Serialize>(v: &T) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
    let mut out = Vec::new();
    ciborium::ser::into_writer(v, &mut out)?;
    Ok(out)
}

fn from_cbor_bytes<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
) -> Result<T, ciborium::de::Error<std::io::Error>> {
    let mut cur = Cursor::new(bytes);
    let v = ciborium::de::from_reader(&mut cur)?;
    Ok(v)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionKV {
    pub sessions: Arc<DashMap<[u8; DEFAULT_SESSION_LEN], SessionValue<DEFAULT_SESSION_LEN>>>,
}

impl KVTrait<[u8; DEFAULT_SESSION_LEN], SessionValue<DEFAULT_SESSION_LEN>> for SessionKV {
    fn get(&self, key: &[u8; DEFAULT_SESSION_LEN]) -> Option<SessionValue<DEFAULT_SESSION_LEN>> {
        self.sessions.get(key).map(|entry| entry.value().clone())
    }

    fn set(&self, key: &[u8; DEFAULT_SESSION_LEN], value: SessionValue<DEFAULT_SESSION_LEN>) {
        self.sessions.insert(*key, value);
    }

    fn contains(&self, key: &[u8; DEFAULT_SESSION_LEN]) -> bool {
        self.sessions.contains_key(key)
    }

    fn delete(&self, key: &[u8; DEFAULT_SESSION_LEN]) -> bool {
        self.sessions.remove(key).is_some()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountKV {
    pub accounts:
        Arc<DashMap<String, AccountValue<DEFAULT_SALT_LEN, DEFAULT_HASH_LEN, DEFAULT_SESSION_LEN>>>,
}

impl KVTrait<str, AccountValue<DEFAULT_SALT_LEN, DEFAULT_HASH_LEN, DEFAULT_SESSION_LEN>>
    for AccountKV
{
    fn get(
        &self,
        key: &str,
    ) -> Option<AccountValue<DEFAULT_SALT_LEN, DEFAULT_HASH_LEN, DEFAULT_SESSION_LEN>> {
        self.accounts.get(key).map(|entry| entry.value().clone())
    }

    fn set(
        &self,
        key: &str,
        value: AccountValue<DEFAULT_SALT_LEN, DEFAULT_HASH_LEN, DEFAULT_SESSION_LEN>,
    ) {
        self.accounts.insert(key.to_string(), value);
    }

    fn contains(&self, key: &str) -> bool {
        self.accounts.contains_key(key)
    }

    fn delete(&self, key: &str) -> bool {
        self.accounts.remove(key).is_some()
    }
}
