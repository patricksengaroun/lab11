// Loading and saving game data. Week 11: every failure gets a name (Tasks 3 to 8).
use crate::board::{Board, Cell, SIZE, new_board};
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;

pub const MIN_SHIP: usize = 2;
pub const MAX_SHIP: usize = 5;

#[derive(Debug, Clone, PartialEq)]
pub struct ShipSpec {
    pub name: String,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreRecord {
    pub name: String,
    pub score: u32,
}

// ---- Error types ----

#[derive(Debug, Clone, PartialEq)]
pub enum ShipLineError {
    MissingComma,
    EmptyName,
    BadLength(String),
    LengthOutOfRange(usize),
}

impl fmt::Display for ShipLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingComma => write!(f, "is missing a comma"),
            Self::EmptyName => write!(f, "has an empty name"),
            Self::BadLength(text) => write!(f, "'{text}' is not a valid number"),
            Self::LengthOutOfRange(value) => write!(f, "has length {value}, outside {MIN_SHIP}-{MAX_SHIP}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoardError {
    WrongRowCount(usize),
    WrongRowWidth { line: usize, found: usize },
    BadChar { line: usize, ch: char },
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongRowCount(count) => write!(
                f,
                "The board layout is invalid because it contains {count} rows, but exactly {SIZE} rows are required."
            ),
            Self::WrongRowWidth { line, found } => write!(
                f,
                "On file line {line}, the row width is {found} characters, but exactly {SIZE} characters are required."
            ),
            Self::BadChar { line, ch } => write!(
                f,
                "On line {line}, an invalid character '{ch}' was found; only '.' and 'S' are permitted."
            ),
        }
    }
}

#[derive(Debug)]
pub enum DataError {
    Io(io::Error),
    Board(BoardError),
    NoShips(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Io(e) => write!(f, "{e}"),
            DataError::Board(e) => write!(f, "bad board layout: {e}"),
            DataError::NoShips(path) => write!(f, "no valid ships in {path}"),
        }
    }
}

impl From<io::Error> for DataError {
    fn from(e: io::Error) -> DataError {
        DataError::Io(e)
    }
}

impl From<BoardError> for DataError {
    fn from(e: BoardError) -> DataError {
        DataError::Board(e)
    }
}

#[derive(Debug)]
pub struct FleetReport {
    pub fleet: Vec<ShipSpec>,
    pub skipped: Vec<(usize, ShipLineError)>,
}

pub fn parse_score(line: &str) -> Option<ScoreRecord> {
    let parts: Vec<&str> = line.trim().split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    let name = parts[0].trim();
    if name.is_empty() {
        return None;
    }
    match parts[1].trim().parse::<u32>() {
        Ok(score) => Some(ScoreRecord {
            name: name.to_string(),
            score,
        }),
        Err(_) => None,
    }
}

pub fn sort_scores_desc(records: &mut Vec<ScoreRecord>) {
    for i in 0..records.len() {
        let mut best = i;
        for j in (i + 1)..records.len() {
            if records[j].score > records[best].score {
                best = j;
            }
        }
        records.swap(i, best);
    }
}

pub fn parse_ship_line(line: &str) -> Result<ShipSpec, ShipLineError> {
    let trimmed = line.trim();

    let parts: Vec<&str> = trimmed.split(',').collect();

    if parts.len() != 2 {
        return Err(ShipLineError::MissingComma);
    }

    let name = parts[0].trim();
    let length_str = parts[1].trim();

    if name.is_empty() {
        return Err(ShipLineError::EmptyName);
    }

    let length = length_str
        .parse::<usize>()
        .map_err(|_| ShipLineError::BadLength(length_str.to_string()))?;

    if !(2..=5).contains(&length) {
        return Err(ShipLineError::LengthOutOfRange(length));
    }

    Ok(ShipSpec {
        name: name.to_string(),
        length,
    })
}

pub fn load_fleet(path: &str) -> Result<FleetReport, DataError> {
    let contents = fs::read_to_string(path)?;

    let mut fleet = Vec::new();
    let mut skipped = Vec::new();

    for (index, line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        match parse_ship_line(trimmed) {
            Ok(ship) => fleet.push(ship),
            Err(err) => skipped.push((line_number, err)),
        }
    }

    if fleet.is_empty() {
        return Err(DataError::NoShips(path.to_string()));
    }

    Ok(FleetReport { fleet, skipped })
}

pub fn parse_board(text: &str) -> Result<Board, BoardError> {
    let mut board = new_board();
    let mut row_count = 0;

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if row_count >= SIZE {
            row_count += 1;
            continue;
        }

        let chars: Vec<char> = trimmed.chars().collect();
        if chars.len() != SIZE {
            return Err(BoardError::WrongRowWidth {
                line: line_number,
                found: chars.len(),
            });
        }

        for col in 0..SIZE {
            match chars[col] {
                '.' => board[row_count][col] = Cell::Water,
                'S' => board[row_count][col] = Cell::Ship,
                ch => {
                    return Err(BoardError::BadChar {
                        line: line_number,
                        ch,
                    })
                }
            }
        }
        row_count += 1;
    }

    if row_count != SIZE {
        return Err(BoardError::WrongRowCount(row_count));
    }

    Ok(board)
}
  
pub fn load_board(path: &str) -> Result<Board, DataError> {
    let text = fs::read_to_string(path)?;
    let board = parse_board(&text)?;
    Ok(board)
}

pub fn load_scores(path: &str) -> Result<Vec<ScoreRecord>, DataError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(DataError::Io(e)),
    };

    let mut records = Vec::new();
    for line in text.lines() {
        if let Some(record) = parse_score(line) {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn append_score(path: &str, record: &ScoreRecord) -> Result<(), DataError> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let name = record.name.replace(',', " ");
    writeln!(file, "{},{}", name, record.score)?;
    Ok(())
}
