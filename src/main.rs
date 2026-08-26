use std::env;
use std::fs;
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
    input: String,
    to: Format,
    output: String,
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
    let input_format = detect_format(&args.input)?;

    let content = fs::read_to_string(&args.input)
        .map_err(|e| format!("reading {}: {}", args.input, e))?;

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

    fs::write(&args.output, &output_content)
        .map_err(|e| format!("writing {}: {}", args.output, e))?;

    let report = Report {
        input_path: args.input.clone(),
        input_format: format_name(input_format).to_string(),
        output_path: args.output.clone(),
        output_format: format_name(args.to).to_string(),
        rows: table.rows.len(),
        columns: table.headers.len(),
        warnings,
    };

    if args.json {
        print!("{}", report.to_json());
    } else {
        print!("{}", report.to_human());
    }

    Ok(())
}

fn parse_args(raw: Vec<String>) -> Result<Args, String> {
    let mut input: Option<String> = None;
    let mut to: Option<Format> = None;
    let mut output: Option<String> = None;
    let mut json = false;

    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--to" => {
                let v = iter.next().ok_or("--to requires a value (csv or md)")?;
                to = Some(match v.as_str() {
                    "csv" => Format::Csv,
                    "md" | "markdown" => Format::Markdown,
                    other => return Err(format!("unknown format '{}': expected csv or md", other)),
                });
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
            other if !other.starts_with('-') && input.is_none() => {
                input = Some(other.to_string());
            }
            other => return Err(format!("unrecognized argument '{}'", other)),
        }
    }

    let input = input.ok_or("missing input file; usage: csvmd <input> --to csv|md -o <output>")?;
    let to = to.ok_or("missing --to csv|md")?;
    let output = output.ok_or("missing -o/--output <path>")?;

    Ok(Args { input, to, output, json })
}

fn print_usage() {
    println!("usage: csvmd <input> --to csv|md -o <output> [--json]");
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
