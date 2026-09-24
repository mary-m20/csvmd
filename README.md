# csvmd

I keep needing to paste small CSV exports into markdown docs (READMEs, wiki
pages, PR descriptions) as tables, and every so often I need to go the other
way and pull a table back out of a doc into a CSV a spreadsheet can open.
Doing either by hand is fiddly the moment a cell has a comma or a pipe in it.

csvmd converts between CSV, TSV, and markdown pipe tables, in any
direction, from the command line.

## build

    cargo build --release

## usage

    csvmd [<input>] --to csv|tsv|md [-o <output>] [--from csv|tsv|md] [--json]

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

### stdin and stdout

Leave off `<input>` (or pass `-`) to read from stdin, and leave off
`-o`/`--output` (or pass `-`) to write to stdout:

    $ cat employees.csv | csvmd --from csv --to md
    | name  | role      |
    | ----- | --------- |
    | Ana   | engineer  |
    | Ravi  | designer  |

`--from` is required when reading from stdin, since there's no file
extension for csvmd to detect the format from. It's optional otherwise and
overrides the extension-based detection if given.

When the converted output goes to stdout, the summary/JSON report is
printed to stderr instead, so the stdout stream stays pipeable:

    $ cat employees.csv | csvmd --from csv --to md > employees.md
    converted - (csv) -> - (markdown)
    2 rows, 2 columns
    no warnings

## format notes

- CSV parsing follows RFC 4180: fields containing a comma, a quote, or a
  newline must be wrapped in double quotes, and a literal quote inside a
  quoted field is written as two double quotes (`""`).
- TSV fields are tab-separated and unquoted; a literal tab, newline,
  carriage return, or backslash in a field is backslash-escaped
  (`\t`, `\n`, `\r`, `\\`), the convention used by MySQL/Postgres
  tab-delimited dumps.
- Markdown tables use the standard pipe syntax with a `---` header
  separator row. A literal `|` inside a cell is escaped as `\|`.
- The input file's extension decides how it's read (`.csv`, `.tsv`, or
  `.md` / `.markdown`); the `--to` flag decides how it's written.
- If a data row has a different number of fields than the header row,
  csvmd pads or truncates it to fit and reports it as a warning rather
  than failing the whole conversion.

## status

Early. Handles three formats (CSV, TSV, markdown) but no column
alignment markers on markdown tables yet, and no `--delimiter` flag for
CSV variants that don't use a comma. Adding things as I actually need
them.
