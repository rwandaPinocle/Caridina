use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Serialize, Deserialize)]
pub struct MoveRequest {
	// Words must be spelled in increasing i or j indices on the board.
	pub board: Vec<Vec<Tile>>,
	pub tilesLeft: usize,
	pub players: Vec<PlayerInfo>,
	pub letters: Vec<char>, // Which letters you have in your hand
	pub playerIndex: usize,  // Which player you are in the players array
}

#[derive(Serialize, Deserialize)]
pub struct PlayerInfo {
	pub tilesInHand: usize,
	pub points: usize,
}

#[derive(Serialize, Deserialize)]
pub struct MoveResponse {
	pub bank: Vec<usize>, // List of letters in your hand you want to exchange. If there is anything here your turn will be skipped
							// and you'll get your new tiles on your next turn.
	pub board: Vec<Vec<Tile>>, // Make any changes you want to make to the board and send back the adjusted board.
							// I will ensure you've made a legal move and add up your points. If you make an invalid
							// move your turn will be effectively skipped.
}

#[derive(Serialize, Deserialize)]
pub struct InitializeRequest {
	pub board: Vec<Vec<Tile>>,
	pub words: Vec<String>,
	pub letters: HashMap<String, Letter>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tile {
	pub square: String, // " ", "dl", "tl", "dw", "tw", "st"
	pub letter: String, // "a", "b", "" if empty
}

#[derive(Serialize, Deserialize)]
pub struct Letter {
	pub value: usize,
	pub n: usize,
	pub left: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ScrabbleWord {
	pub start: Vec<usize>,
	pub end: Vec<usize>,
	pub word: String,
}
