use crate::table::Table;

// Parses a table out of RFC 4180 flavored CSV text. Returns the table plus
// any warnings about rows that didn't have the same number of fields as
// the header, rather than failing on them.
pub fn parse(input: &str) -> Result<(Table, Vec<String>), String> {
    let records = parse_records(input)?;
    if records.is_empty() {
        return Err("csv input has no rows".to_string());
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

    Ok((Table::new(headers, rows), warnings))
}

fn parse_records(input: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' => {
                    if chars.peek() == Some(&'"') {
                        field.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                }
                _ => field.push(c),
            }
            continue;
        }

        match c {
            '"' if field.is_empty() => in_quotes = true,
            ',' => record.push(std::mem::take(&mut field)),
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

    if in_quotes {
        return Err("unterminated quoted field in csv input".to_string());
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
            out.push(',');
        }
        out.push_str(&escape_field(f));
    }
    out.push('\n');
}

fn escape_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        let mut escaped = String::with_capacity(field.len() + 2);
        escaped.push('"');
        for c in field.chars() {
            if c == '"' {
                escaped.push('"');
            }
            escaped.push(c);
        }
        escaped.push('"');
        escaped
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_table() {
        let (table, warnings) = parse("a,b,c\n1,2,3\n4,5,6\n").unwrap();
        assert_eq!(table.headers, vec!["a", "b", "c"]);
        assert_eq!(table.rows, vec![vec!["1", "2", "3"], vec!["4", "5", "6"]]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn parses_without_trailing_newline() {
        let (table, _) = parse("a,b\n1,2").unwrap();
        assert_eq!(table.rows, vec![vec!["1", "2"]]);
    }

    #[test]
    fn parses_crlf_line_endings() {
        let (table, _) = parse("a,b\r\n1,2\r\n").unwrap();
        assert_eq!(table.headers, vec!["a", "b"]);
        assert_eq!(table.rows, vec![vec!["1", "2"]]);
    }

    #[test]
    fn parses_quoted_field_with_comma_and_newline() {
        let (table, _) = parse("a,b\n\"x,y\",\"line1\nline2\"\n").unwrap();
        assert_eq!(table.rows, vec![vec!["x,y", "line1\nline2"]]);
    }

    #[test]
    fn parses_escaped_double_quote() {
        let (table, _) = parse("a\n\"she said \"\"hi\"\"\"\n").unwrap();
        assert_eq!(table.rows, vec![vec!["she said \"hi\""]]);
    }

    #[test]
    fn unterminated_quote_is_an_error() {
        let err = parse("a\n\"unterminated\n").unwrap_err();
        assert!(err.contains("unterminated"));
    }

    #[test]
    fn empty_input_is_an_error() {
        let err = parse("").unwrap_err();
        assert!(err.contains("no rows"));
    }

    #[test]
    fn short_row_is_padded_with_a_warning() {
        let (table, warnings) = parse("a,b,c\n1,2\n").unwrap();
        assert_eq!(table.rows, vec![vec!["1", "2", ""]]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("2 field(s), expected 3"));
    }

    #[test]
    fn long_row_is_truncated_with_a_warning() {
        let (table, warnings) = parse("a,b\n1,2,3\n").unwrap();
        assert_eq!(table.rows, vec![vec!["1", "2"]]);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn write_round_trips_a_simple_table() {
        let (table, _) = parse("a,b\n1,2\n3,4\n").unwrap();
        assert_eq!(write(&table), "a,b\n1,2\n3,4\n");
    }

    #[test]
    fn write_quotes_fields_that_need_it() {
        let table = Table::new(
            vec!["a".to_string()],
            vec![vec!["has,comma".to_string()], vec!["has\"quote".to_string()]],
        );
        assert_eq!(write(&table), "a\n\"has,comma\"\n\"has\"\"quote\"\n");
    }

    #[test]
    fn write_leaves_plain_fields_unquoted() {
        let table = Table::new(vec!["a".to_string()], vec![vec!["plain".to_string()]]);
        assert_eq!(write(&table), "a\nplain\n");
    }
}
