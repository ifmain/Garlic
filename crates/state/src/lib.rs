pub type StateError = Box<dyn std::error::Error + Send + Sync>;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::Path,
};

pub fn read<T: DeserializeOwned>(path: &Path) -> Result<T, StateError> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}

/// Garlic's CLI and daemons coordinate writers with a sibling .lock file.
/// Keep the lock through replacement and preserve fields unknown to this build.
pub fn edit<T: DeserializeOwned + Serialize, R>(
    path: &Path,
    f: impl FnOnce(&mut T) -> Result<R, StateError>,
) -> Result<R, StateError> {
    edit_or_create(path, || Err(io_missing()), f)
}

fn io_missing() -> StateError {
    std::io::Error::new(std::io::ErrorKind::NotFound, "state file is missing").into()
}

pub fn edit_or_create<T: DeserializeOwned + Serialize, R>(
    path: &Path,
    init: impl FnOnce() -> Result<T, StateError>,
    f: impl FnOnce(&mut T) -> Result<R, StateError>,
) -> Result<R, StateError> {
    let mut name = OsString::from(path);
    name.push(".lock");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(name)?;
    lock.lock()?;
    let (mut raw, created): (serde_json::Value, bool) = match File::open(path) {
        Ok(file) => (serde_json::from_reader(BufReader::new(file))?, false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            (serde_json::to_value(init()?)?, true)
        }
        Err(e) => return Err(e.into()),
    };
    let mut data: T = serde_json::from_value(raw.clone())?;
    let before = serde_json::to_value(&data)?;
    let result = f(&mut data)?;
    let after = serde_json::to_value(&data)?;
    if created || before != after {
        apply_changes(&mut raw, &before, after);
        save(path, &raw)?;
    }
    Ok(result)
}

fn apply_changes(
    raw: &mut serde_json::Value,
    before: &serde_json::Value,
    after: serde_json::Value,
) {
    use serde_json::Value;
    if before == &after {
        return;
    }
    if let (Some(map), Value::Object(old), Value::Object(new)) =
        (raw.as_object_mut(), before, &after)
    {
        for k in old.keys().filter(|k| !new.contains_key(*k)) {
            map.remove(k);
        }
        for (k, v) in new {
            match (map.get_mut(k), old.get(k)) {
                (Some(raw_value), Some(previous)) => apply_changes(raw_value, previous, v.clone()),
                _ => {
                    map.insert(k.clone(), v.clone());
                }
            }
        }
    } else {
        *raw = after;
    }
}

pub fn save<T: Serialize>(path: &Path, data: &T) -> Result<(), StateError> {
    write(path, data, false)
}
pub fn create<T: Serialize>(path: &Path, data: &T) -> Result<(), StateError> {
    write(path, data, true)
}
fn write<T: Serialize>(path: &Path, data: &T, create_only: bool) -> Result<(), StateError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(tmp.as_file_mut(), data)?;
    tmp.write_all(b"\n")?;
    tmp.as_file().sync_all()?;
    if create_only {
        tmp.persist_noclobber(path)?;
    } else {
        tmp.persist(path)?;
    }
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}
