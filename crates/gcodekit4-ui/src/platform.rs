//! UI crate platform utilities
use raw_window_handle::{HasRawWindowHandle, HasDisplayHandle, RawWindowHandle, RawDisplayHandle, Win32WindowHandle, Win32DisplayHandle};
use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub struct Win32ParentHandle(pub std::num::NonZeroIsize);

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for Win32ParentHandle {
    fn raw_window_handle(&self) -> Result<RawWindowHandle, raw_window_handle::HandleError> {
        let handle = Win32WindowHandle::new(self.0);
        Ok(RawWindowHandle::Win32(handle))
    }
}

unsafe impl HasDisplayHandle for Win32ParentHandle {
    fn raw_display_handle(&self) -> Result<RawDisplayHandle, raw_window_handle::HandleError> {
        // Get module handle for the current process as hinstance
        use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
        let hinst = unsafe { GetModuleHandleW(std::ptr::null()) } as isize;
        if let Some(nz) = std::num::NonZeroIsize::new(hinst) {
            let dh = Win32DisplayHandle::new(nz);
            Ok(RawDisplayHandle::Win32(dh))
        } else {
            Err(raw_window_handle::HandleError::UnsupportedPlatform)
        }
    }
}

#[cfg(target_os = "windows")]
fn get_foreground_hwnd() -> Option<Win32ParentHandle> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let hwnd_val = unsafe { GetForegroundWindow() } as isize;
    std::num::NonZeroIsize::new(hwnd_val).map(Win32ParentHandle)
}

pub fn pick_file_with_parent(dialog: rfd::FileDialog) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
            if let Some(parent) = get_foreground_hwnd() {
            return dialog.set_parent(&parent).pick_file();
        }
    }
    dialog.pick_file()
}

pub fn save_file_with_parent(dialog: rfd::FileDialog) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
            if let Some(parent) = get_foreground_hwnd() {
            return dialog.set_parent(&parent).save_file();
        }
    }
    dialog.save_file()
}

pub fn pick_folder_with_parent(dialog: rfd::FileDialog) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
            if let Some(parent) = get_foreground_hwnd() {
            return dialog.set_parent(&parent).pick_folder();
        }
    }
    dialog.pick_folder()
}
