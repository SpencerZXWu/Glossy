//! Thin wrappers around the Win32 APIs Glossy needs (locale, cursor, monitor and
//! window geometry, non activating window styles).

use std::ffi::c_void;

use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, POINT};
use windows::Win32::Globalization::GetUserDefaultLocaleName;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetAncestor, GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowLongPtrW,
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    SetWindowLongPtrW, WindowFromPoint, GA_PARENT, GA_ROOT, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW,
};

/// Native handle of a window, kept as a plain integer so callers never have to
/// depend on the exact `windows` crate version used by Tauri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Handle(pub isize);

impl Handle {
    pub fn to_hwnd(self) -> HWND {
        HWND(self.0 as *mut c_void)
    }

    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// Region of the desktop: `(left, top, right, bottom)` in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ScreenRect {
    /// True when the point falls inside the rectangle, grown by `pad` pixels.
    pub fn contains_padded(self, x: i32, y: i32, pad: i32) -> bool {
        x >= self.left - pad && x < self.right + pad && y >= self.top - pad && y < self.bottom + pad
    }
}

/// BCP-47 style locale of the current user, e.g. `en-US` or `zh-CN`.
pub fn user_locale() -> String {
    const LOCALE_NAME_MAX_LENGTH: usize = 85;
    let mut buffer = [0u16; LOCALE_NAME_MAX_LENGTH];
    let length = unsafe { GetUserDefaultLocaleName(&mut buffer) };
    if length > 1 {
        String::from_utf16_lossy(&buffer[..length as usize - 1])
    } else {
        "en-US".to_string()
    }
}

/// Cursor position in physical screen pixels.
pub fn cursor_pos() -> (i32, i32) {
    let mut point = POINT::default();
    if unsafe { GetCursorPos(&mut point) }.is_err() {
        return (0, 0);
    }
    (point.x, point.y)
}

/// True when one of our own windows currently owns the foreground.
pub fn foreground_is_self() -> bool {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(foreground, Some(&mut pid));
        pid == GetCurrentProcessId()
    }
}

/// File name of the executable that owns the foreground window, e.g. `chrome.exe`.
pub fn foreground_process_name() -> Option<String> {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() {
            return None;
        }
        process_name_of_window(foreground)
    }
}

/// File name of the executable that owns the window under `(x, y)`, e.g.
/// `chrome.exe`. Used by the "click the program to ignore" picker.
pub fn process_under_point(x: i32, y: i32) -> Option<String> {
    unsafe {
        let window = WindowFromPoint(POINT { x, y });
        if window.0.is_null() {
            return None;
        }
        // Child windows belong to the top level window the user sees.
        let root = GetAncestor(window, GA_ROOT);
        process_name_of_window(if root.0.is_null() { window } else { root })
    }
}

/// Programs that own a visible window right now, as `(process, window title)`.
///
/// One entry per program: the first window with a title stands in for it.
pub fn visible_apps() -> Vec<(String, String)> {
    let mut collected: Vec<(String, String)> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect_app),
            LPARAM(&mut collected as *mut Vec<(String, String)> as isize),
        );
    }
    collected.sort_by(|a, b| a.0.cmp(&b.0));
    collected
}

unsafe extern "system" fn collect_app(window: HWND, lparam: LPARAM) -> BOOL {
    let collected = unsafe { &mut *(lparam.0 as *mut Vec<(String, String)>) };
    if unsafe { IsWindowVisible(window) }.as_bool() {
        let title = unsafe { window_title(window) };
        if !title.is_empty() {
            if let Some(name) = unsafe { process_name_of_window(window) } {
                if !collected.iter().any(|(known, _)| known == &name) {
                    collected.push((name, title));
                }
            }
        }
    }
    // Keep enumerating.
    BOOL(1)
}

unsafe fn window_title(window: HWND) -> String {
    unsafe {
        let length = GetWindowTextLengthW(window);
        if length <= 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; length as usize + 1];
        let written = GetWindowTextW(window, &mut buffer);
        if written <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buffer[..written as usize])
            .trim()
            .to_string()
    }
}

/// File name of the executable owning `window`, or `None` for our own windows.
unsafe fn process_name_of_window(window: HWND) -> Option<String> {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(window, Some(&mut pid));
        if pid == 0 || pid == GetCurrentProcessId() {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 512];
        let mut length = buffer.len() as u32;
        let named = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        );
        let _ = CloseHandle(process);
        if named.is_err() {
            return None;
        }

        let path = String::from_utf16_lossy(&buffer[..length as usize]);
        path.rsplit(['\\', '/'])
            .next()
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string())
    }
}

/// Window classes of the shell surfaces: the desktop, the taskbar, the system
/// pop-ups. Nothing there can be selected, but a click still reaches the mouse
/// hook.
const SHELL_CLASSES: &[&str] = &[
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "NotifyIconOverflowWindow",
    "XamlExplorerHostIslandWindow",
    "Windows.UI.Core.CoreWindow",
    "ForegroundStaging",
];

/// Programs that only draw the start menu, the search box and the lock screen.
/// Explorer itself is absent: its file windows do show selectable text, so only
/// the classes above rule it out.
const SHELL_PROCESSES: &[&str] = &[
    "StartMenuExperienceHost.exe",
    "SearchHost.exe",
    "ShellExperienceHost.exe",
    "TextInputHost.exe",
    "LockApp.exe",
    "sihost.exe",
];

/// True when `(x, y)` lies on the desktop, the taskbar or another surface of the
/// shell.
///
/// Such a click selects nothing, yet the Ctrl+C Glossy sends afterwards still
/// reaches the program in front, which may answer it by copying the clicked item
/// as text - the popup that used to appear out of nowhere.
///
/// The whole window chain is inspected, not just the top level window: a desktop
/// icon is a list view nested inside `Progman`/`WorkerW`, and the window under
/// the cursor reports its own class, which is a plain list.
pub fn shell_surface_at(x: i32, y: i32) -> bool {
    unsafe {
        let window = WindowFromPoint(POINT { x, y });
        if window.0.is_null() {
            return true;
        }
        chain_reaches_shell(window, &window_chain(window))
    }
}

/// True when the program that would receive the copy shortcut is a shell
/// surface, so the text it would produce is a shortcut name rather than a
/// selection.
///
/// A double click on a desktop icon can hand the foreground to the desktop
/// itself, which then answers Ctrl+C with the name of the icon.
pub fn copy_target_is_shell() -> bool {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() {
            return true;
        }
        chain_reaches_shell(foreground, &window_chain(foreground))
    }
}

/// Classes of `window` and every ancestor, innermost first.
unsafe fn window_chain(window: HWND) -> Vec<String> {
    let mut classes = Vec::new();
    let mut current = window;
    // The desktop window ends every chain; the bound only guards a cycle.
    for _ in 0..16 {
        classes.push(window_class(current));
        let parent = GetAncestor(current, GA_PARENT);
        if parent.0.is_null() || parent.0 == current.0 {
            break;
        }
        current = parent;
    }
    classes
}

/// Splits the decision from the window calls so it can be tested.
fn chain_reaches_shell(window: HWND, classes: &[String]) -> bool {
    if classes.iter().any(|class| {
        SHELL_CLASSES
            .iter()
            .any(|shell| class.eq_ignore_ascii_case(shell))
    }) {
        return true;
    }
    // The process check only applies to the outer window, the one Explorer owns.
    let outer = unsafe {
        let root = GetAncestor(window, GA_ROOT);
        if root.0.is_null() {
            window
        } else {
            root
        }
    };
    unsafe {
        process_name_of_window(outer).is_some_and(|name| {
            SHELL_PROCESSES
                .iter()
                .any(|shell| shell.eq_ignore_ascii_case(&name))
        })
    }
}

unsafe fn window_class(window: HWND) -> String {
    unsafe {
        let mut buffer = [0u16; 256];
        let length = GetClassNameW(window, &mut buffer);
        if length <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buffer[..length as usize])
    }
}

/// Work area (desktop minus taskbar) of the monitor containing the point.
pub fn work_area_for_point(x: i32, y: i32) -> Option<ScreenRect> {
    unsafe {
        let monitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        if monitor.0.is_null() {
            return None;
        }
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            Some(ScreenRect {
                left: info.rcWork.left,
                top: info.rcWork.top,
                right: info.rcWork.right,
                bottom: info.rcWork.bottom,
            })
        } else {
            None
        }
    }
}

/// Keeps the popup from stealing focus from the application the user is reading
/// in, and from showing up in the taskbar or alt-tab list.
pub fn make_non_activating(handle: Handle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        let hwnd = handle.to_hwnd();
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let updated = current | (WS_EX_NOACTIVATE.0 as isize) | (WS_EX_TOOLWINDOW.0 as isize);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, updated);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::HWND;

    fn chain(classes: &[&str]) -> Vec<String> {
        classes.iter().map(|class| class.to_string()).collect()
    }

    #[test]
    fn a_desktop_icon_is_part_of_the_shell() {
        // Windows 11 nests the icon list inside WorkerW, Windows 10 inside
        // Progman; the window under the cursor is only the list itself.
        assert!(chain_reaches_shell(
            HWND::default(),
            &chain(&["SysListView32", "SHELLDLL_DefView", "WorkerW", "#32769"])
        ));
        assert!(chain_reaches_shell(
            HWND::default(),
            &chain(&["SysListView32", "SHELLDLL_DefView", "Progman", "#32769"])
        ));
    }

    #[test]
    fn the_taskbar_and_its_children_are_part_of_the_shell() {
        assert!(chain_reaches_shell(
            HWND::default(),
            &chain(&["MSTaskListWClass", "Shell_TrayWnd", "#32769"])
        ));
        assert!(chain_reaches_shell(
            HWND::default(),
            &chain(&["Windows.UI.Core.CoreWindow"])
        ));
    }

    #[test]
    fn a_text_field_inside_a_folder_window_is_not() {
        // Folder windows host the same SHELLDLL_DefView class as the desktop, so
        // only the outer window decides.
        assert!(!chain_reaches_shell(
            HWND::default(),
            &chain(&["Edit", "DirectUIHWND", "SHELLDLL_DefView", "CabinetWClass"])
        ));
        assert!(!chain_reaches_shell(
            HWND::default(),
            &chain(&["Edit", "Notepad", "#32769"])
        ));
        assert!(!chain_reaches_shell(HWND::default(), &chain(&["Edit"])));
    }
}
