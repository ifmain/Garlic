use super::direct_proxy::Egress;
use garlic_state::StateError;
use serde_json::{json, Value};
use std::{
    io::{self, Write},
    net::Ipv4Addr,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, SetLastError, ERROR_ALREADY_EXISTS, HANDLE},
    System::Threading::{CreateMutexW, CREATE_NO_WINDOW},
};

const NETWORK_SCRIPT: &str = include_str!("windows-network.ps1");

fn apply(input: &Value) -> Result<Vec<u8>, StateError> {
    let program = PathBuf::from(std::env::var_os("SystemRoot").ok_or("SystemRoot is unavailable")?)
        .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    let mut child = Command::new(program)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            NETWORK_SCRIPT,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or("VPN network configuration input is unavailable")?;
    let write = stdin.write_all(&serde_json::to_vec(input)?);
    drop(stdin);
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "Windows VPN network configuration failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    write?;
    Ok(output.stdout)
}

pub struct Guard {
    handle: HANDLE,
    state: Option<Value>,
}

impl Guard {
    pub fn lock() -> Result<Self, StateError> {
        // The recovery process holds the same named object until it has finished cleaning up.
        let name: Vec<u16> = "Global\\GarlicVPNNetworkOwner"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let handle = unsafe {
            SetLastError(0);
            CreateMutexW(std::ptr::null(), 0, name.as_ptr())
        };
        if handle.is_null() {
            return Err(io::Error::last_os_error().into());
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            return Err("another Garlic VPN instance is active or cleaning up".into());
        }
        Ok(Self {
            handle,
            state: None,
        })
    }

    pub fn enable(&mut self, bypass: impl IntoIterator<Item = Ipv4Addr>) -> Result<Egress, StateError> {
        let bypass: Vec<_> = bypass.into_iter().map(|ip| format!("{ip}/32")).collect();
        let result = apply(&json!({
            "operation": "enable",
            "owner_pid": std::process::id(),
            "gateway": tproxy_config::TUN_GATEWAY.to_string(),
            "bypass": bypass
        }))?;
        let state: Value = serde_json::from_slice(&result)?;
        let egress = serde_json::from_value(state["direct_egress"].clone());
        self.state = Some(state);
        Ok(egress?)
    }

    pub fn activate(&mut self) -> Result<(), StateError> {
        let state = self.state.as_ref().ok_or("VPN routes were not prepared")?;
        let result = apply(&json!({"operation":"activate","state":state}))?;
        self.state = Some(serde_json::from_slice(&result)?);
        Ok(())
    }

    pub fn disable(&mut self) -> Result<(), StateError> {
        if let Some(state) = &self.state {
            apply(&json!({"operation":"disable","state":state}))?;
            self.state = None;
        }
        Ok(())
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.disable();
        unsafe { CloseHandle(self.handle) };
    }
}
