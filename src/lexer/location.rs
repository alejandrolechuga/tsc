pub struct Location {
    pub column: u32,
    pub line: u32,
}

impl Location {
    pub fn new(line: u32, column: u32) -> Location {
        Location { line, column }
    }
}
