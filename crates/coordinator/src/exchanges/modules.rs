use garlic_relay_data::{envelope, gex4};
use garlic_relay_session::SessionError;
use serde::Deserialize;
use std::{collections::BTreeMap, fs::File, io::Read, path::Path, sync::RwLock};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Frame {
    Fragments,
    Tlv,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Body {
    Base,
    SelectiveAck,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Module {
    pub version: u8,
    pub frame: Frame,
    pub body: Body,
    pub record_size: Option<usize>,
}
impl Module {
    pub(super) fn limit(self) -> usize {
        match self.frame {
            Frame::Fragments => envelope::MAX_RECORD,
            Frame::Tlv => gex4::MAX_MESSAGE,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    abi: u8,
    modules: Vec<Module>,
}

type Table = BTreeMap<u8, Module>;
pub(crate) struct Registry(RwLock<Table>);

const DEFAULT: &str = include_str!("modules-v1.json");
fn compile(text: &[u8]) -> Result<Table, SessionError> {
    let manifest: Manifest = serde_json::from_slice(text)?;
    if manifest.abi != 1 {
        return Err("unsupported protocol module ABI".into());
    }
    let mut table = Table::new();
    for m in manifest.modules {
        let compatible = matches!(
            (m.version, m.frame, m.body),
            (2, Frame::Fragments, Body::Base)
                | (3, Frame::Fragments, Body::SelectiveAck)
                | (4, Frame::Tlv, Body::Base)
                | (128..=255, _, _)
        );
        let size_ok = match m.frame {
            Frame::Fragments => (1024..=envelope::MAX_RECORD)
                .contains(&m.record_size.unwrap_or(envelope::MAX_RECORD)),
            Frame::Tlv => m.record_size.is_none(),
        };
        if !compatible || !size_ok || table.insert(m.version, m).is_some() {
            return Err("incompatible or duplicate exchange module".into());
        }
    }
    Ok(table)
}
fn read(path: &Path) -> Result<Table, SessionError> {
    let mut data = Vec::new();
    File::open(path)?.take(16385).read_to_end(&mut data)?;
    if data.len() > 16384 {
        return Err("module manifest exceeds 16 KiB".into());
    }
    compile(&data)
}
impl Registry {
    pub(crate) fn load(path: Option<&Path>) -> Result<Self, SessionError> {
        let mut modules = match path {
            Some(p) => read(p)?,
            None => compile(DEFAULT.as_bytes())?,
        };
        let allowed = super::enabled_versions().map_err(|e| e.1)?;
        if let Some(allowed) = allowed {
            modules.retain(|v, _| allowed.contains(v));
        }
        Ok(Self(RwLock::new(modules)))
    }
    pub(crate) fn reload(&self, path: &Path) -> Result<bool, SessionError> {
        let next = Self::load(Some(path))?;
        let next = next.0.into_inner().unwrap();
        let mut active = self.0.write().unwrap();
        if *active == next {
            return Ok(false);
        }
        *active = next;
        Ok(true)
    }
    pub(super) fn get(&self, v: u8) -> Option<Module> {
        self.0.read().unwrap().get(&v).copied()
    }
    pub(crate) fn versions(&self) -> Vec<u8> {
        self.0.read().unwrap().keys().copied().collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_reload_preserves_table_and_open_lane_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("modules.json");
        std::fs::write(&path, DEFAULT).unwrap();
        let registry = Registry::load(Some(&path)).unwrap();
        let original = registry.get(2).unwrap();
        for invalid in [
            r#"{"abi":2,"modules":[]}"#,
            r#"{"abi":1,"modules":[{"version":2,"frame":"tlv","body":"base"}]}"#,
            r#"{"abi":1,"modules":[{"version":2,"frame":"fragments","body":"base","record_size":20}]}"#,
            r#"{"abi":1,"modules":[{"version":2,"frame":"fragments","body":"base"},{"version":2,"frame":"fragments","body":"base"}]}"#,
        ] {
            std::fs::write(&path, invalid).unwrap();
            assert!(registry.reload(&path).is_err());
            assert_eq!(registry.get(2), Some(original));
        }
        std::fs::write(&path,r#"{"abi":1,"modules":[{"version":2,"frame":"fragments","body":"base","record_size":4096}]}"#).unwrap();
        assert!(registry.reload(&path).unwrap());
        assert!(!registry.reload(&path).unwrap());
        assert_eq!(registry.get(2).unwrap().record_size, Some(4096));
        assert_eq!(original.record_size, None);
    }
}
