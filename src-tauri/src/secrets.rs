//! DPAPI protection for the credentials in `settings.json`.
//!
//! Windows can encrypt a small blob with a key derived from the current user's
//! login, so the API keys in the settings file are only readable by this user on
//! this computer - copying the file somewhere else does not disclose them.
//! Protected values carry a `dpapi:` prefix and hold base64; a value without the
//! prefix is plain text written by an older version and is rewritten protected
//! the next time the settings are saved.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

/// Prefix of a value DPAPI produced, which tells it apart from plain text.
const PREFIX: &str = "dpapi:";

/// What someone reading the raw blob in another login sees instead of the key.
const DESCRIPTION: &str = "Glossy credential";

/// Whether `stored` was written by [`protect`].
pub fn is_protected(stored: &str) -> bool {
    stored.starts_with(PREFIX)
}

/// Encrypts `plain` for the current user.
pub fn protect(plain: &str) -> Result<String, String> {
    let description: Vec<u16> = DESCRIPTION.encode_utf16().chain([0]).collect();
    let input = as_blob(plain.as_bytes());
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(
            &input,
            PCWSTR(description.as_ptr()),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    }
    .map_err(|error| format!("CryptProtectData failed: {error}"))?;

    Ok(format!("{PREFIX}{}", STANDARD.encode(take_output(output))))
}

/// Decrypts a value written by [`protect`].
///
/// `None` when the blob cannot be unlocked: DPAPI keys belong to a login, so a
/// settings file copied from another account or another computer has to be
/// filled in again.
pub fn reveal(stored: &str) -> Option<String> {
    let encoded = stored.strip_prefix(PREFIX)?;
    let blob = STANDARD.decode(encoded.trim()).ok()?;
    if blob.is_empty() {
        return None;
    }

    let input = as_blob(&blob);
    let mut output = CRYPT_INTEGER_BLOB::default();
    let mut label = PWSTR::null();
    let result = unsafe {
        CryptUnprotectData(
            &input,
            Some(&mut label),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if !label.is_null() {
        // The description DPAPI hands back was allocated for the caller.
        unsafe { LocalFree(Some(HLOCAL(label.0.cast()))) };
    }
    result.ok()?;

    String::from_utf8(take_output(output)).ok()
}

fn as_blob(bytes: &[u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    }
}

/// Copies what DPAPI allocated into a `Vec` and gives the buffer back.
fn take_output(output: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    if output.pbData.is_null() {
        return Vec::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) };
    let copied = bytes.to_vec();
    unsafe { LocalFree(Some(HLOCAL(output.pbData.cast()))) };
    copied
}
