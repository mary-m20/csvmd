use crate::table::Table;

// Parses tab-separated text. Unlike CSV, TSV fields aren't quoted: a
// literal tab, newline, carriage return, or backslash in a field is
// backslash-escaped instead (the convention used by MySQL/Postgres
// tab-delimited dumps).
pub fn parse(input: &str) -> Result<(Table, Vec<String>), String> {
    let records = parse_records(input)?;
    if records.is_empty() {
        return Err("tsv input has no rows".to_string());
    }

    let headers = records[0].clone();
    let width = headers.len();
    let mut rows = Vec::new();
    let mut warnings = Vec::new();

    for (i, mut record) in records.into_iter().enumerate().skip(1) {
        if record.len() != width {
            warnings.push(format!(
                "row {} has {} field(s), expected {}; padded/truncated to fit",
                i,
                record.len(),
                width
            ));
        }
        record.resize(width, String::new());
        record.truncate(width);
        rows.push(record);
    }

    Ok((Table { headers, rows }, warnings))
}

fn parse_records(input: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('t') => field.push('\t'),
                Some('n') => field.push('\n'),
                Some('r') => field.push('\r'),
                Some('\\') => field.push('\\'),
                Some(other) => {
                    field.push('\\');
                    field.push(other);
                }
                None => return Err("tsv input ends with a trailing backslash".to_string()),
            },
            '\t' => record.push(std::mem::take(&mut field)),
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
            }
            '\n' => {
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
            }
            _ => field.push(c),
        }
    }

    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }

    Ok(records)
}

pub fn write(table: &Table) -> String {
    let mut out = String::new();
    write_record(&mut out, &table.headers);
    for row in &table.rows {
        write_record(&mut out, row);
    }
    out
}

fn write_record(out: &mut String, fields: &[String]) {
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            out.push('\t');
        }
        out.push_str(&escape_field(f));
    }
    out.push('\n');
}

fn escape_field(field: &str) -> String {
    let mut escaped = String::with_capacity(field.len());
    for c in field.chars() {
        match c {
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\\' => escaped.push_str("\\\\"),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_table() {
        let (table, warnings) = parse("a\tb\tc\n1\t2\t3\n4\t5\t6\n").unwrap();
        assert_eq!(table.headers, vec!["a", "b", "c"]);
        assert_eq!(table.rows, vec![vec!["1", "2", "3"], vec!["4", "5", "6"]]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn parses_without_trailing_newline() {
        let (table, _) = parse("a\tb\n1\t2").unwrap();
        assert_eq!(table.rows, vec![vec!["1", "2"]]);
    }

    #[test]
    fn parses_crlf_line_endings() {
        let (table, _) = parse("a\tb\r\n1\t2\r\n").unwrap();
        assert_eq!(table.headers, vec!["a", "b"]);
        assert_eq!(table.rows, vec![vec!["1", "2"]]);
    }

    #[test]
    fn parses_escaped_tab_and_newline_in_field() {
        let (table, _) = parse("a\n1\\t2\\n3\n").unwrap();
        assert_eq!(table.rows, vec![vec!["1\t2\n3"]]);
    }

    #[test]
    fn parses_escaped_backslash() {
        let (table, _) = parse("a\nc:\\\\path\n").unwrap();
        assert_eq!(table.rows, vec![vec!["c:\\path"]]);
    }

    #[test]
    fn trailing_backslash_is_an_error() {
        let err = parse("a\nfoo\\").unwrap_err();
        assert!(err.contains("trailing backslash"));
    }

    #[test]
    fn empty_input_is_an_error() {
        let err = parse("").unwrap_err();
        assert!(err.contains("no rows"));
    }

    #[test]
    fn short_row_is_padded_with_a_warning() {
        let (table, warnings) = parse("a\tb\tc\n1\t2\n").unwrap();
        assert_eq!(table.rows, vec![vec!["1", "2", ""]]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("2 field(s), expected 3"));
    }

    #[test]
    fn long_row_is_truncated_with_a_warning() {
        let (table, warnings) = parse("a\tb\n1\t2\t3\n").unwrap();
        assert_eq!(table.rows, vec![vec!["1", "2"]]);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn write_round_trips_a_simple_table() {
        let (table, _) = parse("a\tb\n1\t2\n3\t4\n").unwrap();
        assert_eq!(write(&table), "a\tb\n1\t2\n3\t4\n");
    }

    #[test]
    fn write_escapes_special_characters() {
        let table = Table {
            headers: vec!["a".to_string()],
            rows: vec![
                vec!["has\ttab".to_string()],
                vec!["has\nnewline".to_string()],
                vec!["has\\backslash".to_string()],
            ],
        };
        assert_eq!(
            write(&table),
            "a\nhas\\ttab\nhas\\nnewline\nhas\\\\backslash\n"
        );
    }

    #[test]
    fn round_trips_through_parse_and_write() {
        let input = "a\tb\n1\ttab\\there\n";
        let (table, _) = parse(input).unwrap();
        let (table2, _) = parse(&write(&table)).unwrap();
        assert_eq!(table.headers, table2.headers);
        assert_eq!(table.rows, table2.rows);
    }
}
