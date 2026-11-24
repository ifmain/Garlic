use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use garlic_state::StateError;
use std::{ptr, slice};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    },
};

fn transform(input: &[u8], encrypt: bool) -> Result<Vec<u8>, StateError> {
    let source = CRYPT_INTEGER_BLOB {
        cbData: input.len().try_into()?,
        pbData: input.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    // DPAPI reads source for the duration of the call and allocates its result
    // with LocalAlloc. The current Windows user owns this protection scope.
    unsafe {
        let ok = if encrypt {
            CryptProtectData(
                &source,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &source,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        if ok == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if output.pbData.is_null() {
            return Err("DPAPI returned an empty result".into());
        }
        let result = slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData.cast());
        Ok(result)
    }
}

pub fn encode(token: &str) -> Result<String, StateError> {
    Ok(format!(
        "dpapi:{}",
        URL_SAFE_NO_PAD.encode(transform(token.as_bytes(), true)?)
    ))
}

pub fn decode(token: &str) -> Result<String, StateError> {
    match token.strip_prefix("dpapi:") {
        Some(encoded) => Ok(String::from_utf8(transform(
            &URL_SAFE_NO_PAD.decode(encoded)?,
            false,
        )?)?),
        None => Ok(token.to_owned()),
    }
}
