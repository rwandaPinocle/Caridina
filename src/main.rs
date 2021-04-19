use std::fmt;
use std::fs;
use std::collections::HashMap;
use failure::{ Error, Fail };
use itertools::{ iproduct, sorted, enumerate, Itertools };

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
    mask: String,
}

impl Move {
    fn new(row: usize, col: usize, direction: Direction, word: String, mask: String) -> Move {
        Move {row, col, direction, word, mask}
    }
}

struct Shell {
    row: usize,
    col: usize,
    direction: Direction,
    text: String,
}

impl Shell {
    fn new(row: usize, col:usize, direction: Direction, text: String) -> Shell {
        Shell { row, col, direction, text }
    }

    fn spaces(&self) -> usize {
        let mut total = 0;
        for c in self.text.chars() {
            if c == '.' {
                total += 1;
            }
        }
        total
    }

    fn letters(&self) -> Vec<char> {
        let mut chars = Vec::new();
        for c in self.text.chars() {
            if c != '.' {
                chars.push(c);
            }
        }
        chars
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

    fn from(board: &Board) -> Board {
        Board {
            squares: board.squares.to_vec(),
            w: board.w,
            h: board.h,
        }
    }

    fn make_move(&mut self, mv: &Move) -> Result<Board, BoardError>{
        let mut letters = mv.word
            .chars()
            .zip(mv.mask.chars())
            .filter(|(_, mc)| {*mc == '.'})
            .map(|(wc, _)| {wc});
        let mut cur_row = mv.row;
        let mut cur_col = mv.col;
        let mut new_board = Board::from(self);
        let mut c = match letters.next() {
            Some(c) => c,
            None => '?',
        };
        loop {
            let idx = (cur_row * new_board.w) + cur_col;
            match mv.direction {
                Direction::Right => { cur_col += 1; }
                Direction::Down => { cur_row += 1; }
            };
            if new_board.squares[idx] != '.' {
                continue;
            };
            new_board.squares[idx] = c;
            c = match letters.next() {
                Some(c) => c,
                None => break,
            };
            
            if cur_col > new_board.w || cur_row > new_board.h {
                return Err(BoardError("Not enough space for word".into()));
            }
        }
        Ok(new_board)
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
            println!("{}", result);
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


#[derive(Hash, Eq, PartialEq)]
struct LetterPlace {
    idx: usize,
    letter: char,
}

impl fmt::Display for LetterPlace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {}]", self.idx, self.letter)
    }
}


fn build_letter_place_map(words: &Vec<String>) -> HashMap<LetterPlace, Vec<String>> {
    let mut map = HashMap::<LetterPlace, Vec<String>>::new();
    for word in words {
        for (idx, letter) in word.chars().enumerate() {
            let lp = LetterPlace {idx, letter};

            match map.get_mut(&lp) {
                Some(vec) => { vec.push(word.clone()); },
                None => { map.insert(lp, vec![word.clone()]); },
            };
        }
    }
    map
}


fn build_letters_word_map(words: &Vec<String>) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::<String, Vec<String>>::new();
    for word in words {
        let key: String = sorted(word.chars()).collect::<String>();
        match map.get_mut(&key) {
            Some(vec) => { vec.push(word.clone()); },
            None => { map.insert(key, vec![word.clone()]); },
        };
    }
    map
}


fn main() {
    // Create wordlist
    let word_file = fs::read_to_string("Collins Scrabble Words (2019).txt") .expect("Something went wrong reading the file");
    let word_list: Vec<String> = word_file.lines().map(|x: &str| {x.to_string()}).collect();
    let letter_place_map = build_letter_place_map(&word_list);
    let letters_word_map = build_letters_word_map(&word_list);

    let mut total: f64 = 0.0;
    for (idx, (key, value)) in letters_word_map.iter().enumerate() {
        total += value.len() as f64;
    }

    let width: usize = 7;
    let height: usize = 7;
    let tiles = vec!['W', 'O', 'R', 'L', 'D'];

    // Board is in row major order
    // Origin is top left of board
    let mut board = Board::new(width, height);
    let mv = Move {
        word: "HELLO".into(),
        row: 0,
        col: 0,
        direction: Direction::Right,
        mask: ".....".into()
    };
    board = match board.make_move(&mv) {
        Ok(new_board) => new_board,
        Error => board
    };

    // TODO: Eliminate duplicates
    let shells = board.find_shells();

    let mut moves: Vec<Move> = Vec::new();
    for shell in shells.iter() {
        let space_ct = shell.spaces();
        for combo in tiles.iter().combinations(space_ct) {
            let mut letters = Vec::new();
            for &c in combo { letters.push(c); }
            for l in shell.letters() { letters.push(l); }
            let key = letters.iter().sorted().join("");
            //println!("Key: {}", key);
            let words = match letters_word_map.get(&key) {
                Some(vec) => vec,
                None => continue,
            };

            for word in words {
                let mut valid = true;
                for (wc, sc) in word.chars().zip(shell.text.chars()) {
                    valid = valid && !(sc != '.' && wc != sc);
                }
                if !valid { continue }
                let mv = Move::new(
                    shell.row,
                    shell.col,
                    shell.direction,
                    word.clone(),
                    shell.text.clone());
                println!("r:{} c:{} {} {}", mv.row, mv.col, mv.word, mv.mask);
                match board.make_move(&mv) {
                    Ok(board) => {println!("{}", board)},
                    Error => {}
                }
                moves.push(mv);
            }
        }
    }
}
