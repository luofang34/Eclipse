//! An SVG document built as text, and the SMIL animations that play a sequence of frames
//! over and over.

use std::fmt::Write;

/// Path data that draws nothing, for frames where a shape is out of view.
pub(crate) const EMPTY_D: &str = "M0 0";

/// An SVG document of a fixed size.
pub(crate) struct Svg {
    text: String,
}

impl Svg {
    /// An empty document `width` by `height` with a white background.
    pub(crate) fn new(width: f64, height: f64) -> Self {
        let mut text = String::new();
        write!(
            text,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" \
             viewBox=\"0 0 {width} {height}\" font-family=\"Helvetica, Arial, sans-serif\">\
             <rect width=\"{width}\" height=\"{height}\" fill=\"#ffffff\"/>"
        )
        .ok();
        Self { text }
    }

    /// Adds markup as it is.
    pub(crate) fn push(&mut self, markup: &str) {
        self.text.push_str(markup);
    }

    /// Adds a path with `d` and the attributes `style`, unless `d` is empty.
    pub(crate) fn path(&mut self, d: &str, style: &str) {
        if !d.is_empty() {
            write!(self.text, "<path d=\"{d}\" {style}/>").ok();
        }
    }

    /// Adds text at `(x, y)` with the attributes `style`.
    pub(crate) fn text(&mut self, x: f64, y: f64, content: &str, style: &str) {
        write!(
            self.text,
            "<text x=\"{x:.1}\" y=\"{y:.1}\" {style}>{}</text>",
            escape(content)
        )
        .ok();
    }

    /// The document's text.
    pub(crate) fn finish(mut self) -> String {
        self.text.push_str("</svg>\n");
        self.text
    }
}

/// `text` with the characters XML reserves escaped.
pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A loop of equally long frames.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Animation {
    pub(crate) frames: usize,
    pub(crate) seconds: f64,
}

impl Animation {
    /// An attribute that takes each of `values` for one frame in turn.
    pub(crate) fn discrete(&self, attribute: &str, values: &[String]) -> String {
        format!(
            "<animate attributeName=\"{attribute}\" calcMode=\"discrete\" values=\"{}\" \
             dur=\"{}s\" repeatCount=\"indefinite\"/>",
            values.join(";"),
            self.seconds
        )
    }

    /// An attribute that moves smoothly through `values`, one at the start of each frame
    /// and the last at the loop's end.
    pub(crate) fn linear(&self, attribute: &str, values: &[f64]) -> String {
        let values: Vec<String> = values.iter().map(|v| format!("{v:.1}")).collect();
        format!(
            "<animate attributeName=\"{attribute}\" values=\"{}\" dur=\"{}s\" \
             repeatCount=\"indefinite\"/>",
            values.join(";"),
            self.seconds
        )
    }

    /// Shows an element during `frame` only.
    pub(crate) fn only_during(&self, frame: usize) -> String {
        let n = self.frames.max(1) as f64;
        let (start, end) = (frame as f64 / n, (frame + 1) as f64 / n);
        format!(
            "<animate attributeName=\"visibility\" calcMode=\"discrete\" \
             values=\"hidden;visible;hidden\" keyTimes=\"0;{start:.5};{end:.5}\" dur=\"{}s\" \
             repeatCount=\"indefinite\"/>",
            self.seconds
        )
    }

    /// The time of each frame's start over `start..end`, and `end` after the last.
    pub(crate) fn instants(&self, start: f64, end: f64) -> Vec<f64> {
        let n = self.frames.max(1);
        (0..=n)
            .map(|i| start + (end - start) * i as f64 / n as f64)
            .collect()
    }
}

#[cfg(test)]
mod tests;
