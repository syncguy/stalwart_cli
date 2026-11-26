// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
//
// Preserve the dirs crate's directory choices, but keep MSVC Windows builds
// compatible with NT 6.1: CoTaskMemFree must import from Ole32.dll, not combase.dll.

#[cfg(not(all(windows, target_env = "msvc")))]
pub use dirs::{cache_dir, config_dir, data_local_dir};

#[cfg(all(windows, target_env = "msvc"))]
pub use windows::{cache_dir, config_dir, data_local_dir};

#[cfg(all(windows, target_env = "msvc"))]
mod windows {
    use std::ffi::{OsString, c_void};
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;
    use std::ptr;
    use std::slice;

    // The documented KNOWNFOLDERID/GUID ABI (16 bytes).
    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    const LOCAL_APP_DATA: Guid = Guid {
        data1: 0xf1b32785,
        data2: 0x6fba,
        data3: 0x4fcf,
        data4: [0x9d, 0x55, 0x7b, 0x8e, 0x7f, 0x15, 0x70, 0x91],
    };

    const ROAMING_APP_DATA: Guid = Guid {
        data1: 0x3eb685db,
        data2: 0x65f9,
        data3: 0x4cf6,
        data4: [0xa0, 0x3a, 0xe3, 0xef, 0x65, 0x72, 0x9f, 0x3d],
    };

    // raw-dylib fixes the actual import DLL instead of relying on newer SDK
    // import-library forwarding. Both APIs are documented for NT 6.1.
    #[link(name = "shell32.dll", kind = "raw-dylib", modifiers = "+verbatim")]
    unsafe extern "system" {
        fn SHGetKnownFolderPath(
            folder_id: *const Guid,
            flags: u32,
            token: *mut c_void,
            path: *mut *mut u16,
        ) -> i32;
    }

    #[link(name = "ole32.dll", kind = "raw-dylib", modifiers = "+verbatim")]
    unsafe extern "system" {
        fn CoTaskMemFree(memory: *mut c_void);
    }

    #[link(name = "kernel32.dll", kind = "raw-dylib", modifiers = "+verbatim")]
    unsafe extern "system" {
        fn lstrlenW(text: *const u16) -> i32;
    }

    struct FolderBuffer(*mut u16);

    impl Drop for FolderBuffer {
        fn drop(&mut self) {
            // SHGetKnownFolderPath requires this allocator on success AND failure.
            // CoTaskMemFree is also safe for null.
            unsafe { CoTaskMemFree(self.0.cast()) };
        }
    }

    fn known_folder(id: &Guid) -> Option<PathBuf> {
        let mut path = FolderBuffer(ptr::null_mut());
        // Use the current user's known folder, respecting Windows redirection.
        let result = unsafe { SHGetKnownFolderPath(id, 0, ptr::null_mut(), &mut path.0) };
        if result != 0 || path.0.is_null() {
            return None;
        }
        // SAFETY: on S_OK the API supplies a valid NUL-terminated UTF-16
        // string. The owning buffer outlives this slice and conversion.
        let len = unsafe { lstrlenW(path.0) };
        if len <= 0 {
            return None;
        }
        let wide = unsafe { slice::from_raw_parts(path.0, len as usize) };
        Some(PathBuf::from(OsString::from_wide(wide)))
    }

    pub fn cache_dir() -> Option<PathBuf> {
        data_local_dir()
    }

    pub fn data_local_dir() -> Option<PathBuf> {
        known_folder(&LOCAL_APP_DATA)
    }

    pub fn config_dir() -> Option<PathBuf> {
        known_folder(&ROAMING_APP_DATA)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn current_user_known_folders() {
            let local = data_local_dir().expect("local app data");
            let config = config_dir().expect("roaming app data");
            assert!(local.is_absolute());
            assert!(config.is_absolute());
            assert_eq!(cache_dir(), Some(local));
        }

        #[test]
        fn invalid_known_folder_returns_none() {
            let invalid = Guid {
                data1: 0,
                data2: 0,
                data3: 0,
                data4: [0; 8],
            };
            assert!(known_folder(&invalid).is_none());
        }
    }
}
