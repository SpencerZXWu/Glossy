//! The OCR pipeline itself: find the lines of text in the screenshot, then
//! read each one.
//!
//! Two ONNX models from PaddleOCR's PP-OCRv4 do the work — a detector that
//! turns the picture into a heat map of where text is, and a recogniser that
//! reads one line at a time — and the steps between them are the ones the
//! RapidOCR project wraps them in, so the numbers below are that project's
//! defaults for this pair of models.
//!
//! The engine is loaded once and kept, because reading the models off the disk
//! takes longer than recognising a screenshot does.

use std::sync::Mutex;

use ort::session::Session;
use ort::value::Tensor;

use super::models;
use super::vision::{components, dilate, min_area_rect, quad_contains, Image, Pt};

/// The detector resizes the picture so that its shorter side is this long,
/// unless it is already shorter — but never lets the longer side pass
/// [`DETECT_MAX_SIDE`].
///
/// A screen region is wide and short, and growing its shorter side to 736 on
/// its own grew a 900×140 region to 4736×736: fifteen times the pixels of the
/// picture it came from, for text that was already 40 pixels tall. The cap
/// keeps the detector's work near the size of the region whatever its shape,
/// and it is what makes reading a screenshot with more than one language cost
/// almost nothing: the detector is the slow half, and the recogniser is not.
const DETECT_SIDE: usize = 736;
/// The longest side the detector's picture may have.
const DETECT_MAX_SIDE: usize = 1440;
/// A pixel of the detector's heat map counts as text above this.
const DETECT_TEXT: f32 = 0.3;
/// A candidate line is kept if its average heat is above this.
const DETECT_BOX: f32 = 0.5;
/// How far a line is grown outwards before it is used as a cut-out.
const DETECT_UNCLIP: f32 = 1.6;
/// The shortest side a candidate line can have, in pixels.
const DETECT_MIN_SIDE: f32 = 3.0;
/// The shortest side the grown line can have, in pixels.
const DETECT_MIN_GROWN: f32 = 5.0;
/// The most candidate lines the detector will look at.
const DETECT_CANDIDATES: usize = 1000;
/// Lines shorter than this, in pixels, are dropped rather than cut out.
const CROP_MIN_SIDE: f32 = 3.0;

/// The height the recogniser reads at, and the width it always pads to.
const READ_HEIGHT: usize = 48;
const READ_WIDTH: usize = 320;
/// How many lines are read in one pass through the recogniser.
const READ_BATCH: usize = 6;
/// A line whose characters average below this is thrown away rather than
/// translated.
const READ_SCORE: f32 = 0.5;

/// One line of text found in the picture.
#[derive(Debug, Clone)]
pub struct Line {
    pub text: String,
    pub score: f32,
}

/// What is loaded and kept between screenshots.
///
/// The detector finds the lines of text whatever they are written in, so there
/// is exactly one of it however many languages are read with; a recogniser
/// writes one script and nothing else, so there is one per language that is
/// read with — held together with its dictionary, because the two only make
/// sense as a pair, and dropped when the language stops being read with.
struct Loaded {
    detect: Option<Session>,
    readers: Vec<(&'static str, Reader)>,
}

/// The recogniser of one language, loaded and ready.
struct Reader {
    session: Session,
    /// The character each output of the recogniser stands for: the blank it
    /// uses as a separator first, then the dictionary, then a space.
    labels: Vec<String>,
}

/// The loaded models, kept between screenshots.
///
/// Reading a model off the disk and setting up its runtime takes a moment, so
/// the detector and every recogniser are held here for as long as the app runs.
/// Everything is behind one lock because running a model needs it to be the
/// only one using it.
static LOADED: Mutex<Loaded> = Mutex::new(Loaded {
    detect: None,
    readers: Vec::new(),
});

/// Forgets the loaded models, so the next screenshot loads them again. Called
/// when the files they were built from are removed.
pub fn forget() {
    if let Ok(mut guard) = LOADED.lock() {
        guard.detect = None;
        guard.readers.clear();
    }
}

/// Opens one of the models, loading the runtime if it is not loaded yet.
///
/// The library itself arrives by path rather than being linked in, so it comes
/// with the models.
fn open(path: &std::path::Path, dll: &std::path::Path, which: &str) -> Result<Session, String> {
    ort::init_from(dll)
        .map_err(|error| format!("The OCR runtime is not usable: {error}"))?
        .commit();
    Session::builder()
        .map_err(|error| format!("The OCR engine could not be set up: {error}"))?
        .commit_from_file(path)
        .map_err(|error| format!("The OCR {which} model could not be opened: {error}"))
}

impl Reader {
    fn load(paths: &models::Paths) -> Result<Reader, String> {
        let text = std::fs::read_to_string(&paths.keys)
            .map_err(|error| format!("The OCR dictionary could not be read: {error}"))?;
        let mut labels = vec![String::new()];
        labels.extend(
            text.lines()
                .map(|line| line.trim_end_matches('\r').to_string()),
        );
        labels.push(" ".to_string());
        Ok(Reader {
            session: open(&paths.rec, &paths.dll, "recognition")?,
            labels,
        })
    }

    /// Reads the lines the detector found, one entry per box and in the same
    /// order. A box this language cannot read — a score below [`READ_SCORE`], or
    /// nothing its dictionary can write — is left empty rather than dropped, so
    /// the boxes of one language line up with the boxes of the next.
    fn read(&mut self, boxes: &[[Pt; 4]], image: &Image) -> Result<Vec<Option<Line>>, String> {
        let mut lines = Vec::with_capacity(boxes.len());
        for batch in boxes.chunks(READ_BATCH) {
            let cut: Vec<Image> = batch
                .iter()
                .filter_map(|points| image.crop(points))
                .collect();
            if cut.len() != batch.len() {
                return Err("A line of text could not be cut out of the picture.".to_string());
            }
            let (width, data) = reading_input(&cut);
            let classes = self.labels.len();
            let shape = [cut.len(), 3, READ_HEIGHT, width];
            let tensor = Tensor::from_array((shape, data)).map_err(|error| {
                format!("The picture could not be handed to the OCR engine: {error}")
            })?;
            let outputs = self
                .session
                .run(ort::inputs![tensor])
                .map_err(|error| format!("The OCR engine could not read the text: {error}"))?;
            let (_, logits) = outputs[0].try_extract_tensor::<f32>().map_err(|error| {
                format!("The OCR engine returned something unexpected: {error}")
            })?;

            let steps = logits.len() / (cut.len() * classes);
            if steps == 0 || steps * cut.len() * classes != logits.len() {
                return Err(
                    "The OCR engine returned an unexpected number of characters.".to_string(),
                );
            }
            for index in 0..batch.len() {
                let sample = &logits[index * steps * classes..(index + 1) * steps * classes];
                lines.push(
                    decode(sample, steps, &self.labels)
                        .filter(|(text, score)| *score >= READ_SCORE && !text.is_empty())
                        .map(|(text, score)| Line { text, score }),
                );
            }
        }
        Ok(lines)
    }
}

/// The boxes the detector sees, in reading order and scaled back to the size of
/// the original picture.
fn lines(detector: &mut Session, image: &Image) -> Result<Vec<[Pt; 4]>, String> {
    let (width, height) = detect_size(image.width, image.height);
    let resized = image.resize(width, height);
    let tensor = Tensor::from_array(([1usize, 3, height, width], resized.normalized()))
        .map_err(|error| format!("The picture could not be handed to the OCR engine: {error}"))?;
    let outputs = detector
        .run(ort::inputs![tensor])
        .map_err(|error| format!("The OCR engine could not look at the picture: {error}"))?;
    let (_, heat) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|error| format!("The OCR engine returned something unexpected: {error}"))?;
    if heat.len() != width * height {
        return Err("The OCR engine looked at the wrong part of the picture.".to_string());
    }
    Ok(text_boxes(heat, width, height, image.width, image.height))
}

/// Reads the text in a screenshot with every language in `paths`.
///
/// The detector is one model for all of them, so it runs once and every
/// language reads the very same lines; the reading with the highest score for
/// each line is the one that is kept. That is what lets a screenshot holding
/// more than one script come back right without the user saying which it is,
/// and it costs very little: the detector is the slow half of the pair and the
/// recogniser of one more language is a few hundred milliseconds.
///
/// `paths` is empty only when the settings could not be read at all, which
/// `models::checked` already answers with the default language.
pub fn recognize(
    paths: &[models::Paths],
    pixels: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<Line>, String> {
    if width == 0 || height == 0 {
        return Err("There is nothing to read in that part of the screen.".to_string());
    }
    if paths.is_empty() {
        return Err("No reading language is installed.".to_string());
    }
    let mut loaded = LOADED
        .lock()
        .map_err(|_| "The OCR engine is no longer usable. Please restart Glossy.".to_string())?;

    // The runtime and the detector are the same file for every language, so they
    // are loaded once and shared; a language that is not read with any more is
    // dropped with its recogniser, and one that is read with for the first time
    // is loaded now.
    if loaded.detect.is_none() {
        loaded.detect = Some(open(&paths[0].det, &paths[0].dll, "detection")?);
    }
    let wanted: Vec<&'static str> = paths.iter().map(models::pack_of).collect();
    loaded.readers.retain(|(id, _)| wanted.contains(id));
    for path in paths {
        let id = models::pack_of(path);
        if !loaded.readers.iter().any(|(loaded, _)| *loaded == id) {
            loaded.readers.push((id, Reader::load(path)?));
        }
    }

    let image = Image::from_bgra(pixels, width as usize, height as usize);
    // Every language reads the same boxes, so they are found once.
    let detector = loaded
        .detect
        .as_mut()
        .ok_or_else(|| "The OCR engine is not ready yet.".to_string())?;
    let boxes = lines(detector, &image)?;

    let mut best: Vec<Option<Line>> = vec![None; boxes.len()];
    for (_, reader) in loaded.readers.iter_mut() {
        keep_the_better_readings(&mut best, reader.read(&boxes, &image)?);
    }
    Ok(best.into_iter().flatten().collect())
}

/// Weighs one language's reading of every line against the best one so far.
///
/// The readings line up box by box — [`Reader::read`] answers one entry per box
/// whatever it made of it — so a line one language could not write is simply
/// left to the others instead of shifting every line after it.
fn keep_the_better_readings(best: &mut [Option<Line>], lines: Vec<Option<Line>>) {
    for (slot, line) in best.iter_mut().zip(lines) {
        let Some(line) = line else { continue };
        if slot.as_ref().map_or(true, |kept| line.score > kept.score) {
            *slot = Some(line);
        }
    }
}

/// The size the detector runs at: the shorter side is brought up to
/// [`DETECT_SIDE`], and both sides are rounded to a multiple of 32, which is
/// what the model's downsampling needs.
fn detect_size(width: usize, height: usize) -> (usize, usize) {
    let shorter = width.min(height);
    let longer = width.max(height);
    let ratio = if shorter < DETECT_SIDE {
        // Bring the shorter side up to [`DETECT_SIDE`] so small text is read,
        // but hold the longer side at [`DETECT_MAX_SIDE`] so a wide, short
        // region is not stretched far past the picture it came from.
        (DETECT_SIDE as f32 / shorter as f32).min(DETECT_MAX_SIDE as f32 / longer as f32)
    } else {
        1.0
    };
    let round = |side: usize| -> usize {
        let scaled = (side as f32 * ratio) as usize;
        ((scaled as f32 / 32.0).round() as usize * 32).max(32)
    };
    (round(width), round(height))
}

/// Turns the detector's heat map into the boxes around each line of text.
///
/// The heat map is a blob per line; each blob is turned into the smallest
/// rectangle that holds it, grown outwards by a share of its own size so the
/// first and last characters are inside it, and scaled back to the size of the
/// original picture.
fn text_boxes(
    heat: &[f32],
    width: usize,
    height: usize,
    target_width: usize,
    target_height: usize,
) -> Vec<[Pt; 4]> {
    let mask: Vec<bool> = heat.iter().map(|value| *value > DETECT_TEXT).collect();
    let grown_mask = dilate(&mask, width, height);

    let mut boxes = Vec::new();
    for shape in components(&grown_mask, width, height, DETECT_CANDIDATES) {
        let rect = min_area_rect(&shape);
        if rect.side < DETECT_MIN_SIDE {
            continue;
        }
        if box_score(heat, width, height, &rect.points) < DETECT_BOX {
            continue;
        }
        let distance = rect.area() * DETECT_UNCLIP / rect.perimeter().max(f32::EPSILON);
        let grown = rect.grown(distance);
        if grown.side < DETECT_MIN_GROWN {
            continue;
        }

        let mut points = [Pt { x: 0.0, y: 0.0 }; 4];
        for (index, point) in grown.points.iter().enumerate() {
            points[index] = Pt {
                x: (point.x / width as f32 * target_width as f32)
                    .round()
                    .clamp(0.0, target_width.saturating_sub(1) as f32),
                y: (point.y / height as f32 * target_height as f32)
                    .round()
                    .clamp(0.0, target_height.saturating_sub(1) as f32),
            };
        }
        let width_of = |a: Pt, b: Pt| ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
        if width_of(points[0], points[1]) <= CROP_MIN_SIDE
            || width_of(points[0], points[3]) <= CROP_MIN_SIDE
        {
            continue;
        }
        boxes.push(points);
    }
    reading_order(&mut boxes);
    boxes
}

/// How much of a candidate box is actually text: the average heat over the
/// pixels inside it.
fn box_score(heat: &[f32], width: usize, height: usize, points: &[Pt; 4]) -> f32 {
    let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
    let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
    for point in points {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }
    let x0 = (min_x.floor() as i64).clamp(0, width as i64 - 1);
    let x1 = (max_x.ceil() as i64).clamp(0, width as i64 - 1);
    let y0 = (min_y.floor() as i64).clamp(0, height as i64 - 1);
    let y1 = (max_y.ceil() as i64).clamp(0, height as i64 - 1);

    // The shape is walked in its own little frame, the way OpenCV fills it.
    let local: [Pt; 4] = std::array::from_fn(|index| Pt {
        x: points[index].x.trunc() - x0 as f32,
        y: points[index].y.trunc() - y0 as f32,
    });
    let (mut total, mut count) = (0f32, 0usize);
    for y in y0..=y1 {
        for x in x0..=x1 {
            if quad_contains(&local, (x - x0) as i32, (y - y0) as i32) {
                total += heat[y as usize * width + x as usize];
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// Puts the boxes in the order they would be read: down the page, and left to
/// right within a line, with the small vertical differences between boxes on
/// the same line smoothed out.
fn reading_order(boxes: &mut [[Pt; 4]]) {
    boxes.sort_by(|left, right| {
        left[0]
            .y
            .partial_cmp(&right[0].y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                left[0]
                    .x
                    .partial_cmp(&right[0].x)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    for index in 0..boxes.len().saturating_sub(1) {
        let mut current = index;
        while boxes[current + 1][0].y - boxes[current][0].y < 10.0
            && boxes[current + 1][0].x < boxes[current][0].x
        {
            boxes.swap(current, current + 1);
            if current == 0 {
                break;
            }
            current -= 1;
        }
    }
}

/// The tensor a batch of cut-out lines is read as: each line is scaled to the
/// height the recogniser reads at, keeping its shape, and the batch is padded
/// to the width of its widest line.
fn reading_input(images: &[Image]) -> (usize, Vec<f32>) {
    let widest = images
        .iter()
        .map(|image| image.width as f32 / image.height.max(1) as f32)
        .fold(READ_WIDTH as f32 / READ_HEIGHT as f32, f32::max);
    let width = (READ_HEIGHT as f32 * widest) as usize;
    let plane = READ_HEIGHT * width;

    let mut data = vec![0f32; images.len() * plane * 3];
    for (index, image) in images.iter().enumerate() {
        let ratio = image.width as f32 / image.height.max(1) as f32;
        let scaled = (READ_HEIGHT as f32 * ratio).ceil() as usize;
        let scaled = scaled.min(width).max(1);
        let resized = image.resize(scaled, READ_HEIGHT);
        let normalized = resized.normalized();
        let target = index * plane * 3;
        for row in 0..READ_HEIGHT {
            for channel in 0..3 {
                let from = channel * READ_HEIGHT * scaled + row * scaled;
                let to = channel * plane + row * width;
                data[target + to..target + to + scaled]
                    .copy_from_slice(&normalized[from..from + scaled]);
            }
        }
    }
    (width, data)
}

/// Turns the recogniser's per-character guesses into the text of one line.
///
/// The recogniser repeats each character across the steps it is written in, and
/// uses a blank at index zero to separate characters that would otherwise run
/// together, so a character is kept when it differs from the step before it and
/// is not a blank. The score is the average certainty of the characters kept.
fn decode(probabilities: &[f32], steps: usize, labels: &[String]) -> Option<(String, f32)> {
    let classes = labels.len();
    if probabilities.len() != steps * classes {
        return None;
    }
    let mut text = String::new();
    let mut total = 0f32;
    let mut kept = 0usize;
    let mut previous = usize::MAX;
    for step in 0..steps {
        let row = &probabilities[step * classes..(step + 1) * classes];
        let (index, value) =
            row.iter()
                .enumerate()
                .fold((0usize, f32::MIN), |best, (index, value)| {
                    if *value > best.1 {
                        (index, *value)
                    } else {
                        best
                    }
                });
        if index != 0 && index != previous {
            text.push_str(&labels[index]);
            total += value;
            kept += 1;
        }
        previous = index;
    }
    if kept == 0 {
        None
    } else {
        Some((text.trim().to_string(), total / kept as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Times the two halves of the pipeline over a sample picture.
    ///
    /// The detector runs once per screenshot whatever is checked; the recogniser
    /// runs once per line, and once more for every language that is read. Which
    /// of the two the seconds go into decides how much a language costs.
    ///
    /// `GLOSSY_OCR_CHECK=<dir>` `GLOSSY_OCR_SAMPLE=<file>.bgra`.
    #[test]
    #[ignore = "needs the downloaded models and a sample picture"]
    fn a_sample_is_read_in_stages() {
        let files = std::path::PathBuf::from(std::env::var("GLOSSY_OCR_CHECK").unwrap());
        let sample = std::path::PathBuf::from(std::env::var("GLOSSY_OCR_SAMPLE").unwrap());
        let stem = sample.file_stem().unwrap().to_string_lossy().to_string();
        let size = stem.rsplit_once('-').unwrap().1;
        let (width, height) = size.split_once('x').unwrap();
        let pixels = std::fs::read(&sample).unwrap();
        let image = Image::from_bgra(&pixels, width.parse().unwrap(), height.parse().unwrap());
        let paths = models::Paths {
            dll: files.join("onnxruntime.dll"),
            det: files.join("ch_PP-OCRv4_det_mobile.onnx"),
            rec: files.join("ch_PP-OCRv4_rec_mobile.onnx"),
            keys: files.join("ppocr_keys_v1.txt"),
        };
        let started = std::time::Instant::now();
        let mut detector = open(&paths.det, &paths.dll, "detection").unwrap();
        let mut reader = Reader::load(&paths).unwrap();
        println!("load      {:>5} ms", started.elapsed().as_millis());
        for pass in 0..3 {
            let started = std::time::Instant::now();
            let boxes = lines(&mut detector, &image).unwrap();
            let detect = started.elapsed().as_millis();
            let started = std::time::Instant::now();
            let read = reader.read(&boxes, &image).unwrap();
            println!(
                "pass {pass}   detect {detect:>5} ms ({} lines), recognise {:>5} ms -> {}",
                boxes.len(),
                started.elapsed().as_millis(),
                read.iter().flatten().count()
            );
        }
    }

    /// A dictionary that stands in for the real one, so the tests do not need
    /// the models to be downloaded.
    fn labels() -> Vec<String> {
        let mut labels = vec![String::new()];
        labels.push("你".to_string());
        labels.push("好".to_string());
        labels.push("A".to_string());
        labels.push("b".to_string());
        labels.push(" ".to_string());
        labels
    }

    /// Turns a text into the guesses the recogniser would make: each character
    /// held for a couple of steps, with a blank before it.
    fn guesses(text: &str, certainty: f32) -> (Vec<f32>, usize) {
        let labels = labels();
        let classes = labels.len();
        let mut rows = Vec::new();
        for character in text.chars() {
            let index = labels
                .iter()
                .position(|label| label == &character.to_string())
                .unwrap();
            let mut blank = vec![0.0f32; classes];
            blank[0] = certainty;
            rows.push(blank);
            let mut row = vec![0.0f32; classes];
            row[index] = certainty;
            rows.push(row);
        }
        let steps = rows.len();
        (rows.into_iter().flatten().collect(), steps)
    }

    #[test]
    fn a_text_is_read_back_the_way_it_was_written() {
        let (probabilities, steps) = guesses("A你好", 0.9);
        let (text, score) = decode(&probabilities, steps, &labels()).unwrap();
        assert_eq!(text, "A你好");
        assert!(score > 0.8 && score < 1.0, "{score}");
    }

    #[test]
    fn a_character_written_in_the_same_step_twice_is_only_read_once() {
        // The same guess for three steps in a row, which is how the recogniser
        // writes a character it is sure about.
        let labels = labels();
        let classes = labels.len();
        let mut rows = vec![vec![0.0; classes]; 3];
        for row in &mut rows {
            row[3] = 0.9;
        }
        let probabilities: Vec<f32> = rows.into_iter().flatten().collect();
        let (text, score) = decode(&probabilities, 3, &labels).unwrap();
        assert_eq!(text, "A");
        assert!((score - 0.9).abs() < 0.001);
    }

    #[test]
    fn the_same_character_twice_in_a_row_is_read_twice_when_a_blank_separates_it() {
        let labels = labels();
        let classes = labels.len();
        let mut rows = vec![vec![0.0; classes]; 4];
        rows[0][0] = 0.9; // the blank the recogniser uses between characters
        rows[1][3] = 0.9; // A
        rows[2][0] = 0.9; // a gap
        rows[3][3] = 0.9; // A again
        let probabilities: Vec<f32> = rows.into_iter().flatten().collect();
        let (text, _) = decode(&probabilities, 4, &labels).unwrap();
        assert_eq!(text, "AA");
    }

    #[test]
    fn the_readings_of_every_language_are_weighed_line_by_line() {
        let line = |text: &str, score: f32| {
            Some(Line {
                text: text.to_string(),
                score,
            })
        };
        // Japanese and English checked: the Japanese recogniser reads the kana
        // and the Latin one reads the English, and neither wins outright.
        let mut best = vec![None, None];
        keep_the_better_readings(&mut best, vec![line("日本語", 0.98), None]);
        keep_the_better_readings(&mut best, vec![None, line("Hello", 0.99)]);
        assert_eq!(best[0].as_ref().unwrap().text, "日本語");
        assert_eq!(best[1].as_ref().unwrap().text, "Hello");

        // A second language that reads the same line better takes it over, and
        // one that reads it worse leaves it alone.
        keep_the_better_readings(
            &mut best,
            vec![line("日本語です", 0.99), line("Hello!", 0.10)],
        );
        assert_eq!(best[0].as_ref().unwrap().text, "日本語です");
        assert_eq!(best[1].as_ref().unwrap().text, "Hello");

        // A reading of a line no language can write leaves that line to the
        // others rather than shifting the ones behind it.
        let mut best = vec![None, line("kept", 0.9)];
        keep_the_better_readings(&mut best, vec![None, None]);
        assert!(best[0].is_none());
        assert_eq!(best[1].as_ref().unwrap().text, "kept");
    }

    #[test]
    fn a_line_of_nothing_but_blanks_is_not_read_at_all() {
        let labels = labels();
        let classes = labels.len();
        let mut rows = vec![vec![0.0; classes]; 4];
        for row in &mut rows {
            row[0] = 1.0;
        }
        let probabilities: Vec<f32> = rows.into_iter().flatten().collect();
        assert!(decode(&probabilities, 4, &labels).is_none());
    }

    #[test]
    fn the_wider_side_of_the_picture_is_brought_up_to_the_detectors_size() {
        // A picture that is already big enough is left as it is.
        assert_eq!(detect_size(1920, 1920), (1920, 1920));

        // A landscape picture whose longer side would pass the cap: the cap is
        // what decides, so the height stops short of 736 rather than the region
        // growing to nearly five thousand pixels wide.
        assert_eq!(detect_size(1920, 300), (DETECT_MAX_SIDE, 224));

        // A wide, short region — what a screenshot of a screen really is —
        // stays near its own size instead of being stretched to its shorter
        // side times five.
        let (width, height) = detect_size(900, 140);
        assert_eq!((width, height), (1440, 224));
        assert!(width * height < 900 * 140 * 3);

        // A picture already big enough is left alone, rounded to the multiple
        // of 32 the model needs.
        assert_eq!(detect_size(1024, 1024), (1024, 1024));
        assert_eq!(detect_size(100, 100), (736, 736));
    }

    #[test]
    fn a_block_of_text_in_the_heat_map_becomes_one_box_around_it() {
        let (width, height) = (64, 64);
        let mut heat = vec![0f32; width * height];
        // A line of text: a bar 40 long and 10 high with a gap in the middle,
        // well inside the picture.
        for y in 20..30 {
            for x in 10..50 {
                if !(28..29).contains(&x) {
                    heat[y * width + x] = 1.0;
                }
            }
        }
        let boxes = text_boxes(&heat, width, height, width, height);
        assert_eq!(boxes.len(), 1, "{boxes:?}");
        let found = boxes[0];
        // The box holds the bar with a little room around it, and comes back in
        // the detector's order.
        assert!(found[0].x < 10.0 && found[0].y < 20.0, "{found:?}");
        assert!(found[2].x > 50.0 && found[2].y > 30.0, "{found:?}");
        assert!(
            found[1].x > found[0].x && found[1].y <= found[0].y + 1.0,
            "{found:?}"
        );
        assert!(found[3].y > found[1].y, "{found:?}");
    }

    #[test]
    fn a_heat_map_with_nothing_in_it_produces_no_boxes() {
        let heat = vec![0f32; 32 * 32];
        assert!(text_boxes(&heat, 32, 32, 32, 32).is_empty());
    }

    #[test]
    fn a_faint_block_is_not_a_line_of_text() {
        let (width, height) = (64, 64);
        let mut heat = vec![0f32; width * height];
        for y in 20..30 {
            for x in 10..50 {
                heat[y * width + x] = 0.2;
            }
        }
        assert!(text_boxes(&heat, width, height, width, height).is_empty());
    }

    #[test]
    fn the_boxes_come_back_in_reading_order() {
        let mut boxes = [
            [
                Pt { x: 100.0, y: 50.0 },
                Pt { x: 140.0, y: 50.0 },
                Pt { x: 140.0, y: 62.0 },
                Pt { x: 100.0, y: 62.0 },
            ],
            [
                Pt { x: 10.0, y: 52.0 },
                Pt { x: 40.0, y: 52.0 },
                Pt { x: 40.0, y: 64.0 },
                Pt { x: 10.0, y: 64.0 },
            ],
            [
                Pt { x: 10.0, y: 10.0 },
                Pt { x: 90.0, y: 10.0 },
                Pt { x: 90.0, y: 22.0 },
                Pt { x: 10.0, y: 22.0 },
            ],
        ];
        reading_order(&mut boxes);
        assert_eq!(boxes[0][0].y, 10.0);
        assert_eq!(boxes[1][0].x, 10.0);
        assert_eq!(boxes[2][0].x, 100.0);
    }

    #[test]
    fn a_batch_of_lines_is_padded_to_the_width_of_its_widest() {
        let lines = vec![Image {
            width: 96,
            height: 48,
            data: vec![255; 96 * 48 * 3],
        }];
        let (width, data) = reading_input(&lines);
        // The recogniser always works on strips 320 columns wide, so a line
        // twice as wide as it is high fills 96 of them and the rest is padding.
        assert_eq!(width, 320);
        assert_eq!(data.len(), 3 * 48 * width);
        let last_row_start = 2 * 48 * width + 47 * width;
        assert!(data[last_row_start..last_row_start + 96]
            .iter()
            .all(|value| *value == 1.0));
        assert!(data[last_row_start + 96..last_row_start + width]
            .iter()
            .all(|value| *value == 0.0));
    }
}
