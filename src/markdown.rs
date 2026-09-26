use crate::table::{Alignment, Table};

// Parses a GitHub-flavored markdown pipe table: a header row, a
// `---`-style separator row, then zero or more data rows.
pub fn parse(input: &str) -> Result<(Table, Vec<String>), String> {
    let lines: Vec<&str> = input.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() < 2 {
        return Err("markdown input needs a header row and a separator row".to_string());
    }

    let headers = split_row(lines[0]);
    let separator_cells = split_row(lines[1]);
    if !is_separator(&separator_cells, headers.len()) {
        return Err(
            "second line of the markdown table must be a header separator, e.g. |---|---|"
                .to_string(),
        );
    }
    let alignments: Vec<Alignment> = separator_cells.iter().map(|c| parse_alignment(c)).collect();

    let width = headers.len();
    let mut rows = Vec::new();
    let mut warnings = Vec::new();

    for (i, line) in lines[2..].iter().enumerate() {
        let mut cells = split_row(line);
        if cells.len() != width {
            warnings.push(format!(
                "row {} has {} cell(s), expected {}; padded/truncated to fit",
                i + 1,
                cells.len(),
                width
            ));
        }
        cells.resize(width, String::new());
        cells.truncate(width);
        rows.push(cells);
    }

    let mut table = Table::new(headers, rows);
    table.alignments = alignments;
    Ok((table, warnings))
}

// Reads the alignment off a single separator cell, e.g. ":---" is left,
// "---:" is right, ":---:" is center, and a bare "---" is unspecified.
fn parse_alignment(cell: &str) -> Alignment {
    match (cell.starts_with(':'), cell.ends_with(':')) {
        (true, true) => Alignment::Center,
        (true, false) => Alignment::Left,
        (false, true) => Alignment::Right,
        (false, false) => Alignment::None,
    }
}

fn split_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('|').unwrap_or(trimmed);

    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut chars = trimmed.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'|') {
            cell.push('|');
            chars.next();
        } else if c == '|' {
            cells.push(cell.trim().to_string());
            cell = String::new();
        } else {
            cell.push(c);
        }
    }
    cells.push(cell.trim().to_string());
    cells
}

fn is_separator(cells: &[String], width: usize) -> bool {
    cells.len() == width
        && cells
            .iter()
            .all(|c| c.contains('-') && c.chars().all(|ch| ch == '-' || ch == ':'))
}

pub fn write(table: &Table) -> String {
    let widths = column_widths(table);
    let mut out = String::new();

    write_row(&mut out, &table.headers, &widths, &table.alignments);
    write_separator(&mut out, &widths, &table.alignments);
    for row in &table.rows {
        write_row(&mut out, row, &widths, &table.alignments);
    }

    out
}

fn write_separator(out: &mut String, widths: &[usize], alignments: &[Alignment]) {
    out.push('|');
    for (i, &width) in widths.iter().enumerate() {
        let align = alignments.get(i).copied().unwrap_or(Alignment::None);
        out.push(' ');
        out.push_str(&separator_marker(width, align));
        out.push(' ');
        out.push('|');
    }
    out.push('\n');
}

// widths are always at least 3 (see column_widths), so there's room for
// the colon(s) plus at least one dash in every case below.
fn separator_marker(width: usize, align: Alignment) -> String {
    match align {
        Alignment::None => "-".repeat(width),
        Alignment::Left => format!(":{}", "-".repeat(width - 1)),
        Alignment::Right => format!("{}:", "-".repeat(width - 1)),
        Alignment::Center => format!(":{}:", "-".repeat(width - 2)),
    }
}

fn column_widths(table: &Table) -> Vec<usize> {
    let mut widths: Vec<usize> = table.headers.iter().map(|h| h.chars().count()).collect();
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            let len = escape_cell(cell).chars().count();
            if len > widths[i] {
                widths[i] = len;
            }
        }
    }
    for w in widths.iter_mut() {
        *w = (*w).max(3);
    }
    widths
}

fn write_row(out: &mut String, cells: &[String], widths: &[usize], alignments: &[Alignment]) {
    out.push('|');
    for (i, cell) in cells.iter().enumerate() {
        let escaped = escape_cell(cell);
        let width = widths.get(i).copied().unwrap_or(escaped.chars().count());
        let align = alignments.get(i).copied().unwrap_or(Alignment::None);
        out.push(' ');
        out.push_str(&pad_cell(&escaped, width, align));
        out.push(' ');
        out.push('|');
    }
    out.push('\n');
}

fn pad_cell(content: &str, width: usize, align: Alignment) -> String {
    let pad = width.saturating_sub(content.chars().count());
    match align {
        Alignment::Right => format!("{}{}", " ".repeat(pad), content),
        Alignment::Center => {
            let left = pad / 2;
            let right = pad - left;
            format!("{}{}{}", " ".repeat(left), content, " ".repeat(right))
        }
        Alignment::Left | Alignment::None => format!("{}{}", content, " ".repeat(pad)),
    }
}

fn escape_cell(cell: &str) -> String {
    cell.replace('|', "\\|").replace('\n', " ").replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_table() {
        let input = "| a | b |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |\n";
        let (table, warnings) = parse(input).unwrap();
        assert_eq!(table.headers, vec!["a", "b"]);
        assert_eq!(table.rows, vec![vec!["1", "2"], vec!["3", "4"]]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn parses_table_without_outer_pipes() {
        let input = "a | b\n---|---\n1 | 2\n";
        let (table, _) = parse(input).unwrap();
        assert_eq!(table.headers, vec!["a", "b"]);
        assert_eq!(table.rows, vec![vec!["1", "2"]]);
    }

    #[test]
    fn parses_separator_with_alignment_colons() {
        let input = "| a | b |\n|:---|---:|\n| 1 | 2 |\n";
        let (table, _) = parse(input).unwrap();
        assert_eq!(table.headers, vec!["a", "b"]);
        assert_eq!(table.alignments, vec![Alignment::Left, Alignment::Right]);
    }

    #[test]
    fn parses_center_alignment_colons() {
        let input = "| a | b |\n|:---:|---|\n| 1 | 2 |\n";
        let (table, _) = parse(input).unwrap();
        assert_eq!(table.alignments, vec![Alignment::Center, Alignment::None]);
    }

    #[test]
    fn parses_escaped_pipe_in_cell() {
        let input = "| a |\n|---|\n| x \\| y |\n";
        let (table, _) = parse(input).unwrap();
        assert_eq!(table.rows, vec![vec!["x | y"]]);
    }

    #[test]
    fn skips_blank_lines() {
        let input = "| a | b |\n|---|---|\n\n| 1 | 2 |\n\n";
        let (table, _) = parse(input).unwrap();
        assert_eq!(table.rows, vec![vec!["1", "2"]]);
    }

    #[test]
    fn missing_separator_row_is_an_error() {
        let err = parse("| a | b |\n| 1 | 2 |\n").unwrap_err();
        assert!(err.contains("separator"));
    }

    #[test]
    fn too_few_lines_is_an_error() {
        let err = parse("| a | b |\n").unwrap_err();
        assert!(err.contains("header row and a separator row"));
    }

    #[test]
    fn short_row_is_padded_with_a_warning() {
        let input = "| a | b | c |\n|---|---|---|\n| 1 | 2 |\n";
        let (table, warnings) = parse(input).unwrap();
        assert_eq!(table.rows, vec![vec!["1", "2", ""]]);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn write_pads_columns_to_equal_width() {
        let table = Table::new(
            vec!["a".to_string(), "bb".to_string()],
            vec![vec!["1".to_string(), "22".to_string()]],
        );
        assert_eq!(write(&table), "| a   | bb  |\n| --- | --- |\n| 1   | 22  |\n");
    }

    #[test]
    fn write_escapes_pipes_and_strips_newlines() {
        let table = Table::new(vec!["a".to_string()], vec![vec!["x|y\nz".to_string()]]);
        assert_eq!(write(&table), "| a      |\n| ------ |\n| x\\|y z |\n");
    }

    #[test]
    fn write_emits_alignment_markers_and_pads_content() {
        let mut table = Table::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![vec!["1".to_string(), "22".to_string(), "333".to_string()]],
        );
        table.alignments = vec![Alignment::Left, Alignment::Right, Alignment::Center];
        assert_eq!(
            write(&table),
            "| a   |   b |  c  |\n| :-- | --: | :-: |\n| 1   |  22 | 333 |\n"
        );
    }

    #[test]
    fn alignment_round_trips_through_parse_and_write() {
        let input = "| a | b | c |\n|:---|---:|:---:|\n| 1 | 2 | 3 |\n";
        let (table, _) = parse(input).unwrap();
        let (table2, _) = parse(&write(&table)).unwrap();
        assert_eq!(table.alignments, table2.alignments);
    }

    #[test]
    fn round_trips_through_parse_and_write() {
        let input = "| a | b |\n|---|---|\n| 1 | 2 |\n";
        let (table, _) = parse(input).unwrap();
        let (table2, _) = parse(&write(&table)).unwrap();
        assert_eq!(table.headers, table2.headers);
        assert_eq!(table.rows, table2.rows);
    }
}
