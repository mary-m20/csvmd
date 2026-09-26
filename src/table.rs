#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}

pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    // Per-column markdown alignment, read from `:---`/`---:`/`:---:`
    // separator cells. CSV and TSV have no such concept, so parsers for
    // those formats leave every column as `Alignment::None`.
    pub alignments: Vec<Alignment>,
}

impl Table {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Table {
        let alignments = vec![Alignment::None; headers.len()];
        Table {
            headers,
            rows,
            alignments,
        }
    }
}
