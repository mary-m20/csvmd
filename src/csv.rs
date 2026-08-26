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

    Ok((Table { headers, rows }, warnings))
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
