//! Reading and rewriting a Word document (`.docx`).
//!
//! A `.docx` is a zip of XML parts, and the text of the document is spread over
//! the runs (`w:r`) a paragraph (`w:p`) is made of: one run per change of
//! formatting, so a sentence with a single bold word in it is already three of
//! them. Glossy reads each paragraph as a whole — nothing is translated in
//! fragments — and writes the translation back into the first run of the
//! paragraph, the run that carries the formatting of the sentence, emptying the
//! runs that only held part of the source. Every other part of the package,
//! styles, tables, pictures, headers and all, is copied through byte for byte.

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, Write};
use std::ops::Range;

use quick_xml::events::Event;
use quick_xml::Reader;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// A `.docx` package, kept as it came in so a rewritten copy can be built from
/// it once the translations are known.
pub struct Document {
    bytes: Vec<u8>,
}

impl Document {
    /// Reads a package. The text is taken out here as well, so a file that is
    /// not really a Word document is refused before the page offers to
    /// translate it.
    pub fn parse(bytes: Vec<u8>) -> Result<Self, String> {
        let document = Self { bytes };
        document.parts()?;
        Ok(document)
    }

    /// The text of every paragraph of the document — body, headers, footers and
    /// notes — in the order the parts and their paragraphs are read.
    pub fn paragraphs(&self) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for part in self.parts()? {
            for paragraph in part.paragraphs {
                out.push(paragraph.text());
            }
        }
        Ok(out)
    }

    /// A copy of the package in which the paragraphs named by `translations` —
    /// an index into [`paragraphs`](Self::paragraphs) and the translation of it
    /// — read the translation instead of the source.
    pub fn rewrite(&self, translations: &HashMap<usize, String>) -> Result<Vec<u8>, String> {
        let mut rewritten: HashMap<String, String> = HashMap::new();
        let mut index = 0usize;
        for part in self.parts()? {
            let mut edits: Vec<(Range<usize>, String)> = Vec::new();
            for paragraph in &part.paragraphs {
                if let Some(translation) = translations.get(&index) {
                    paragraph.edit(translation, &mut edits);
                }
                index += 1;
            }
            if edits.is_empty() {
                continue;
            }
            // Edits are applied from the end so the offsets of the ones before
            // them stay what they were when they were measured.
            edits.sort_by_key(|edit| std::cmp::Reverse(edit.0.start));
            let mut xml = part.xml;
            for (range, text) in edits {
                xml.replace_range(range, &text);
            }
            rewritten.insert(part.name, xml);
        }
        package(&self.bytes, &rewritten)
    }

    /// The XML parts that hold text a reader sees, with the paragraphs they
    /// hold.
    fn parts(&self) -> Result<Vec<TextPart>, String> {
        let mut archive = ZipArchive::new(Cursor::new(&self.bytes[..]))
            .map_err(|error| format!("it is not a zip package: {error}"))?;
        let mut out = Vec::new();
        for index in 0..archive.len() {
            let mut file = archive
                .by_index(index)
                .map_err(|error| format!("a part of the package cannot be opened: {error}"))?;
            if file.is_dir() || !holds_text(file.name()) {
                continue;
            }
            let name = file.name().to_string();
            let mut xml = String::new();
            file.read_to_string(&mut xml)
                .map_err(|_| format!("`{name}` is not text Glossy can read."))?;
            drop(file);
            let mut paragraphs = read_paragraphs(&xml)?;
            paragraphs.retain(|paragraph| !paragraph.runs.is_empty());
            out.push(TextPart {
                name,
                xml,
                paragraphs,
            });
        }
        Ok(out)
    }
}

/// Whether a part of the package holds text of the document. `word/styles.xml`
/// holds the definition of the styles, which is not text of the document, and
/// the same goes for every other part.
fn holds_text(name: &str) -> bool {
    if name == "word/document.xml" {
        return true;
    }
    if name == "word/footnotes.xml" || name == "word/endnotes.xml" {
        return true;
    }
    name.ends_with(".xml") && (name.starts_with("word/header") || name.starts_with("word/footer"))
}

/// One XML part of the package.
struct TextPart {
    name: String,
    xml: String,
    paragraphs: Vec<Paragraph>,
}

/// A paragraph and the runs it is written with, in reading order.
#[derive(Default)]
struct Paragraph {
    runs: Vec<Run>,
}

impl Paragraph {
    /// The paragraph as a reader sees it, which is what is sent to the
    /// translator.
    fn text(&self) -> String {
        let mut out = String::new();
        for run in &self.runs {
            // A line break or a tab between two runs separates two words, so a
            // space takes its place here. The break itself is left where it is.
            if run.after_break && !out.is_empty() && !out.ends_with(char::is_whitespace) {
                out.push(' ');
            }
            out.push_str(&run.text);
        }
        out
    }

    /// Where the translation of this paragraph goes: the first run reads it,
    /// the runs that held the rest of the source are emptied.
    fn edit(&self, translation: &str, edits: &mut Vec<(Range<usize>, String)>) {
        let Some(first) = self.runs.first() else {
            return;
        };
        let escaped = quick_xml::escape::escape(translation).into_owned();
        if first.empty_element || (needs_preserved_space(translation) && !first.preserves_space()) {
            // The element has to be built again: either there was no pair of
            // tags to put the text between, or the text starts or ends with a
            // space and the attribute that keeps it has to be added.
            if escaped.is_empty() {
                edits.push((first.tag.clone(), String::new()));
            } else {
                edits.push((first.tag.clone(), first.element(&escaped)));
            }
        } else {
            edits.push((first.inner.clone(), escaped));
        }
        for run in &self.runs[1..] {
            edits.push((run.tag.clone(), String::new()));
        }
    }
}

/// One `w:t` element: where it sits in the part, and the text it holds.
struct Run {
    /// The whole element, `<w:t>…</w:t>` or `<w:t/>`.
    tag: Range<usize>,
    /// The text between the tags, empty for `<w:t/>`.
    inner: Range<usize>,
    /// The opening tag, as it was written, so its attributes are kept.
    open: String,
    /// The name of the element, which is `w:t`.
    name: String,
    empty_element: bool,
    /// Whether a break sits between this run and the one before it.
    after_break: bool,
    text: String,
}

impl Run {
    fn preserves_space(&self) -> bool {
        self.open.contains("xml:space")
    }

    /// The element with `escaped` between its tags, keeping the attributes it
    /// was written with.
    fn element(&self, escaped: &str) -> String {
        let open = if self.preserves_space() {
            self.open.clone()
        } else {
            with_preserved_space(&self.open)
        };
        format!("{}>{escaped}</{}>", without_end(&open), self.name)
    }
}

/// The opening tag without its `>` or `/>`, so more can be written after it.
fn without_end(open: &str) -> &str {
    open.trim_end_matches("/>").trim_end_matches('>')
}

/// The opening tag with `xml:space="preserve"`, which is what keeps the spaces
/// at the ends of a translation from being dropped.
fn with_preserved_space(open: &str) -> String {
    let head = without_end(open);
    format!("{head} xml:space=\"preserve\"{}", &open[head.len()..])
}

/// Whether the text starts or ends with a space that has to survive.
fn needs_preserved_space(text: &str) -> bool {
    text.starts_with(char::is_whitespace) || text.ends_with(char::is_whitespace)
}

/// The paragraphs of one XML part, in the order a reader meets them, and the
/// runs of each of them.
///
/// Runs are attached to the paragraph they are in, which is the innermost open
/// one: a text box inside a paragraph holds paragraphs of its own.
fn read_paragraphs(xml: &str) -> Result<Vec<Paragraph>, String> {
    let mut reader = Reader::from_str(xml);
    let config = reader.config_mut();
    // The text of a run is read from the bytes of the part itself, so nothing
    // has to be decoded while reading, but the events still have to say where
    // everything is.
    config.trim_text_start = false;
    config.trim_text_end = false;
    config.expand_empty_elements = false;

    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    let mut run: Option<Run> = None;
    let mut after_break = false;
    loop {
        let event = reader
            .read_event()
            .map_err(|error| format!("the XML of the document is damaged: {error}"))?;
        if matches!(event, Event::Eof) {
            break;
        }
        let position = reader.buffer_position() as usize;
        let (name, kind) = match &event {
            Event::Start(start) => (start.name().as_ref().to_vec(), Kind::Start),
            Event::Empty(start) => (start.name().as_ref().to_vec(), Kind::Empty),
            Event::End(end) => (end.name().as_ref().to_vec(), Kind::End),
            _ => (Vec::new(), Kind::Other),
        };
        match (kind, name.as_slice()) {
            (Kind::Start, b"w:p") => {
                paragraphs.push(Paragraph::default());
                open.push(paragraphs.len() - 1);
            }
            (Kind::End, b"w:p") => {
                open.pop();
            }
            (Kind::Start, b"w:t") => {
                run = Some(opened_run(xml, position, after_break));
                after_break = false;
            }
            (Kind::End, b"w:t") => {
                if let Some(mut run) = run.take() {
                    run.inner = run.inner.start..closing_at(xml, position);
                    run.tag = run.tag.start..position;
                    run.text = text_of(xml, &run.inner);
                    attach(&mut paragraphs, &open, run);
                }
            }
            (Kind::Empty, b"w:t") => {
                let open_at = opening_at(xml, position);
                let run = Run {
                    tag: open_at..position,
                    inner: open_at..open_at,
                    open: xml[open_at..position].to_string(),
                    name: "w:t".to_string(),
                    empty_element: true,
                    after_break,
                    text: String::new(),
                };
                after_break = false;
                attach(&mut paragraphs, &open, run);
            }
            (Kind::Start | Kind::Empty, b"w:br" | b"w:cr" | b"w:tab") => {
                after_break = true;
            }
            _ => {}
        }
    }
    Ok(paragraphs)
}

/// What an event was, which is all of the event the paragraph reader needs.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Start,
    Empty,
    End,
    Other,
}

/// The run that a `w:t` start tag begins.
fn opened_run(xml: &str, position: usize, after_break: bool) -> Run {
    let open_at = opening_at(xml, position);
    let inner_start = xml[open_at..position]
        .rfind('>')
        .map(|at| open_at + at + 1)
        .unwrap_or(position);
    Run {
        tag: open_at..position,
        inner: inner_start..inner_start,
        open: xml[open_at..position].to_string(),
        name: "w:t".to_string(),
        empty_element: false,
        after_break,
        text: String::new(),
    }
}

/// Where the tag that ended at `position` began.
fn opening_at(xml: &str, position: usize) -> usize {
    xml[..position]
        .rfind('<')
        .unwrap_or(position.saturating_sub(1))
}

/// Where the closing tag that ends at `position` begins, which is the end of
/// the text of the element.
fn closing_at(xml: &str, position: usize) -> usize {
    opening_at(xml, position)
}

/// The run is put in the paragraph it is in, which is the innermost open one. A
/// run outside any paragraph is not text of the document, and is left alone.
fn attach(paragraphs: &mut [Paragraph], open: &[usize], run: Run) {
    if let Some(&index) = open.last() {
        paragraphs[index].runs.push(run);
    }
}

/// The text between the tags, with the escapes of the XML undone.
fn text_of(xml: &str, range: &Range<usize>) -> String {
    match quick_xml::escape::unescape(&xml[range.clone()]) {
        Ok(text) => text.into_owned(),
        Err(_) => xml[range.clone()].to_string(),
    }
}

/// Writes a copy of the package in which the parts that were rewritten have the
/// new text. The parts that were not are copied as they are, compression and
/// all, so nothing else about the document changes.
fn package(bytes: &[u8], rewritten: &HashMap<String, String>) -> Result<Vec<u8>, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("the package cannot be opened again: {error}"))?;
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let name = raw(&mut archive, index)?.name().to_string();
        match rewritten.get(&name) {
            Some(xml) => {
                let file = archive
                    .by_index(index)
                    .map_err(|error| format!("`{name}` cannot be opened again: {error}"))?;
                // The method of the source is kept, except when this build of
                // Glossy cannot write it, in which case the text is deflated.
                let options: SimpleFileOptions = file
                    .options()
                    .compression_method(zip::CompressionMethod::Deflated);
                drop(file);
                writer
                    .start_file(&name, options)
                    .map_err(|error| format!("`{name}` cannot be written: {error}"))?;
                writer
                    .write_all(xml.as_bytes())
                    .map_err(|error| format!("`{name}` cannot be written: {error}"))?;
            }
            None => {
                writer
                    .raw_copy_file(raw(&mut archive, index)?)
                    .map_err(|error| format!("`{name}` cannot be copied: {error}"))?;
            }
        }
    }
    let written = writer
        .finish()
        .map_err(|error| format!("the package cannot be finished: {error}"))?;
    Ok(written.into_inner())
}

/// One entry of the package, as it was stored, so it can be copied without
/// being deflated and deflated again.
fn raw<'a, R: Read + Seek>(
    archive: &'a mut ZipArchive<R>,
    index: usize,
) -> Result<zip::read::ZipFile<'a, R>, String> {
    archive
        .by_index_raw(index)
        .map_err(|error| format!("the package cannot be read: {error}"))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::HashMap;

    const DOCUMENT: &str = "word/document.xml";
    const STYLES: &str = "word/styles.xml";
    const HEADER: &str = "word/header1.xml";

    pub(crate) fn package_of(parts: &[(&str, &str)]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, xml) in parts {
            writer
                .start_file(*name, SimpleFileOptions::default())
                .expect("a part starts");
            writer.write_all(xml.as_bytes()).expect("a part writes");
        }
        writer.finish().expect("the package finishes").into_inner()
    }

    fn document(body: &str) -> Vec<u8> {
        package_of(&[
            (
                DOCUMENT,
                &format!(
                    "<?xml version=\"1.0\"?><w:document xmlns:w=\"w\"><w:body>{body}</w:body>\
                     </w:document>"
                ),
            ),
            (STYLES, "<w:styles/>"),
        ])
    }

    fn part_of(bytes: &[u8], name: &str) -> String {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("the package reads");
        let mut file = archive.by_name(name).expect("the part is there");
        let mut xml = String::new();
        file.read_to_string(&mut xml).expect("the part reads");
        xml
    }

    #[test]
    fn a_paragraph_of_many_runs_is_read_as_one_text() {
        let bytes = document(
            "<w:p><w:r><w:t>This is </w:t></w:r><w:r><w:rPr/><w:t>bold</w:t></w:r>\
             <w:r><w:t> text.</w:t></w:r></w:p>",
        );
        let parsed = Document::parse(bytes).expect("it parses");
        assert_eq!(parsed.paragraphs().unwrap(), vec!["This is bold text."]);
    }

    #[test]
    fn the_translation_lands_in_the_first_run_and_the_rest_are_emptied() {
        let bytes = document(
            "<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Hello </w:t></w:r><w:r><w:t>world.</w:t></w:r>\
             </w:p>",
        );
        let parsed = Document::parse(bytes).expect("it parses");
        let translations = HashMap::from([(0usize, "你好，世界。".to_string())]);
        let written = parsed.rewrite(&translations).expect("it rewrites");
        let xml = part_of(&written, DOCUMENT);
        assert!(xml.contains("<w:t>你好，世界。</w:t>"));
        assert!(xml.contains("<w:rPr><w:b/></w:rPr>"));
        assert!(!xml.contains("Hello"));
        assert!(!xml.contains("world"));
        let again = Document::parse(written).expect("it parses again");
        assert_eq!(again.paragraphs().unwrap(), vec!["你好，世界。"]);
    }

    #[test]
    fn styles_tables_and_pictures_are_left_as_they_were() {
        let bytes = document(
            "<w:p><w:r><w:t>In a cell.</w:t></w:r></w:p>\
             <w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>\
             <w:tr><w:tc><w:p><w:r><w:t>Cell one</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
             <w:p><w:r><w:drawing><wp:inline/></w:drawing></w:r></w:p>",
        );
        let parsed = Document::parse(bytes).expect("it parses");
        assert_eq!(parsed.paragraphs().unwrap(), vec!["In a cell.", "Cell one"]);
        let translations =
            HashMap::from([(0usize, "One.".to_string()), (1usize, "Two.".to_string())]);
        let written = parsed.rewrite(&translations).expect("it rewrites");
        let xml = part_of(&written, DOCUMENT);
        assert!(xml.contains("<w:t>One.</w:t>"));
        assert!(xml.contains("<w:t>Two.</w:t>"));
        assert!(xml.contains("<w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>"));
        assert!(xml.contains("<w:drawing><wp:inline/></w:drawing>"));
        assert_eq!(part_of(&written, STYLES), "<w:styles/>");
    }

    #[test]
    fn an_escaped_ampersand_survives_a_round_trip() {
        let bytes = document("<w:p><w:r><w:t>A &amp; B</w:t></w:r></w:p>");
        let parsed = Document::parse(bytes).expect("it parses");
        assert_eq!(parsed.paragraphs().unwrap(), vec!["A & B"]);
        let translations = HashMap::from([(0usize, "C & D".to_string())]);
        let written = parsed.rewrite(&translations).expect("it rewrites");
        let xml = part_of(&written, DOCUMENT);
        assert!(xml.contains("<w:t>C &amp; D</w:t>"));
    }

    #[test]
    fn an_empty_element_takes_the_translation_and_keeps_its_attributes() {
        let bytes = document("<w:p><w:r><w:t xml:space=\"preserve\"/></w:r></w:p>");
        let parsed = Document::parse(bytes).expect("it parses");
        // A run with no text holds nothing to translate.
        assert_eq!(parsed.paragraphs().unwrap(), vec![""]);
        let translations = HashMap::from([(0usize, "Filled in.".to_string())]);
        let written = parsed.rewrite(&translations).expect("it rewrites");
        let xml = part_of(&written, DOCUMENT);
        assert!(xml.contains("<w:t xml:space=\"preserve\">Filled in.</w:t>"));
    }

    #[test]
    fn a_translation_that_starts_or_ends_with_a_space_keeps_it() {
        let bytes = document("<w:p><w:r><w:t>Hi there.</w:t></w:r></w:p>");
        let parsed = Document::parse(bytes).expect("it parses");
        let translations = HashMap::from([(0usize, " Bonjour. ".to_string())]);
        let written = parsed.rewrite(&translations).expect("it rewrites");
        let xml = part_of(&written, DOCUMENT);
        assert!(xml.contains("<w:t xml:space=\"preserve\"> Bonjour. </w:t>"));
        assert_eq!(
            Document::parse(written).unwrap().paragraphs().unwrap(),
            vec![" Bonjour. "]
        );
    }

    #[test]
    fn headers_and_footers_hold_paragraphs_too() {
        let bytes = package_of(&[
            (
                DOCUMENT,
                "<?xml version=\"1.0\"?><w:document xmlns:w=\"w\"><w:body>\
                 <w:p><w:r><w:t>Body.</w:t></w:r></w:p></w:body></w:document>",
            ),
            (
                HEADER,
                "<?xml version=\"1.0\"?><w:hdr xmlns:w=\"w\">\
                 <w:p><w:r><w:t>Header.</w:t></w:r></w:p></w:hdr>",
            ),
            (STYLES, "<w:styles/>"),
        ]);
        let parsed = Document::parse(bytes).expect("it parses");
        assert_eq!(parsed.paragraphs().unwrap(), vec!["Body.", "Header."]);
        let translations = HashMap::from([
            (0usize, "Corps.".to_string()),
            (1usize, "En-tête.".to_string()),
        ]);
        let written = parsed.rewrite(&translations).expect("it rewrites");
        assert!(part_of(&written, DOCUMENT).contains("Corps."));
        assert!(part_of(&written, HEADER).contains("En-tête."));
    }

    #[test]
    fn a_paragraph_inside_a_text_box_is_its_own_paragraph() {
        let bytes = document(
            "<w:p><w:r><w:t>Outside.</w:t></w:r>\
             <w:r><w:pict><v:shape><v:textbox><w:txbxContent>\
             <w:p><w:r><w:t>Inside.</w:t></w:r></w:p>\
             </w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>",
        );
        let parsed = Document::parse(bytes).expect("it parses");
        assert_eq!(parsed.paragraphs().unwrap(), vec!["Outside.", "Inside."]);
    }

    #[test]
    fn a_break_between_runs_separates_two_words() {
        let bytes =
            document("<w:p><w:r><w:t>One</w:t></w:r><w:r><w:br/><w:t>Two</w:t></w:r></w:p>");
        let parsed = Document::parse(bytes).expect("it parses");
        assert_eq!(parsed.paragraphs().unwrap(), vec!["One Two"]);
    }

    #[test]
    fn a_file_that_is_not_a_word_document_is_refused() {
        assert!(Document::parse(b"not a zip at all".to_vec()).is_err());
        let bytes = package_of(&[("word/document.xml", "<w:document")]);
        assert!(Document::parse(bytes).is_err());
    }

    #[test]
    fn parts_that_hold_no_text_of_the_document_are_left_alone() {
        let bytes = package_of(&[
            (
                DOCUMENT,
                "<?xml version=\"1.0\"?><w:document xmlns:w=\"w\"><w:body>\
                 <w:p><w:r><w:t>Text.</w:t></w:r></w:p></w:body></w:document>",
            ),
            (
                "word/styles.xml",
                "<w:styles><w:style><w:name w:val=\"Heading 1\"/></w:style></w:styles>",
            ),
        ]);
        let parsed = Document::parse(bytes).expect("it parses");
        assert_eq!(parsed.paragraphs().unwrap(), vec!["Text."]);
        let translations = HashMap::from([(0usize, "Texte.".to_string())]);
        let written = parsed.rewrite(&translations).expect("it rewrites");
        assert!(part_of(&written, STYLES).contains("Heading 1"));
    }
}
