//! What the desktop looks like: the cursor, the monitors, the window under a
//! point, the program in front and the user's locale, plus the window styles
//! that keep the popup out of the way.

use std::ffi::c_void;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Duration;

use crate::platform::ScreenRect;

use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, POINT};
use windows::Win32::Globalization::GetUserDefaultLocaleName;
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_COLOR_DEFAULT,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GetAncestor, GetClassNameW, GetCursorPos, GetForegroundWindow,
    GetGUIThreadInfo, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, SetForegroundWindow,
    SetWindowLongPtrW, ShowWindow, WindowFromPoint, GA_PARENT, GA_ROOT, GA_ROOTOWNER,
    GUITHREADINFO, GWL_EXSTYLE, SW_RESTORE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
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

/// What Windows calls this installation of Windows, from the registry.
///
/// It is the one identifier that survives Glossy being uninstalled and
/// installed again, which is what the daily allowance is counted by: a value
/// drawn at random would be drawn again on every install, and the allowance
/// would start over with it. It is per installation of Windows rather than per
/// account, so it is read together with the account name by the caller that
/// wants one allowance per user.
///
/// The value is not personal and not secret — it identifies the Windows
/// installation, not its owner — and nothing reads it here but the callers that
/// hash it before it leaves the machine.
pub fn machine_id() -> Option<String> {
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
    };

    let path: Vec<u16> = "SOFTWARE\\Microsoft\\Cryptography\0"
        .encode_utf16()
        .collect();
    let name: Vec<u16> = "MachineGuid\0".encode_utf16().collect();
    let mut key = HKEY::default();
    if unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            windows::core::PCWSTR(path.as_ptr()),
            None,
            KEY_READ,
            &mut key,
        )
    }
    .is_err()
    {
        return None;
    }

    let mut kind = REG_SZ;
    let mut bytes = [0u8; 128];
    let mut size = bytes.len() as u32;
    let read = unsafe {
        RegQueryValueExW(
            key,
            windows::core::PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            Some(bytes.as_mut_ptr()),
            Some(&mut size),
        )
    };
    let _ = unsafe { RegCloseKey(key) };
    if read.is_err() || size < 2 {
        return None;
    }

    // The value is a string, which the registry hands back as UTF-16 with its
    // closing null inside the byte count.
    let units: Vec<u16> = bytes[..size as usize - 2]
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let value = String::from_utf16_lossy(&units);
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
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

/// Whether `handle` is the window that currently holds the keyboard.
pub fn is_foreground(handle: Handle) -> bool {
    if handle.is_null() {
        return false;
    }
    unsafe { GetForegroundWindow() == handle.to_hwnd() }
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

/// Process id of the window under `(x, y)`, `None` when there is no window.
///
/// The list of a `<select>` is drawn by the browser process of the webview, a
/// program of ours that owns no window of ours, so the process is what tells it
/// apart from the program behind us.
pub fn process_id_under_point(x: i32, y: i32) -> Option<u32> {
    unsafe {
        let window = WindowFromPoint(POINT { x, y });
        if window.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(window, Some(&mut pid));
        if pid == 0 {
            None
        } else {
            Some(pid)
        }
    }
}

/// Process id of the first child of `parent` that is not our own process.
///
/// A webview window is drawn by a separate browser process, which is a child of
/// ours and the process that opens the list of a `<select>`.
pub fn child_process_id(parent: Handle) -> Option<u32> {
    if parent.is_null() {
        return None;
    }
    let mut found = 0u32;
    unsafe {
        let _ = EnumChildWindows(
            Some(parent.to_hwnd()),
            Some(collect_child_process),
            LPARAM(&mut found as *mut u32 as isize),
        );
    }
    if found == 0 {
        None
    } else {
        Some(found)
    }
}

unsafe extern "system" fn collect_child_process(child: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let slot = &mut *(lparam.0 as *mut u32);
        let mut pid = 0u32;
        GetWindowThreadProcessId(child, Some(&mut pid));
        if pid != 0 && pid != GetCurrentProcessId() {
            *slot = pid;
            return BOOL(0);
        }
        BOOL(1)
    }
}

/// True when the window under `(x, y)` belongs to `root`.
///
/// The desktop draws parts of a window somewhere else: the list of a `<select>`
/// is a popup window of its own, owned by a window of the webview that opened
/// it, and a long list reaches past the edges of the window it belongs to.
/// Walking the owner chain is what tells a click there apart from a click on
/// the program behind us.
pub fn owns_point(root: Handle, x: i32, y: i32) -> bool {
    if root.is_null() {
        return false;
    }
    unsafe {
        let window = WindowFromPoint(POINT { x, y });
        if window.0.is_null() {
            return false;
        }
        root_owner_of(window).0 == root.to_hwnd().0
    }
}

/// Top level window that answers for `window`.
///
/// The owner of a popup hangs off the top level window, the one carrying the
/// owner is only reached by walking up the parent chain first: the point asks
/// for the deepest window under it, a render widget several levels down.
unsafe fn root_owner_of(window: HWND) -> HWND {
    unsafe {
        let top = GetAncestor(window, GA_ROOT);
        let top = if top.0.is_null() { window } else { top };
        let owner = GetAncestor(top, GA_ROOTOWNER);
        if owner.0.is_null() {
            top
        } else {
            owner
        }
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

/// The window the selection being translated was read from.
static SELECTION_WINDOW: AtomicIsize = AtomicIsize::new(0);

/// Remember which window the selection in front of the user belongs to.
///
/// The popup is shown without taking the focus, but a click on the badge or on
/// the card still hands it the focus, and a paste goes to whoever is in front -
/// so who owned the selection has to be written down while it is still true.
pub fn note_focus_owner() {
    let window = unsafe { GetForegroundWindow() };
    SELECTION_WINDOW.store(window.0 as isize, Ordering::Relaxed);
}

/// Give the focus back to the window remembered by [`note_focus_owner`].
///
/// A hand-over of the keyboard that is undone when it goes out of scope.
///
/// The keys have to be sent while the two input queues are still joined:
/// joining them is what lets a window that is not in front be given the
/// keyboard, and leaving them joined is what keeps the keys going there.
pub struct FocusHandover {
    front_thread: u32,
    target_thread: u32,
    joined: bool,
}

impl Drop for FocusHandover {
    fn drop(&mut self) {
        if self.joined {
            unsafe {
                let _ = AttachThreadInput(self.front_thread, self.target_thread, false);
            }
        }
    }
}

/// How long a hand-over waits for the keyboard to move, and how often it looks.
const FOCUS_TRIES: u32 = 25;
const FOCUS_STEP: Duration = Duration::from_millis(10);

/// Points the keyboard back at the window the selection was read from, and
/// keeps it there for as long as the returned hand-over lives.
///
/// A no-op when that window is already in front - which is the case when the
/// translation was asked for with a hotkey, since no click ever followed it.
pub fn restore_focus_owner() -> Option<FocusHandover> {
    let remembered = SELECTION_WINDOW.swap(0, Ordering::Relaxed);
    if remembered == 0 {
        return None;
    }
    let target = Handle(remembered).to_hwnd();
    unsafe {
        let front = GetForegroundWindow();
        if front == target || !IsWindow(Some(target)).as_bool() {
            return None;
        }
        if IsIconic(target).as_bool() {
            let _ = ShowWindow(target, SW_RESTORE);
        }
        // Read from the target's own thread, before the queues are joined: once
        // they are, both answer with the one shared focus.
        let inner = focus_window(target);
        let front_thread = GetWindowThreadProcessId(front, None);
        let target_thread = GetWindowThreadProcessId(target, None);
        if target_thread == 0 {
            return None;
        }
        // A background process cannot give a window the keyboard; sharing the
        // input queue of the window that is in front is what makes it possible.
        let joined = front_thread != 0
            && front_thread != target_thread
            && AttachThreadInput(front_thread, target_thread, true).as_bool();
        let handover = FocusHandover {
            front_thread,
            target_thread,
            joined,
        };
        // The window is also brought to the front, so the user sees where the
        // translation is about to land.
        let _ = SetForegroundWindow(target);
        if let Some(inner) = inner {
            let _ = SetFocus(Some(inner));
        }
        let mut tries = 0;
        while tries < FOCUS_TRIES && !keyboard_is_target(target) {
            tries += 1;
            std::thread::sleep(FOCUS_STEP);
        }
        Some(handover)
    }
}

/// Whether the keys would reach `target`: the window carries the keyboard, and
/// either it is what the two joined queues call focused or it is in front.
fn keyboard_is_target(target: HWND) -> bool {
    unsafe {
        let inner = match focus_window(target) {
            Some(inner) => inner,
            None => return false,
        };
        GetForegroundWindow() == target || inner == target || GetAncestor(inner, GA_ROOT) == target
    }
}

fn focus_window(target: HWND) -> Option<HWND> {
    unsafe {
        let thread = GetWindowThreadProcessId(target, None);
        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if GetGUIThreadInfo(thread, &mut info).is_err() || info.hwndFocus.0.is_null() {
            return None;
        }
        Some(info.hwndFocus)
    }
}

/// Keeps the popup from stealing focus from the application the user is reading
/// in, and from showing up in the taskbar or alt-tab list.
pub fn make_non_activating(handle: Handle) {
    set_non_activating(handle, true);
}

/// Hands the keyboard to the window, undoing [`make_non_activating`], and
/// reports whether the window ended up in front.
///
/// A field inside the popup is typed into, and typing needs the keyboard. Only
/// the user's own action asks for this, and the next placement puts the window
/// back out of the way, so the front is held no longer than the edit lasts.
pub fn make_activating(handle: Handle) -> bool {
    if handle.is_null() || !set_non_activating(handle, false) {
        return false;
    }
    let hwnd = handle.to_hwnd();
    unsafe {
        if GetForegroundWindow() == hwnd {
            return true;
        }
        // A background process is not allowed to put a window in front on its
        // own. Sharing the input queue of the window that holds the front is
        // what makes the move possible - the same trick as
        // [`restore_focus_owner`], only in the other direction.
        let front = GetForegroundWindow();
        let front_thread = GetWindowThreadProcessId(front, None);
        let window_thread = GetWindowThreadProcessId(hwnd, None);
        let joined = front_thread != 0
            && window_thread != 0
            && front_thread != window_thread
            && AttachThreadInput(front_thread, window_thread, true).as_bool();
        let _ = SetForegroundWindow(hwnd);
        let mut tries = 0;
        while tries < FOCUS_TRIES && GetForegroundWindow() != hwnd {
            tries += 1;
            std::thread::sleep(FOCUS_STEP);
        }
        if joined {
            let _ = AttachThreadInput(front_thread, window_thread, false);
        }
        GetForegroundWindow() == hwnd
    }
}

/// Sets or clears the styles that keep a window out of the foreground; reports
/// whether the window carries what was asked for afterwards.
fn set_non_activating(handle: Handle, wanted: bool) -> bool {
    if handle.is_null() {
        return false;
    }
    unsafe {
        let hwnd = handle.to_hwnd();
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let flag = WS_EX_NOACTIVATE.0 as isize;
        let updated = if wanted {
            current | flag | (WS_EX_TOOLWINDOW.0 as isize)
        } else {
            current & !flag
        };
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, updated);
        GetWindowLongPtrW(hwnd, GWL_EXSTYLE) == updated
    }
}

/// Shows a folder in File Explorer, with the file it was asked about selected.
///
/// Used for the log, so a user who is asked to hand it over does not have to
/// find the folder themselves. Explorer is told to select the file rather than
/// open it: that is what the user needs to see, and opening a text file in
/// whatever program is registered for `.log` would be a step too far.
pub fn open_folder(path: &Path) -> Result<(), String> {
    let argument = format!("/select,\"{}\"", path.display());
    Command::new("explorer")
        .arg(argument)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("File Explorer could not be opened: {error}"))
}

/// Paints the title bar of a window, or gives it back to Windows.
///
/// The caption is drawn by the window manager rather than by the page, so a
/// palette that states its own window colour has to reach it through DWM or the
/// window keeps a bar the colour of the scheme it is not in. `None` clears the
/// override (`DWMWA_COLOR_DEFAULT`), which is what the default palette wants: it
/// is the WinUI scheme, and its own caption is what belongs there.
///
/// Needs Windows 11 build 22000 or newer; an older build refuses the attribute
/// and this reports false, leaving the caption as it was.
pub fn set_caption_color(handle: Handle, colour: Option<(u8, u8, u8)>) -> bool {
    if handle.is_null() {
        return false;
    }
    let value: u32 = match colour {
        Some((red, green, blue)) => (blue as u32) << 16 | (green as u32) << 8 | red as u32,
        None => DWMWA_COLOR_DEFAULT,
    };
    let hwnd = handle.to_hwnd();
    let pointer = &value as *const u32 as *const c_void;
    let size = std::mem::size_of::<u32>() as u32;
    let caption = unsafe { DwmSetWindowAttribute(hwnd, DWMWA_CAPTION_COLOR, pointer, size) };
    // The frame around the window follows the caption, or a palette leaves a
    // one-pixel line of the previous scheme around the whole window.
    let border = unsafe { DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, pointer, size) };
    caption.is_ok() && border.is_ok()
}

/// Opens a web address in whatever program the user reads the web with.
///
/// `FileProtocolHandler` is the shell's own "open this address" entry point, so
/// the address goes to the default browser and to nothing else — a plain
/// `Command::new(url)` would look for a program of that name first.
pub fn open_url(url: &str) -> Result<(), String> {
    Command::new("rundll32")
        .arg("url.dll,FileProtocolHandler")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("The browser could not be opened: {error}"))
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

    #[test]
    fn a_window_without_a_handle_owns_nothing() {
        assert!(!owns_point(Handle::default(), 0, 0));
        assert!(child_process_id(Handle::default()).is_none());
    }
}
