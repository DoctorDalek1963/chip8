//! This module is all about error handling.

use crate::span::{LineOffsets, Span};
use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};
use lazy_static::lazy_static;
use std::{
    cmp,
    sync::{
        atomic::{AtomicBool, Ordering},
        RwLock,
    },
};

/// Have we encountered at least one error before runtime?
pub static HAD_ERROR: AtomicBool = AtomicBool::new(false);

lazy_static! {
    /// The LineOffsets of the code being worked with.
    static ref LINE_OFFSETS: RwLock<LineOffsets> = RwLock::new(LineOffsets::new(""));

    /// The source code that we're working with.
    static ref SOURCE_CODE: RwLock<String> = RwLock::new(String::new());
}

/// Initialise the error reporting with the given source code.
pub fn init_error_reporting(code: String) {
    *LINE_OFFSETS.write().unwrap() = LineOffsets::new(&code);
    *SOURCE_CODE.write().unwrap() = code;
}

/// The level of severity in an error/warning message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SeverityLevel {
    /// A fatal error.
    Error,

    /// A non-fatal warning.
    Warning,
}

/// Report an error.
pub fn report_error(span: Span, message: &str) {
    print_error_message(Some(span), message, SeverityLevel::Error);
    HAD_ERROR.store(true, Ordering::Relaxed);
}

/// Report a non-fatal warning.
pub fn report_warning(span: Span, message: &str) {
    print_error_message(Some(span), message, SeverityLevel::Warning);
}

/// Print the given error message.
fn print_error_message(span: Option<Span>, message: &str, level: SeverityLevel) {
    let (highlight_color, severity_name) = match level {
        SeverityLevel::Error => (Color::Red, "ERROR"),
        SeverityLevel::Warning => (Color::Yellow, "WARNING"),
    };

    let message = if let Some(span) = span {
        let (start_line, start_nl) = LINE_OFFSETS
            .read()
            .unwrap()
            .line_and_newline_offset(span.start);
        let (end_line, end_nl) = LINE_OFFSETS
            .read()
            .unwrap()
            .line_and_newline_offset(span.end);
        let start_col = span.start - start_nl + 1;
        let end_col = span.end - end_nl + 1;
        let line_number_width =
            cmp::max(start_line.to_string().len(), end_line.to_string().len()) + 1;

        let mut message = format!(": {message}\n");
        message.push_str(&format!(
            "{:width$}{}{}-->{}{} {start_line}:{start_col}\n",
            "",
            SetForegroundColor(Color::Blue),
            Attribute::Bold,
            ResetColor,
            Attribute::Reset,
            width = line_number_width - 1,
        ));
        message.push_str(&format!(
            "{}{}{:line_number_width$}|{}{}\n",
            SetForegroundColor(Color::Blue),
            Attribute::Bold,
            "",
            ResetColor,
            Attribute::Reset,
        ));

        if start_line == end_line {
            message.push_str(&format!(
                "{}{}{start_line}{:width$}|{}{} ",
                SetForegroundColor(Color::Blue),
                Attribute::Bold,
                "",
                ResetColor,
                Attribute::Reset,
                width = line_number_width - start_line.to_string().len(),
            ));
            message.push_str(
                SOURCE_CODE
                    .read()
                    .unwrap()
                    .lines()
                    .nth(start_line.saturating_sub(1))
                    .unwrap_or(""),
            );
            message.push('\n');
            message.push_str(&format!(
                "{}{}{:line_number_width$}|{}{} ",
                SetForegroundColor(Color::Blue),
                Attribute::Bold,
                "",
                ResetColor,
                Attribute::Reset,
            ));

            if start_col == end_col {
                message.push_str(&format!(
                    "{}{}{:space_width$}^{}{}",
                    SetForegroundColor(highlight_color),
                    Attribute::Bold,
                    "",
                    ResetColor,
                    Attribute::Reset,
                    space_width = start_col.saturating_sub(1),
                ));
            } else {
                message.push_str(&format!(
                    "{}{}{:space_width$}^{:-<dash_width$}^{}{}",
                    SetForegroundColor(highlight_color),
                    Attribute::Bold,
                    "",
                    "",
                    ResetColor,
                    Attribute::Reset,
                    space_width = start_col.saturating_sub(1),
                    dash_width = end_col.saturating_sub(start_col).saturating_sub(1),
                ));
            }
        } else {
            let source_code_text = SOURCE_CODE.read().unwrap();

            for line in start_line..=end_line {
                let line_text = source_code_text
                    .lines()
                    .nth(line.saturating_sub(1))
                    .unwrap_or("");

                message.push_str(&format!(
                    "{}{}{line}{:width$}|{}{} ",
                    SetForegroundColor(Color::Blue),
                    Attribute::Bold,
                    "",
                    ResetColor,
                    Attribute::Reset,
                    width = line_number_width - line.to_string().len(),
                ));
                message.push_str(line_text);
                message.push('\n');
                message.push_str(&format!(
                    "{}{}{:line_number_width$}|{}{} ",
                    SetForegroundColor(Color::Blue),
                    Attribute::Bold,
                    "",
                    ResetColor,
                    Attribute::Reset,
                ));

                if line == start_line {
                    message.push_str(&format!(
                        "{}{}{:space_width$}^{:-<dash_width$}{}{}",
                        SetForegroundColor(highlight_color),
                        Attribute::Bold,
                        "",
                        "",
                        ResetColor,
                        Attribute::Reset,
                        space_width = start_col.saturating_sub(1),
                        dash_width = line_text.len().saturating_sub(start_col),
                    ));
                } else if line == end_line {
                    message.push_str(&format!(
                        "{}{}{:-<dash_width$}^{}{}",
                        SetForegroundColor(highlight_color),
                        Attribute::Bold,
                        "",
                        ResetColor,
                        Attribute::Reset,
                        dash_width = line_text.len().saturating_sub(end_col),
                    ));
                } else {
                    message.push_str(&format!(
                        "{}{}{:-<dash_width$}{}{}",
                        SetForegroundColor(highlight_color),
                        Attribute::Bold,
                        "",
                        ResetColor,
                        Attribute::Reset,
                        dash_width = line_text.len(),
                    ));
                }

                message.push('\n');
            }
        }

        message.push_str("\n\n");
        message
    } else {
        format!(": {message}\n")
    };

    execute!(
        std::io::stderr(),
        SetForegroundColor(highlight_color),
        SetAttribute(Attribute::Bold),
        Print(severity_name),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print(message)
    )
    .expect("Should be able to print error messages with crossterm");
}
