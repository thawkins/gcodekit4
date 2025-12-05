//! Platform-specific utilities
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};

/// Wrapper for Win32 RawWindowHandle
#[cfg(target_os = "windows")]
pub struct Win32ParentHandle(pub std::num::NonZeroIsize);

#[cfg(target_os = "windows")]
impl HasWindowHandle for Win32ParentHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = Win32WindowHandle::new(self.0);
        let raw = RawWindowHandle::Win32(handle);
        unsafe { Ok(WindowHandle::borrow_raw(raw)) }
    }
}

#[cfg(target_os = "windows")]
impl HasDisplayHandle for Win32ParentHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let handle = WindowsDisplayHandle::new();
        let raw = RawDisplayHandle::Windows(handle);
        unsafe { Ok(DisplayHandle::borrow_raw(raw)) }
    }
}

/// Initialize the window (maximize and focus)
pub fn initialize_window(_window: &slint::Window) {
    #[cfg(target_os = "windows")]
    {
        // Ensure HasWindowHandle is in scope for slint::Window
        let handle = _window.window_handle();
        if let Ok(raw_handle) = handle.window_handle() {
            if let RawWindowHandle::Win32(win32_handle) = raw_handle.as_raw() {
                let hwnd = win32_handle.hwnd.get();
                unsafe {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{
                        ShowWindow, SetForegroundWindow, SW_MAXIMIZE
                    };
                    
                    // Maximize window on startup
                    ShowWindow(hwnd, SW_MAXIMIZE);
                    SetForegroundWindow(hwnd);
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For other platforms, rely on Slint's preferred size or maximize if needed
        // window.set_maximized(true); 
    }
}

/// Helper to call FileDialog::pick_file with the current foreground window as parent on Windows
pub fn pick_file_with_parent(dialog: rfd::FileDialog, _window: &slint::Window) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // Extract the HWND from the Slint window and use our wrapper
        // This avoids issues where Slint's handle implementation might cause full-screen dialogs
        if let Ok(handle) = _window.window_handle().window_handle() {
            if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                if let Some(hwnd) = std::num::NonZeroIsize::new(win32_handle.hwnd.get()) {
                    let parent = Win32ParentHandle(hwnd);
                    return dialog.set_parent(&parent).pick_file();
                }
            }
        }
        // Fallback if handle extraction fails
        dialog.pick_file()
    }
    #[cfg(not(target_os = "windows"))]
    dialog.pick_file()
}

/// Helper to call FileDialog::save_file with the current foreground window as parent on Windows
pub fn save_file_with_parent(dialog: rfd::FileDialog, _window: &slint::Window) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(handle) = _window.window_handle().window_handle() {
            if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                if let Some(hwnd) = std::num::NonZeroIsize::new(win32_handle.hwnd.get()) {
                    let parent = Win32ParentHandle(hwnd);
                    return dialog.set_parent(&parent).save_file();
                }
            }
        }
        dialog.save_file()
    }
    #[cfg(not(target_os = "windows"))]
    dialog.save_file()
}

/// Helper for folder picking with the foreground window as parent on Windows
pub fn pick_folder_with_parent(dialog: rfd::FileDialog, _window: &slint::Window) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(handle) = _window.window_handle().window_handle() {
            if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                if let Some(hwnd) = std::num::NonZeroIsize::new(win32_handle.hwnd.get()) {
                    let parent = Win32ParentHandle(hwnd);
                    return dialog.set_parent(&parent).pick_folder();
                }
            }
        }
        dialog.pick_folder()
    }
    #[cfg(not(target_os = "windows"))]
    dialog.pick_folder()
}
