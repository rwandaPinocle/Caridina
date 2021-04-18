use std::fmt;
use failure::{Error, Fail};
use itertools::iproduct;

extern crate serde;

const MAX_TILES: i64 = 7;

#[derive(Debug, Copy, Clone)]
enum Direction {
    Right, Down
}

struct Move {
    row: usize,
    col: usize,
    direction: Direction,
    word: String,
}

struct Shell {
    row: usize,
    col: usize,
    direction: Direction,
    text: String,
}

impl Shell {
    fn new(row: usize, col:usize, direction: Direction, text: String) -> Shell {
        Shell {
            row: row,
            col: col, 
            direction: direction,
            text: text,
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dir = match self.direction {
            Direction::Right => { "R" }
            Direction::Down => { "D" }
        };
        write!(f, "({},{}) {} {}", self.row, self.col, dir, self.text)
    }
}

struct Board {
    squares: Vec::<char>,
    w: usize,
    h: usize,
}

#[derive(Fail, Debug)]
#[fail(display = "There is an error: {}.", _0)]
struct BoardError(String);

impl Board {
    fn new(width: usize, height: usize) -> Board {
        let mut board = Board {
            squares: Vec::<char>::new(),
            w: width,
            h: height,
        };
        for _ in 0..(width*height) {
            board.squares.push('.');
        };
        board
    }

    fn make_move(&mut self, mv: Move) -> Result<(), BoardError>{
        let mut word_iter = mv.word.chars();
        let mut cur_row = mv.row;
        let mut cur_col = mv.col;
        loop {
            let idx = (cur_row * self.w) + cur_col;
            match mv.direction {
                Direction::Right => { cur_col += 1; }
                Direction::Down => { cur_row += 1; }
            };
            if self.squares[idx] != '.' {
                continue;
            };
            match word_iter.next() {
                Some(c) => { self.squares[idx] = c }
                _ => { break }
            };
            if cur_col > self.w || cur_row > self.h {
                return Err(BoardError("Not enough space for word".into()));
            }
        }
        Ok(())
    }

    fn find_shells(&self) -> Vec<Shell>{
        let mut shells = Vec::<Shell>::new();
        let itr = iproduct!(
            [Direction::Right, Direction::Down].iter(),
            (1..8),
            0..self.h-1,
            0..self.w-1
        );
        for (dir, len, row, col) in itr {
            let char_vec = match dir {
                Direction::Right => self.get_row(row),
                Direction::Down => self.get_col(col),
            };
            match dir {
                Direction::Right => {
                    let not_enough_space = col + len >= self.w;
                    let first_is_letter = char_vec[col] != '.';
                    let sec_is_letter = char_vec[col + 1] != '.';
                    if not_enough_space || (first_is_letter && sec_is_letter) { continue }
                }
                Direction::Down => {
                    let not_enough_space = row + len >= self.h;
                    let first_is_letter = char_vec[row] != '.';
                    let sec_is_letter = char_vec[row + 1] != '.';
                    if not_enough_space || (first_is_letter && sec_is_letter) { continue }
                }
            }
            let starting_idx = match dir {
                Direction::Right => col,
                Direction::Down => row,
            };
            let shell_str = Board::get_shell_from_vec(char_vec, len, starting_idx);
            if shell_str.replace('.', "").len() != 0 {
                let new_shell = Shell::new(row, col, *dir, shell_str);
                println!("{}", new_shell);
                shells.push(new_shell);
            }
        }
        shells
    }

    fn get_row(&self, row_idx: usize) -> Vec<char>{
        let mut row = Vec::<char>::new();
        for col_idx in 0..self.h {
            row.push(self.squares[row_idx * self.w + col_idx]);
        }
        row
    }

    fn get_col(&self, col_idx: usize) -> Vec<char>{
        let mut col = Vec::<char>::new();
        for row_idx in 0..self.h {
            col.push(self.squares[row_idx * self.w + col_idx]);
        }
        col
    }

    fn get_shell_from_vec(
        char_vec: Vec<char>,
        space_count: usize,
        starting_idx: usize
    ) -> String
    {
        let mut result = String::new();
        let mut spaces_used = 0;
        for (idx, &c) in char_vec.iter().enumerate() {
            if spaces_used == space_count && c == '.' { break }
            if idx >= starting_idx {
                result.push(c);
                if c == '.' { spaces_used += 1 }
            } else {
                match c {
                    '.' => { result = "".to_string() }
                    _ => { result.push(c) }
                }
            }
        }
        result
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        let mut count = 0;
        for c in self.squares.iter() {
            result.push(*c);
            result.push(' ');
            count += 1;
            if count == self.w {
                count = 0;
                result.push('\n');
            }
        }
        write!(f, "{}", result)
    }
}


fn main() {
    // Parse args
    let width: usize = 20;
    let height: usize = 20;

    // Board is in row major order
    // Origin is top left of board
    let mut board = Board::new(width, height);
    let mv = Move {
        word: "ABC".into(),
        row: 4,
        col: 4,
        direction: Direction::Right,
    };
    board.make_move(mv);
    println!("{}", board);

    let shells = board.find_shells();
    /*
    for shell in shells.iter() {
        println!("{}", shell);
    }
    */
}
