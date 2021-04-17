use std::fmt;
use failure::Error;
extern crate serde;

#[macro_use] extern crate failure;


enum Direction {
    Right, Down
}

struct Move {
    word: String,
    row: usize,
    col: usize,
    direction: Direction,
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
    let width: usize = 15;
    let height: usize = 15;

    // Board is in row major order
    // Origin is top left of board
    let mut board = Board::new(width, height);
    let mv = Move {
        word: "HELLOWORLD".into(),
        row: 0,
        col: 0,
        direction: Direction::Right,
    };
    board.make_move(mv);

    println!("{}", board);

    let mv = Move {
        word: "HELLOWORLD".into(),
        row: 0,
        col: 3,
        direction: Direction::Down,
    };
    board.make_move(mv);


    println!("{}", board);
}
