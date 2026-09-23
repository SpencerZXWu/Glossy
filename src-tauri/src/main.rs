// Glossy lives in the notification area, so double clicking it must never open a
// console window, not even in a debug build. Diagnostics still go to stderr,
// which `platform::console::attach_parent` reconnects to the terminal that started the
// process, if there is one.
#![windows_subsystem = "windows"]

fn main() {
    glossy_lib::platform::console::attach_parent();
    glossy_lib::run()
}
