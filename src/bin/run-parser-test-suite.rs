#![warn(unused_qualifications)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::let_underscore_untyped,
    clippy::missing_errors_doc,
    clippy::missing_safety_doc,
    clippy::too_many_lines
)]

use libyaml_safer::{EventData, Parser, ScalarStyle};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{self, ExitCode};
use std::slice;

pub(crate) fn test_main(
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let mut parser = Parser::new();

    let mut stdin = io::BufReader::new(stdin);
    parser.set_input(&mut stdin);

    loop {
        let event = match parser.parse() {
            Err(err) => {
                let error = format!("Parse error: {err}");
                return Err(error.into());
            }
            Ok(event) => event,
        };

        let mut is_end = false;

        match &event.data {
            EventData::StreamStart { .. } => {
                _ = stdout.write_all("+STR\n".as_bytes());
            }
            EventData::StreamEnd => {
                is_end = true;
                _ = stdout.write_all("-STR\n".as_bytes());
            }
            EventData::DocumentStart { implicit, .. } => {
                _ = stdout.write_all("+DOC".as_bytes());
                if !implicit {
                    _ = stdout.write_all(" ---".as_bytes());
                }
                _ = stdout.write_all("\n".as_bytes());
            }
            EventData::DocumentEnd { implicit } => {
                _ = stdout.write_all("-DOC".as_bytes());
                if !implicit {
                    _ = stdout.write_all(" ...".as_bytes());
                }
                _ = stdout.write_all("\n".as_bytes());
            }
            EventData::Alias { anchor } => {
                _ = writeln!(stdout, "=ALI *{anchor}");
            }
            EventData::Scalar {
                anchor,
                tag,
                value,
                style,
                ..
            } => {
                let _ = stdout.write_all("=VAL".as_bytes());
                if let Some(anchor) = anchor {
                    _ = write!(stdout, " &{anchor}");
                }
                if let Some(tag) = tag {
                    _ = write!(stdout, " <{tag}>");
                }
                _ = stdout.write_all(match style {
                    ScalarStyle::Plain => b" :",
                    ScalarStyle::SingleQuoted => b" '",
                    ScalarStyle::DoubleQuoted => b" \"",
                    ScalarStyle::Literal => b" |",
                    ScalarStyle::Folded => b" >",
                    _ => process::abort(),
                });
                print_escaped(stdout, value);
                _ = stdout.write_all("\n".as_bytes());
            }
            EventData::SequenceStart { anchor, tag, .. } => {
                let _ = stdout.write_all("+SEQ".as_bytes());
                if let Some(anchor) = anchor {
                    _ = write!(stdout, " &{anchor}");
                }
                if let Some(tag) = tag {
                    _ = write!(stdout, " <{tag}>");
                }
                _ = stdout.write_all("\n".as_bytes());
            }
            EventData::SequenceEnd => {
                _ = stdout.write_all("-SEQ\n".as_bytes());
            }
            EventData::MappingStart { anchor, tag, .. } => {
                let _ = stdout.write_all("+MAP".as_bytes());
                if let Some(anchor) = anchor {
                    _ = write!(stdout, " &{anchor}");
                }
                if let Some(tag) = tag {
                    _ = write!(stdout, " <{tag}>");
                }
                _ = stdout.write_all("\n".as_bytes());
            }
            EventData::MappingEnd => {
                _ = stdout.write_all("-MAP\n".as_bytes());
            }
        }

        if is_end {
            break;
        }
    }
    Ok(())
}

fn print_escaped(stdout: &mut dyn Write, s: &str) {
    for ch in s.bytes() {
        let repr = match &ch {
            b'\\' => b"\\\\",
            b'\0' => b"\\0",
            b'\x08' => b"\\b",
            b'\n' => b"\\n",
            b'\r' => b"\\r",
            b'\t' => b"\\t",
            c => slice::from_ref(c),
        };
        let _ = stdout.write_all(repr);
    }
}

fn main() -> ExitCode {
    let args = env::args_os().skip(1);
    if args.len() == 0 {
        _ = io::stderr().write_all("Usage: run-parser-test-suite <in.yaml>...".as_bytes());
        return ExitCode::FAILURE;
    }
    for arg in args {
        let mut stdin = File::open(arg).unwrap();
        let mut stdout = io::stdout();
        let result = test_main(&mut stdin, &mut stdout);
        if let Err(err) = result {
            let _ = writeln!(io::stderr(), "{err}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
