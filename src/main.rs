use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

mod csv;
mod markdown;
mod report;
mod table;

use report::Report;

#[derive(Clone, Copy, PartialEq)]
enum Format {
    Csv,
    Markdown,
}

struct Args {
    input: Option<String>,
    from: Option<Format>,
    to: Format,
    output: Option<String>,
    json: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1).collect())?;

    let reading_stdin = matches!(args.input.as_deref(), None | Some("-"));
    let writing_stdout = matches!(args.output.as_deref(), None | Some("-"));

    let input_format = match args.from {
        Some(f) => f,
        None if reading_stdin => {
            return Err("reading from stdin requires --from csv|md".to_string())
        }
        None => detect_format(args.input.as_deref().unwrap())?,
    };

    let content = if reading_stdin {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("reading stdin: {}", e))?;
        buf
    } else {
        let path = args.input.as_deref().unwrap();
        fs::read_to_string(path).map_err(|e| format!("reading {}: {}", path, e))?
    };

    let (table, warnings) = match input_format {
        Format::Csv => csv::parse(&content),
        Format::Markdown => markdown::parse(&content),
    }?;

    if table.headers.is_empty() {
        return Err("no columns found in input".to_string());
    }

    let output_content = match args.to {
        Format::Csv => csv::write(&table),
        Format::Markdown => markdown::write(&table),
    };

    if writing_stdout {
        io::stdout()
            .write_all(output_content.as_bytes())
            .map_err(|e| format!("writing stdout: {}", e))?;
    } else {
        let path = args.output.as_deref().unwrap();
        fs::write(path, &output_content).map_err(|e| format!("writing {}: {}", path, e))?;
    }

    let report = Report {
        input_path: args.input.clone().unwrap_or_else(|| "-".to_string()),
        input_format: format_name(input_format).to_string(),
        output_path: args.output.clone().unwrap_or_else(|| "-".to_string()),
        output_format: format_name(args.to).to_string(),
        rows: table.rows.len(),
        columns: table.headers.len(),
        warnings,
    };

    let report_text = if args.json {
        report.to_json()
    } else {
        report.to_human()
    };

    // Keep converted content on stdout pipeable: when it's the thing going to
    // stdout, the report goes to stderr instead of interleaving with it.
    if writing_stdout {
        eprint!("{}", report_text);
    } else {
        print!("{}", report_text);
    }

    Ok(())
}

fn parse_args(raw: Vec<String>) -> Result<Args, String> {
    let mut input: Option<String> = None;
    let mut from: Option<Format> = None;
    let mut to: Option<Format> = None;
    let mut output: Option<String> = None;
    let mut json = false;

    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--from" => {
                let v = iter.next().ok_or("--from requires a value (csv or md)")?;
                from = Some(parse_format(&v)?);
            }
            "--to" => {
                let v = iter.next().ok_or("--to requires a value (csv or md)")?;
                to = Some(parse_format(&v)?);
            }
            "-o" | "--output" => {
                let v = iter.next().ok_or("--output requires a file path")?;
                output = Some(v);
            }
            "--json" => json = true,
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-" if input.is_none() => input = Some("-".to_string()),
            other if !other.starts_with('-') && input.is_none() => {
                input = Some(other.to_string());
            }
            other => return Err(format!("unrecognized argument '{}'", other)),
        }
    }

    let to = to.ok_or("missing --to csv|md")?;

    Ok(Args {
        input,
        from,
        to,
        output,
        json,
    })
}

fn parse_format(v: &str) -> Result<Format, String> {
    match v {
        "csv" => Ok(Format::Csv),
        "md" | "markdown" => Ok(Format::Markdown),
        other => Err(format!("unknown format '{}': expected csv or md", other)),
    }
}

fn print_usage() {
    println!("usage: csvmd [<input>] --to csv|md [-o <output>] [--from csv|md] [--json]");
    println!();
    println!("<input> and -o/--output default to stdin/stdout when omitted, or given as -.");
    println!("--from is required when reading from stdin, since there's no extension to detect the format from.");
}

fn detect_format(path: &str) -> Result<Format, String> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "csv" => Ok(Format::Csv),
        "md" | "markdown" => Ok(Format::Markdown),
        _ => Err(format!(
            "cannot detect the format of {} from its extension; expected .csv, .md, or .markdown",
            path
        )),
    }
}

fn format_name(f: Format) -> &'static str {
    match f {
        Format::Csv => "csv",
        Format::Markdown => "markdown",
    }
}
