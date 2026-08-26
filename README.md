# csvmd

I keep needing to paste small CSV exports into markdown docs (READMEs, wiki
pages, PR descriptions) as tables, and every so often I need to go the other
way and pull a table back out of a doc into a CSV a spreadsheet can open.
Doing either by hand is fiddly the moment a cell has a comma or a pipe in it.

csvmd converts between CSV and markdown pipe tables, in both directions,
from the command line.

## build

    cargo build --release

## usage

    csvmd <input> --to csv|md -o <output> [--json]

Examples:

    csvmd employees.csv --to md -o employees.md
    csvmd notes.md --to csv -o notes.csv

By default csvmd prints a short human-readable summary of the conversion
after it runs:

    $ csvmd employees.csv --to md -o employees.md
    converted employees.csv (csv) -> employees.md (markdown)
    6 rows, 3 columns
    no warnings

Pass `--json` to get the same information as a single JSON object, for
piping into another script:

    $ csvmd employees.csv --to md -o employees.md --json
    {"input_path":"employees.csv","input_format":"csv","output_path":"employees.md","output_format":"markdown","rows":6,"columns":3,"warnings":[]}

## format notes

- CSV parsing follows RFC 4180: fields containing a comma, a quote, or a
  newline must be wrapped in double quotes, and a literal quote inside a
  quoted field is written as two double quotes (`""`).
- Markdown tables use the standard pipe syntax with a `---` header
  separator row. A literal `|` inside a cell is escaped as `\|`.
- The input file's extension decides how it's read (`.csv`, or `.md` /
  `.markdown`); the `--to` flag decides how it's written.
- If a data row has a different number of fields than the header row,
  csvmd pads or truncates it to fit and reports it as a warning rather
  than failing the whole conversion.

## status

Early. Handles exactly two formats and no column alignment markers on
markdown tables yet, no TSV support, no stdin/stdout streaming. Adding
things as I actually need them.
