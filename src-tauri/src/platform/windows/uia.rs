//! The text around the current selection, read through UI Automation.
//!
//! Browsers, Word and most readers expose their text this way; a program that
//! does not (a terminal, a canvas application) answers with nothing, which is
//! why every call returns an `Option` rather than an error.

use windows::core::BSTR;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation8, IUIAutomation, IUIAutomationTextPattern, IUIAutomationTextRange,
    IUIAutomationTextRangeArray, TextUnit, TextUnit_Line, TextUnit_Paragraph, UIA_TextPatternId,
};

/// A unit of text a selection can be expanded to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// The line the selection sits on.
    Line,
    /// The paragraph the selection sits in, which a provider that does not
    /// support paragraph units reports as its whole document.
    Paragraph,
}

impl Unit {
    fn raw(self) -> TextUnit {
        match self {
            Unit::Line => TextUnit_Line,
            Unit::Paragraph => TextUnit_Paragraph,
        }
    }
}

/// The text of the `unit` around the current selection, when it can be read.
pub fn text_of(unit: Unit) -> Option<String> {
    // UI Automation needs COM, and the caller is a worker thread that has not
    // initialised it yet.
    let started = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
    let text = unsafe { read_enclosing(unit) };
    if started {
        unsafe { CoUninitialize() };
    }
    text
}

unsafe fn read_enclosing(unit: Unit) -> Option<String> {
    let automation: IUIAutomation = CoCreateInstance(&CUIAutomation8, None, CLSCTX_ALL).ok()?;
    let element = automation.GetFocusedElement().ok()?;
    let pattern: IUIAutomationTextPattern = element.GetCurrentPatternAs(UIA_TextPatternId).ok()?;
    let ranges: IUIAutomationTextRangeArray = pattern.GetSelection().ok()?;
    let range = ranges.GetElement(0).ok()?;
    let text = expand(&range, unit)?;
    (!text.trim().is_empty()).then_some(text)
}

/// Expands a copy of `range` to the unit around it and returns its text.
unsafe fn expand(range: &IUIAutomationTextRange, unit: Unit) -> Option<String> {
    // The range is expanded in place, so the caller keeps the original.
    let range = range.clone();
    range.ExpandToEnclosingUnit(unit.raw()).ok()?;
    let text: BSTR = range.GetText(-1).ok()?;
    Some(text.to_string())
}
