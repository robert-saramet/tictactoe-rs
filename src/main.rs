use console::Term;
use dialoguer::Select;
use rand::Rng;
use std::fmt::{Display, Formatter, Result};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

enum Keymap {
    Standard,
    Numpad,
}

enum Opponent {
    Computer,
    Human,
}

enum Difficulty {
    Easy, // random choice
    Hard, // minmax algo
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Mark {
    X,
    O,
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Cell {
    empty: bool,
    mark: Mark,
}
type Player = Cell;

enum State {
    Ongoing,
    Won(Mark),
    Tie,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            empty: true,
            mark: Mark::X,
        }
    }
}

impl std::ops::Not for Mark {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            grid: Grid::default(),
            state: State::Ongoing,
            player: Mark::X,
            turn: Mark::X,
            opponent: Opponent::Computer,
            difficulty: Some(Difficulty::Easy),
            keymap: Keymap::Standard,
            score: 0,
        }
    }
}

impl Display for Mark {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::X => write!(f, "X"),
            Self::O => write!(f, "O"),
        }
    }
}

// TODO: Possibly render the table more fancifully
impl Display for Cell {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if self.empty {
            write!(f, "□")
        } else {
            write!(f, "{}", self.mark)
        }
    }
}

type Grid = [[Cell; 3]; 3];

struct Game {
    grid: Grid,
    state: State,
    player: Mark,
    turn: Mark,
    opponent: Opponent,
    difficulty: Option<Difficulty>,
    keymap: Keymap,
    score: u32,
}

impl Game {
    fn setup() -> Self {
        println!("Welcome to tic-tac-toe");

        // TODO: Consider refactoring to use other data structures

        const OPPONENT_PROMPT: &str = "Play against";
        const OPPONENTS: [&str; 2] = ["the computer", "another human"];
        const DIFFICULTY_PROMPT: &str = "Difficulty";
        const DIFFICULTIES: [&str; 2] = ["easy", "hard"];
        const PLAYER_PROMPT: &str = "Player";
        const PLAYERS: [&str; 2] = ["X", "O"];
        const KEYMAP_PROMPT: &str = "Keymap";
        const KEYMAPS: [&str; 2] = [
            "standard - top-left (1) to bottom-right (9)",
            "numpad - bottom-right (1) to top-left (9)",
        ];
        const PROMPTS: [&str; 4] = [
            OPPONENT_PROMPT,
            DIFFICULTY_PROMPT,
            PLAYER_PROMPT,
            KEYMAP_PROMPT,
        ];
        const OPTIONS: [[&str; 2]; 4] = [OPPONENTS, DIFFICULTIES, PLAYERS, KEYMAPS];

        let mut choices = [0, 0, 0, 0];
        for ((i, prompt), items) in PROMPTS.iter().enumerate().zip(OPTIONS.iter()) {
            // Don't prompt for difficulty when playing against another human
            if i == 1 && choices[0] == 1 {
                continue;
            }
            choices[i] = Select::new()
                .with_prompt(prompt.to_owned())
                .items(items)
                .default(0)
                .interact()
                .unwrap();
        }

        let grid = Grid::default();
        let state = State::Ongoing;
        let player = if choices[2] == 0 { Mark::X } else { Mark::O };
        let turn = Mark::X;
        let opponent = if choices[0] == 1 {
            Opponent::Human
        } else {
            Opponent::Computer
        };
        let difficulty = if choices[0] == 1 {
            if choices[1] == 0 {
                Some(Difficulty::Easy)
            } else {
                Some(Difficulty::Hard)
            }
        } else {
            None
        };
        let keymap = if choices[3] == 0 {
            Keymap::Standard
        } else {
            Keymap::Numpad
        };
        Game {
            grid,
            state,
            player,
            turn,
            opponent,
            difficulty,
            keymap,
            score: 0,
        }
    }
    // TODO: Highlight winning sequence
    // TODO: Handle tie conditions
    fn check_winner(&self) -> Option<Mark> {
        let grid = &self.grid;
        for i in 0..3 {
            // Check if all cells in a row are the same and not empty
            if grid[i]
                .iter()
                .all(|&cell| cell == grid[i][0] && !cell.empty)
            {
                return Some(grid[i][0].mark);
            }
            // Check if all cells in a column are the same and not empty
            if grid
                .iter()
                .map(|row| row[i])
                .all(|cell| cell == grid[0][i] && !cell.empty)
            {
                return Some(grid[0][i].mark);
            }
        }
        // Scan diagonals
        // Check if all cells in the primary diagonal are the same and not empty
        if grid
            .iter()
            .enumerate()
            .map(|(i, row)| row[i])
            .all(|cell| cell == grid[0][0] && !cell.empty)
        {
            return Some(grid[0][0].mark);
        }
        // Check if all cells in the secondary diagonal are the same and not empty
        if grid
            .iter()
            .enumerate()
            .map(|(i, row)| row[2 - i])
            .all(|cell| cell == grid[0][2] && !cell.empty)
        {
            return Some(grid[0][2].mark);
        }
        None
    }

    fn play_turn(_grid: &mut Grid, _player: Cell) {}

    fn play_turn_human(&self) -> (usize, usize) {
        let term = Term::stdout();
        println!("Select a cell from 1 to 9");
        // TODO: Handle invalid inputs
        let choice = term.read_char().unwrap().to_digit(10).unwrap() as usize;
        // TODO: Make this DRYer
        let row = if let Keymap::Standard = self.keymap {
            (choice - 1) / 3
        } else {
            (9 - choice) / 3
        };
        let col = (choice - 1) % 3;
        (row, col)
    }

    fn play_turn_computer_random(&self) -> (usize, usize) {
        // Search
        todo!()
    }

    fn play_turn_computer_minimax(&self) -> (usize, usize) {
        todo!()
    }

    fn play_round(&mut self) -> Mark {
        let term = Term::stdout();
        loop {
            self.print_grid();
            if let Some(winner) = self.check_winner() {
                println!("\n{} won this round!", winner);
                return winner;
            }
            let (row, col) = if let Opponent::Computer = self.opponent {
                if self.turn == self.player {
                    self.play_turn_human()
                } else if let Some(Difficulty::Easy) = self.difficulty {
                    self.play_turn_computer_random()
                } else {
                    self.play_turn_computer_minimax()
                }
            } else {
                self.play_turn_human()
            };
            if self.grid[row][col].empty {
                self.grid[row][col] = Cell {
                    empty: false,
                    mark: self.turn,
                };
                self.turn = !self.turn;
            }
            // TODO: Implement computer turn
            term.clear_last_lines(4);
        }
    }

    // TODO: Possibly render the table more fancifully
    fn print_grid(&self) {
        for row in self.grid.iter() {
            for cell in row.iter() {
                print!("{}\t", cell);
            }
            println!();
        }
    }
}

fn main() {
    let mut game = Game::setup();
    // TODO: Main menu + scoring here
    loop {
        game.play_round();
        game.grid = Grid::default();
        print!("Starting new game");
        stdout().flush();
        let term = Term::stdout();
        for _ in 0..3 {
            print!(".");
            stdout().flush();
            sleep(Duration::from_millis(500));
        }
        // Fix issue with new line getting created
        term.clear_last_lines(6);
        term.clear_to_end_of_screen();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CELL_X: Cell = Cell {
        empty: false,
        mark: Mark::X,
    };
    const CELL_O: Cell = Cell {
        empty: false,
        mark: Mark::O,
    };

    #[test]
    fn winner_none() {
        // O   X   X
        // X   O   O
        // X   O   X
        let grid = [
            [CELL_O, CELL_X, CELL_X],
            [CELL_X, CELL_O, CELL_O],
            [CELL_X, CELL_O, CELL_X],
        ];
        let game = Game {
            grid,
            ..Game::default()
        };
        assert_eq!(game.check_winner(), None);
    }
    #[test]
    fn winner_line_0_x() {
        // X   X   X
        // O   X   O
        // X   O   O
        let grid = [
            [CELL_X, CELL_O, CELL_X],
            [CELL_X, CELL_X, CELL_O],
            [CELL_X, CELL_O, CELL_O],
        ];
        let game = Game {
            grid,
            ..Game::default()
        };
        assert_eq!(game.check_winner(), Some(Mark::X));
    }
    #[test]
    fn winner_line_0_o() {
        // O   O   O
        // X   O   X
        // O   X   X
        let grid = [
            [CELL_O, CELL_X, CELL_O],
            [CELL_O, CELL_O, CELL_X],
            [CELL_O, CELL_X, CELL_X],
        ];
        let game = Game {
            grid,
            ..Game::default()
        };
        assert_eq!(game.check_winner(), Some(Mark::O));
    }
    #[test]
    fn winner_col_0_x() {
        // X   O   X
        // X   X   O
        // X   O   O
        let grid = [
            [CELL_X, CELL_X, CELL_X],
            [CELL_O, CELL_X, CELL_O],
            [CELL_X, CELL_O, CELL_O],
        ];
        let game = Game {
            grid,
            ..Game::default()
        };
        assert_eq!(game.check_winner(), Some(Mark::X));
    }
    // TODO: Test col 0 for O

    // TODO: Test lines &columns 1 and 2 for X and O
    #[test]
    fn winner_prim_diag_x() {
        // X   O   X
        // O   X   O
        // X   O   X
        let grid = [
            [CELL_X, CELL_O, CELL_X],
            [CELL_O, CELL_X, CELL_O],
            [CELL_X, CELL_O, CELL_X],
        ];
        let game = Game {
            grid,
            ..Game::default()
        };
        assert_eq!(game.check_winner(), Some(Mark::X));
    }
    // TODO: Test primary diagonal for O
    // TODO: Test secondary diagonal for X and O
    // TODO: Test total count of winning cases is 5478
}
