//! The two Windows calls the real system needs (docs/research/windows-paths.md Q5, Q7; ADR 0006).
#![allow(unsafe_code)]

use crate::locate::system::Hive;
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW,
};
use windows_sys::Win32::UI::Shell::{
    FOLDERID_Documents, KF_FLAG_DONT_VERIFY, SHGetKnownFolderPath,
};

/// `s` as a NUL-terminated UTF-16 string.
fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// The Documents known folder, unverified so a dangling OneDrive redirect still comes back.
pub(super) fn documents_folder() -> Option<String> {
    let mut raw: *mut u16 = ptr::null_mut();
    // SAFETY: valid GUID and out-pointer; `raw` is freed below on every path (ADR 0006).
    let hr = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_Documents,
            KF_FLAG_DONT_VERIFY as u32,
            ptr::null_mut(),
            &mut raw,
        )
    };
    let path = if hr >= 0 && !raw.is_null() {
        // SAFETY: on success `raw` is a NUL-terminated UTF-16 string, so every index up to and
        // including the NUL is in bounds; we own it until CoTaskMemFree below.
        let len = (0..).take_while(|&i| unsafe { *raw.add(i) } != 0).count();
        // SAFETY: the `len` units before the NUL were just read above and stay valid until freed.
        let units = unsafe { std::slice::from_raw_parts(raw, len) };
        OsString::from_wide(units).into_string().ok()
    } else {
        None
    };
    // SAFETY: CoTaskMemFree accepts null and the pointer returned above, success or not.
    unsafe { CoTaskMemFree(raw as *const _) };
    path
}

/// A REG_SZ value, or `None` if the key or value is missing or of another type.
pub(super) fn registry_string(hive: Hive, subkey: &str, value: &str) -> Option<String> {
    let root: HKEY = match hive {
        Hive::CurrentUser => HKEY_CURRENT_USER,
        Hive::LocalMachine => HKEY_LOCAL_MACHINE,
    };
    let (k, v) = (wide(subkey), wide(value));
    let mut size = 0u32;
    // SAFETY: NUL-terminated strings; a null data pointer asks for the size only.
    let status = unsafe {
        RegGetValueW(
            root,
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut size,
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let mut buf = vec![0u16; (size as usize).div_ceil(2)];
    // SAFETY: `buf` holds at least `size` bytes, and `size` tells RegGetValueW so.
    let status = unsafe {
        RegGetValueW(
            root,
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    OsString::from_wide(buf.get(..len)?).into_string().ok()
}
