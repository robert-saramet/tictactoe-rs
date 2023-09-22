use console::Term;
use dialoguer::Select;
use rand::{thread_rng, Rng};
use std::fmt::{Display, Formatter, Result};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

#[derive(PartialEq, Debug)]
struct Game {
    grid: Grid,
    state: State,
    player: Mark,
    turn: Mark,
    opponent: Opponent,
    difficulty: Difficulty,
    keymap: Keymap,
    score_x: u32,
    score_o: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            grid: Grid::default(),
            state: State::Ongoing,
            player: Mark::X,
            turn: Mark::X,
            opponent: Opponent::Computer,
            difficulty: Difficulty::Easy,
            keymap: Keymap::Numpad,
            score_x: 0,
            score_o: 0,
        }
    }
}

type Grid = [Cell; 9];

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    Ongoing,
    Won(Mark),
    Tie,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Mark {
    X,
    O,
}

impl Display for Mark {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::X => write!(f, "X"),
            Self::O => write!(f, "O"),
        }
    }
}

#[derive(PartialEq, Debug)]
enum Opponent {
    Computer,
    Human,
}

#[derive(PartialEq, Debug)]
enum Difficulty {
    Easy, // random choice
    Hard, // minmax algo
}

#[derive(PartialEq, Debug)]
enum Keymap {
    Standard,
    Numpad,
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Cell {
    empty: bool,
    mark: Mark,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            empty: true,
            mark: Mark::X,
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if self.empty {
            write!(f, "□")
        } else {
            write!(f, "{}", self.mark)
        }
    }
}

impl Game {
    fn terminal_setup_prompts() -> Self {
        let term = Term::stdout();
        term.clear_screen()
            .expect("Failed to clear the terminal - quitting");

        println!("Welcome to tic-tac-toe");

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
                .expect("Failed to read input on setup - quitting");
        }

        let player = match choices[2] {
            0 => Mark::X,
            1 => Mark::O,
            _ => unreachable!(),
        };
        let opponent = match choices[0] {
            0 => Opponent::Computer,
            1 => Opponent::Human,
            _ => unreachable!(),
        };
        let difficulty = match choices[1] {
            0 => Difficulty::Easy,
            1 => Difficulty::Hard,
            _ => unreachable!(),
        };
        let keymap = match choices[3] {
            0 => Keymap::Standard,
            1 => Keymap::Numpad,
            _ => unreachable!(),
        };
        Game {
            player,
            opponent,
            difficulty,
            keymap,
            ..Game::default()
        }
    }

    fn print_grid(&self) {
        for (i, cell) in self.grid.iter().enumerate() {
            print!("{}\t", cell);
            if (i + 1) % 3 == 0 {
                println!();
            }
        }
    }

    fn play_round(&mut self) {
        let term = Term::stdout();
        self.turn = Mark::X;
        loop {
            self.print_grid();
            if let State::Won(winner) = self.check_state() {
                println!("\n{} won this round!", winner);
                match winner {
                    Mark::X => self.score_x += 1,
                    Mark::O => self.score_o += 1,
                };
                return;
            } else if let State::Tie = self.check_state() {
                println!("\nIt's a tie");
                return;
            }
            let choice = if let Opponent::Computer = self.opponent {
                if self.turn == self.player {
                    sleep(Duration::from_millis(500));
                    self.play_turn_human()
                } else {
                    self.play_turn_computer()
                }
            } else {
                self.play_turn_human()
            };
            // Set cell if it is empty
            if self.grid[choice].empty {
                self.grid[choice] = Cell {
                    empty: false,
                    mark: self.turn,
                };
                self.turn = match self.turn {
                    Mark::X => Mark::O,
                    Mark::O => Mark::X,
                };
            }
            // 3 lines for the grid, 2 lines for messages printed
            term.clear_last_lines(5)
                .expect("Failed to clear the terminal - quitting");
        }
    }

    fn check_state(&mut self) -> State {
        // Check and return if game is won
        if let Some(winner) = self.check_winner() {
            self.state = State::Won(winner);
            return self.state;
        }
        // Check and return early if any moves are available
        if self.grid.iter().any(|cell| cell.empty) {
            self.state = State::Ongoing;
            return self.state;
        }
        // If no moves are available then it's a tie
        self.state = State::Tie;
        self.state
    }

    fn check_winner(&self) -> Option<Mark> {
        let grid = &self.grid;
        // Check if all cells on a row/column are the same and not empty
        for i in 0..3 {
            let j = i * 3;
            if !grid[i].empty && grid[i] == grid[i + 3] && grid[i + 3] == grid[i + 6] {
                // eq columns
                return Some(grid[i].mark);
            } else if !grid[j].empty && grid[j] == grid[j + 1] && grid[j + 1] == grid[j + 2] {
                // eq rows
                return Some(grid[j].mark);
            }
        }
        // Check if all cells on the primary/secondary diagonal are the same and not empty
        if !grid[4].empty
            && ((grid[0] == grid[4] && grid[4] == grid[8])
                || (grid[6] == grid[4] && grid[4] == grid[2]))
        {
            return Some(grid[4].mark);
        }
        None
    }

    fn play_turn_human(&self) -> usize {
        println!("{}'s turn", self.turn);
        println!("Select a cell from 1 to 9");
        let choice = self.get_human_input(0);
        match self.keymap {
            Keymap::Standard => choice - 1,
            Keymap::Numpad => (choice - 1) % 3 + 3 * (2 - (choice - 1) / 3),
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn get_human_input(&self, depth: u8) -> usize {
        // Add a max recursion depth constant
        if depth > 5 {
            panic!("Failed to read a valid digit after 5 attempts - quitting")
        }
        let term = Term::stdout();
        let input = term.read_char();
        let mut choice = '*';
        input.is_ok().then(|| choice = input.unwrap());
        if choice == '0' || !choice.is_numeric() {
            return self.get_human_input(depth + 1);
        }
        choice
            .to_digit(10)
            .expect("Failed to convert char to digit - quitting") as usize
    }

    fn play_turn_computer(&self) -> usize {
        println!("{}'s turn", self.turn);
        print!("Let the computer think");
        for _ in 0..5 {
            print!(".");
            sleep(Duration::from_millis(150));
            stdout().flush().expect("Failed to flush stdout");
        }
        println!();
        match self.difficulty {
            Difficulty::Easy => self.play_turn_computer_random(),
            Difficulty::Hard => self.play_turn_computer_minimax(),
        }
    }

    fn play_turn_computer_random(&self) -> usize {
        let term = Term::stdout();
        let mut empty_cells = vec![];
        // Search for empty cells
        self.grid
            .iter()
            .filter(|cell| cell.empty)
            .enumerate()
            .for_each(|(i, _)| empty_cells.push(i));
        let id = thread_rng().gen_range(0..empty_cells.len());
        term.clear_to_end_of_screen()
            .expect("Failed to clear terminal - quitting");
        empty_cells[id]
    }

    fn play_turn_computer_minimax(&self) -> usize {
        todo!()
    }
}

fn main() {
    let mut game = Game::terminal_setup_prompts();
    loop {
        game.play_round();
        println!("Scores\tX: {}\tO: {}", game.score_x, game.score_o);
        game.grid = Grid::default();
        stdout().flush().expect("Failed to flush stdout");
        print!("Starting new game");
        stdout().flush().expect("Failed to flush stdout");
        let term = Term::stdout();
        for _ in 0..5 {
            print!(".");
            stdout().flush().expect("Failed to flush stdout");
            sleep(Duration::from_millis(600));
        }
        // Fix issue with new line getting created
        term.clear_last_lines(6).expect("Failed to clear screen");
        term.clear_to_end_of_screen()
            .expect("Failed to clear screen");
    }
}

#[cfg(test)]
mod win_tests {
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
            CELL_O, CELL_X, CELL_X, CELL_X, CELL_O, CELL_O, CELL_X, CELL_O, CELL_X,
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
            CELL_X, CELL_X, CELL_O, CELL_X, CELL_O, CELL_X, CELL_X, CELL_O, CELL_O,
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
            CELL_O, CELL_X, CELL_O, CELL_O, CELL_O, CELL_X, CELL_O, CELL_X, CELL_X,
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
            CELL_X, CELL_X, CELL_X, CELL_O, CELL_X, CELL_O, CELL_X, CELL_O, CELL_O,
        ];
        let game = Game {
            grid,
            ..Game::default()
        };
        assert_eq!(game.check_winner(), Some(Mark::X));
    }

    #[test]
    fn winner_prim_diag_x() {
        // X   O   X
        // O   X   O
        // X   O   X
        let grid = [
            CELL_X, CELL_O, CELL_X, CELL_O, CELL_X, CELL_O, CELL_X, CELL_O, CELL_X,
        ];
        let game = Game {
            grid,
            ..Game::default()
        };
        assert_eq!(game.check_winner(), Some(Mark::X));
    }
}

#[cfg(test)]
mod tie_tests {
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
    fn tie_1() {
        // X   X   O
        // O   X   X
        // X   O   O
        let grid = [
            CELL_X, CELL_X, CELL_O, CELL_O, CELL_X, CELL_X, CELL_X, CELL_O, CELL_O,
        ];
        let mut game = Game {
            grid,
            ..Game::default()
        };
        assert_eq!(game.check_state(), State::Tie);
    }
}

#[cfg(test)]
mod setup_tests {
    use super::*;

    #[test]
    fn new_grid_default() {
        const E: Cell = Cell {
            empty: true,
            mark: Mark::X,
        };
        const EMPTY_GRID: [Cell; 9] = [E, E, E, E, E, E, E, E, E];
        assert_eq!(Grid::default(), EMPTY_GRID);
    }

    #[test]
    fn new_game_default() {
        assert_eq!(
            Game::default(),
            Game {
                grid: Grid::default(),
                state: State::Ongoing,
                player: Mark::X,
                turn: Mark::X,
                opponent: Opponent::Computer,
                difficulty: Difficulty::Easy,
                keymap: Keymap::Numpad,
                score_x: 0,
                score_o: 0,
            }
        )
    }
}
