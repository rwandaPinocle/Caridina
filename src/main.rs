#![feature(proc_macro_hygiene, decl_macro)]
use std::fmt;
use std::fs;
use std::collections::HashMap;
use rand::Rng;
use failure::{ Error, Fail };
use itertools::{ iproduct, sorted, enumerate, Itertools };
use rocket::{ get, ignite };



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
    mults: Vec::<char>,
}

#[derive(Fail, Debug)]
#[fail(display = "There is an error: {}.", _0)]
struct BoardError(String);

impl Board {
    fn new(width: usize, height: usize, score_str: String) -> Board {
        let mut board = Board {
            squares: Vec::<char>::new(),
            w: width,
            h: height,
            mults: score_str.chars().collect(),
        };
        for _ in 0..(width*height) {
            board.squares.push('.');
        };
        board
    }

    fn clone(&self) -> Board {
        let mut board = Board {
            squares: self.squares.to_vec(),
            w: self.w,
            h: self.h,
            mults: self.mults.to_vec(),
        };
        board
    }

    fn from(board: &Board) -> Board {
        Board {
            squares: board.squares.to_vec(),
            w: board.w,
            h: board.h,
            mults: board.mults.to_vec(),
        }
    }


    fn is_empty(&self) -> bool{
        let mut result = true;
        for &c in self.squares.iter() {
            if c != '.' {
                result = false;
                break;
            }
        }
        result
    }

    fn make_move(
        &self,
        mv: &Move,
        letter_values: &HashMap<char, usize>
    ) -> Result<(Board, usize), BoardError> {

        let mut letters_to_place = mv.word
            .chars()
            .zip(mv.mask.chars())
            .filter(|(_, mc)| {*mc == '.'})
            .map(|(wc, _)| {wc});
        let mut cur_row = mv.row;
        let mut cur_col = mv.col;

        let mut move_score = 0;
        let mut multiplier = 1;

        let mut new_board = Board::from(self);
        let mut c = match letters_to_place.next() {
            Some(c) => c,
            None => '?',
        };
        loop {
            let idx = (cur_row * new_board.w) + cur_col;
            match mv.direction {
                Direction::Right => { cur_col += 1; }
                Direction::Down => { cur_row += 1; }
            };
            if new_board.squares[idx] == '.' {
                match self.mults[idx] {
                    'w' => { multiplier *= 2 }
                    'W' => { multiplier *= 3 }
                    'l' => { move_score +=  letter_values[&c] * 2}
                    'L' => { move_score +=  letter_values[&c] * 3}
                    '-' => { move_score += letter_values[&c] }
                    _ => {}
                }
            } else {
                move_score += letter_values[&c];
                continue;
            }
            new_board.squares[idx] = c;
            c = match letters_to_place.next() {
                Some(c) => c,
                None => break,
            };
            
            if cur_col > new_board.w || cur_row > new_board.h {
                return Err(BoardError("Not enough space for word".into()));
            }
        }
        move_score *= multiplier;
        Ok((new_board, move_score))
    }

    fn affected_rows_cols(&self, mv: &Move) -> (Vec<usize>, Vec<usize>) {
        // first return value is row idxs, second is col idxs
        let mut letters = mv.word
            .chars()
            .zip(mv.mask.chars())
            .filter(|(_, mc)| {*mc == '.'})
            .map(|(wc, _)| {wc});
        let mut cur_row = mv.row;
        let mut cur_col = mv.col;
        let mut row_idxs = Vec::new();
        let mut col_idxs = Vec::new();
        let mut c = match letters.next() {
            Some(c) => c,
            None => '?',
        };
        loop {
            let idx = (cur_row * self.w) + cur_col;
            if self.squares[idx] != '.' {
                match mv.direction {
                    Direction::Right => { cur_col += 1; }
                    Direction::Down => { cur_row += 1; }
                };
                continue;
            };
            row_idxs.push(cur_row);
            col_idxs.push(cur_col);
            //new_board.squares[idx] = c;
            c = match letters.next() {
                Some(c) => c,
                None => break,
            };
            
            if cur_col > self.w || cur_row > self.h {
                continue;
            }

            match mv.direction {
                Direction::Right => { cur_col += 1; }
                Direction::Down => { cur_row += 1; }
            };
        }
        (row_idxs, col_idxs)
    }

    fn find_shells(&self) -> Vec<Shell>{
        let mut shells = Vec::<Shell>::new();
        let itr = iproduct!(
            [Direction::Right, Direction::Down].iter(),
            (1..8),
            0..self.h-1,
            0..self.w-1
        );
        if self.is_empty() {
            return self.get_initial_shells();
        }
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
            if starting_idx != 0 && char_vec[starting_idx - 1] != '.'{
                continue
            }
            let shell_str = Board::get_shell_from_vec(char_vec, len, starting_idx);
            if shell_str.replace('.', "").len() != 0 {
                let new_shell = Shell::new(row, col, *dir, shell_str);
                shells.push(new_shell);
            }
        }
        shells
    }

    fn get_initial_shells(&self) -> Vec<Shell>{
        let mut shells: Vec<Shell> = Vec::new();
        for s_count in 2..8 {
            for offset in 0..s_count {
                let mid_row: usize = (self.h - self.h % 2)/2;
                let mid_col: usize = (self.w - self.w % 2)/2;
                if (mid_row >= offset) && (mid_row-offset + s_count <= self.h) {
                    let dshell = Shell::new(mid_row-offset, mid_col, Direction::Down, ".".repeat(s_count));
                    shells.push(dshell);
                }
                if (mid_col >= offset) && (mid_col-offset + s_count <= self.w){
                    let rshell = Shell::new(mid_row, mid_col-offset, Direction::Right, ".".repeat(s_count));
                    shells.push(rshell);
                }
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

    fn get_mult_row(&self, row_idx: usize) -> Vec<char>{
        let mut row = Vec::<char>::new();
        for col_idx in 0..self.h {
            row.push(self.mults[row_idx * self.w + col_idx]);
        }
        row
    }

    fn get_mult_col(&self, col_idx: usize) -> Vec<char>{
        let mut col = Vec::<char>::new();
        for row_idx in 0..self.h {
            col.push(self.mults[row_idx * self.w + col_idx]);
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
                if c == '.' {
                    spaces_used += 1;
                }
            } else {
                if c == '.' {
                    result = "".to_string();
                } else {
                    result.push(c);
                }
            }
        }
        result
    }

    fn is_legal(
        &self,
        rows: &Vec<usize>,
        cols: &Vec<usize>,
        letters_word_map: &HashMap<String, Vec<String>>
    ) -> bool
    {
        for &r_idx in rows {
            let row = self.get_row(r_idx);
            if row.iter().filter(|&c| *c != '.').count() == 0 { continue }

            for word in row.iter().join("").split('.') {
                if word.len() <= 1 { continue }
                let sorted = word.chars().sorted().join("");

                if let Some(word_vec) = letters_word_map.get(&sorted) {
					if !word_vec.iter().any(|w| w == word) {
						return false;
					}
                } else {
					return false;
				}
            }
        }
        for &c_idx in cols {
            let col = self.get_col(c_idx);
            if col.iter().filter(|&c| *c != '.').count() == 0 { continue }

            for word in col.iter().join("").split('.') {
                if word.len() <= 1 { continue }
                let sorted = word.chars().sorted().join("");

                if let Some(word_vec) = letters_word_map.get(&sorted) {
					if !word_vec.iter().any(|w| w == word) {
						return false;
					}
                } else {
					return false;
				}
            }
        }
        true
    }

    fn get_moves(
        &self,
        tiles: &Vec<char>,
        letters_word_map: &HashMap<String,Vec<String>>
    ) -> Vec<Move> {
        let shells = self.find_shells();

        let mut moves: Vec<Move> = Vec::new();
        for shell in shells.iter() {
            let space_ct = shell.spaces();
            for combo in tiles.iter().combinations(space_ct) {
                let mut letters = Vec::new();
                for &c in combo { letters.push(c); }
                for l in shell.letters() { letters.push(l); }
                let key = letters.iter().sorted().join("");
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
                    moves.push(mv);
                }
            }
        }
        moves
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
            if let Some(vec) = map.get_mut(&lp) {
                vec.push(word.clone());
            } else {
                map.insert(lp, vec![word.clone()]);
            };
        }
    }
    map
}


fn build_letters_word_map(words: &Vec<String>) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::<String, Vec<String>>::new();
    for word in words {
        let key: String = sorted(word.chars()).collect::<String>();
        if let Some(vec) = map.get_mut(&key) {
            vec.push(word.clone());
        } else {
            map.insert(key, vec![word.clone()]);
        };
    }
    map
}

fn build_letter_values() -> HashMap<char, usize>{
   let mut ltr_values = HashMap::new();
   let mut rng = rand::thread_rng();
   for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
       ltr_values.insert(c, rng.gen_range(0..10));
   }
   ltr_values
}


fn build_score_string(width: usize, height: usize) -> String {
    let mut result = String::new();
    let values = ['-', 'w', 'W', 'l', 'L'];
    let mut rng = rand::thread_rng();

    for _ in 0..width*height {
        let c = values[rng.gen_range(0..values.len())];
        result.push(c);
    }
    result
}

#[get("/world")]
fn world() -> &'static str {
    "hello, world!"
}


fn main() {
    let width: usize = 11;
    let height: usize = 11;
    let tiles = "ANDREWPO".chars().collect();

    // Create wordlist
    let word_file = fs::read_to_string("Collins Scrabble Words (2019).txt").expect("Something went wrong reading the file");
    let word_list: Vec<String> = word_file.lines().map(|x: &str| {x.to_string()}).collect();
    let letter_place_map = build_letter_place_map(&word_list);
    let letters_word_map = build_letters_word_map(&word_list);
    let letter_values = build_letter_values();
    let score_str = build_score_string(width, height);


    let mut total: f64 = 0.0;
    for (idx, (key, value)) in letters_word_map.iter().enumerate() {
        total += value.len() as f64;
    }

    // Board is in row major order
    // Origin is top left of board
    let mut board = Board::new(width, height, score_str);

    loop {
        let mut best_score = 0;
        let mut best_board = board.clone();
        let moves = board.get_moves(&tiles, &letters_word_map);
        for mv in moves.iter() {
            if let Ok((new_board, move_score)) = board.make_move(&mv, &letter_values) {
                let (rows, cols) = board.affected_rows_cols(&mv);
                let is_legal = new_board.is_legal(&rows, &cols, &letters_word_map);
                //println!("{} {}", new_board, is_legal);
                if is_legal && best_score < move_score {
                    best_score = move_score;
                    best_board = new_board;
                }
            }
        }
        board = best_board;
        if best_score == 0 { break }
        println!("{}", board);
    }
}
