use std::io::{self, Read, prelude::*};

use tic_tac_toe_nd::Board;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("Welcome, to a wild and unbridled version of tic-tac-toe, played in as many dimensions as you wish!");

    let dimension: u8 = loop {
        print!("\nEnter dimension of game: ");
        let _ = stdout.flush().unwrap();

        let mut input = String::new();
        _ = stdin.read_line(&mut input).unwrap();

        match input.trim().parse() {
            Ok(val) => break val,
            Err(e) => println!("Failed to parse as u8: {e}"),
        }
    };
    
    let num_players: u8 = loop {
        print!("\nEnter number of players: ");
        let _ = stdout.flush().unwrap();

        let mut input = String::new();
        _ = stdin.read_line(&mut input).unwrap();

        match input.trim().parse() {
            Ok(val) if val > 0 => break val,

            Ok(_) => println!("Need at least 1 player"),
            Err(e) => println!("Failed to parse as u8: {e}"),
        }
    };

    let mut board = Board::new(dimension);

    println!("\nSorry, but for right now you'll have to keep track of the board yourself.");
    println!("I'll tell you if there's a win, though!\n");

    let mut current_player = 0;

    loop {
        println!("{board:?}");

        let pos: Vec<u8> = loop {
            print!("\nEnter position to place piece: ");
            let _ = stdout.flush().unwrap();

            let mut input = String::new();
            let _ = stdin.read_line(&mut input).unwrap();

            let parsed = input.trim()
                .split([' ', ','])
                .map(|s| s.replace([' ', ',', '_'], ""))
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<u8>());
            
            break parsed
                .map(|n| n.unwrap())
                .collect()
        };

        match board.place_piece(current_player+1, &pos) {
            Ok(b) if b==true => break,
            Ok(_) => {
                current_player = (current_player+1) % num_players;
            },

            Err(e) => println!("{e}"),
        }
        let _ = stdout.flush();
    }

    println!("YOU WIN");
}
