pub struct Report {
    pub input_path: String,
    pub input_format: String,
    pub output_path: String,
    pub output_format: String,
    pub rows: usize,
    pub columns: usize,
    pub warnings: Vec<String>,
}

impl Report {
    pub fn to_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "converted {} ({}) -> {} ({})\n",
            self.input_path, self.input_format, self.output_path, self.output_format
        ));
        out.push_str(&format!("{} rows, {} columns\n", self.rows, self.columns));
        if self.warnings.is_empty() {
            out.push_str("no warnings\n");
        } else {
            out.push_str("warnings:\n");
            for w in &self.warnings {
                out.push_str(&format!("  - {}\n", w));
            }
        }
        out
    }

    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push('{');
        out.push_str(&format!("\"input_path\":{},", json_string(&self.input_path)));
        out.push_str(&format!("\"input_format\":{},", json_string(&self.input_format)));
        out.push_str(&format!("\"output_path\":{},", json_string(&self.output_path)));
        out.push_str(&format!(
            "\"output_format\":{},",
            json_string(&self.output_format)
        ));
        out.push_str(&format!("\"rows\":{},", self.rows));
        out.push_str(&format!("\"columns\":{},", self.columns));
        out.push_str("\"warnings\":[");
        for (i, w) in self.warnings.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&json_string(w));
        }
        out.push_str("]}\n");
        out
    }
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
