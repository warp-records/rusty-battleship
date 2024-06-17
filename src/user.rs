
use ViewState::*;
use crate::game::{SIZE, Player, Coord, Orientation, Ship, NUM_SHIPS};
use Orientation::*;
use std::io::stdin;
use std::str::FromStr;
use std::mem;

#[derive(Clone, Copy, PartialEq)]
pub enum ViewState {
    Hit, Miss, Blank,
}

#[derive(Clone)]
pub struct PlayerView {
    pub state: [[ViewState; SIZE]; SIZE]
}

use std::fmt;
impl fmt::Display for PlayerView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        write!(f, "\n  |");

        for i in 1..=SIZE {
            write!(f, "{}|", i)?;
        }

        assert!(SIZE <= 26);
        for y in 0..SIZE {
            //Just let it crash or something if it's more than 26 lol
            write!(f, "\n{} |", alphabet.as_bytes()[y] as char)?;

            for x in 0..SIZE {
                let indicator = match self.state[x][y] {
                    Hit => "X",
                    Miss => "O",
                    Blank => " ",
                };

                write!(f, "{indicator}|")?;
            }
        }

        write!(f, " ")
    }
}


impl PlayerView {
    pub fn place_ship(&self, ship_size: usize, coord: Coord, orient: Orientation) -> Result<Self, ()> {
        
        let Ship = Hit;

        let mut curr_coord = Ok(coord);
        let mut new_view = self.clone();

        for _ in 0..ship_size {
            if let Ok(new_coord) = curr_coord {
                if new_coord.in_board() && new_view.state[new_coord.x][new_coord.y] == Blank {
                    new_view.state[new_coord.x][new_coord.y] = Ship;
                    curr_coord = new_coord.shift(orient);
                    continue;
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }

        Ok(new_view)
    }

}

pub struct User {
    pub name: String,
    pub view: PlayerView,
}

impl User {
    pub fn new(name: &str) -> User {
        User { 
            name: name.to_string(),
            view: PlayerView { state: [[Blank; SIZE]; SIZE] }
        }
    }
}

    /*
    Test input:

    (1, 1)
    Right
    (1, 2)
    Right
    (1, 3)
    Right
    (1, 4)
    Right
    

    Better test input:

b computer:156 (computer.turn)
    
(4, 4)
Left
(5, 4)
Down
(9, 6)
Up
(6, 1)
Right

(1, 1)
(1, 1)
(1, 1)
(1, 1)
(1, 1)
(1, 1)
(1, 1)
(1, 1)
(1, 1)
(1, 1)


computer plays:
(4, 4) - hit
(4, 3) - miss
(4, 5) - miss
(3, 5) - hit

SHOULD BE (2, 4):
(5, 4) - hit

(6, 6) - miss
    */

impl Player for User {

     fn place_ships(&self) -> [Ship; NUM_SHIPS] {  
        let mut placement_view = PlayerView { state: [[Blank; SIZE]; SIZE] };

        let mut placements = [Ship::uninitialized(); NUM_SHIPS];

        //ship sizes start at 2 according to the rules
        let mut ship_size = 2;
        let mut told_user_ai_error = false;

        while ship_size <= NUM_SHIPS+1 {

            let ship_type_str = match ship_size {
                5 => String::from("Carrier"),
                4 => String::from("Battleship"),
                3 => String::from("Cruiser"),
                2 => String::from("Destroyer"),
                _ => format!("Size {ship_size}"),
            };

            println!("{}", placement_view);
            println!("Place your {ship_type_str} ship:\n");

            let mut input_str = String::new();

            stdin().read_line(&mut input_str).unwrap();
            let coord = Coord::from_str(&input_str);

            input_str.clear();
            stdin().read_line(&mut input_str).unwrap();
            //let trimmed_slice = input_str.as_str().trim();

            let orient: Result<Orientation, ()> = match input_str.as_str().trim() {
                "Up" => Ok(Up),
                "Down" => Ok(Down),
                "Left" => Ok(Left),
                "Right" => Ok(Right),
                _ => Err(())
            };

            if coord.is_err() || orient.is_err() {
                println!("Usage: \n([x], [y])\n[Up, Down, Left, or Right]");
                continue;
            }

            let orient = orient.unwrap();
        

            let mut curr_coord = coord;
            let mut valid_flag = true;

            let mut new_view = placement_view.clone();

            for _ in 0..ship_size {
                if let Ok(new_coord) = curr_coord {
                    if new_coord.in_board() && new_view.state[new_coord.x][new_coord.y] != Hit {
                        new_view.state[new_coord.x][new_coord.y] = Hit;
                        curr_coord = new_coord.shift(orient);
                        continue;
                    }
                }

                valid_flag = false;
                break;
            }

            if !valid_flag {
                println!("Invalid ship placement");
                continue;  
            }

            //Best way to compensate for an algorithm edge case
            let coord = coord.unwrap();

            for i in 0..ship_size-2 {


                let other = placements[i];

                let both_horiz = (orient == Left || orient == Right) &&
                                        (other.orient == Left || other.orient == Right);

                let adjacent_y_axis = coord.y.abs_diff(other.coord.y) == 1;

                let mut start_coord = coord;
                let mut end_coord = coord.shift_dist(orient, ship_size-1).unwrap();
                if start_coord.x > end_coord.x { 
                    mem::swap(&mut start_coord, &mut end_coord); 
                }


                let mut other_start_coord = other.coord;
                let mut other_end_coord = other.coord.shift_dist(other.orient, other.len-1).unwrap();
                if other_start_coord.x > other_end_coord.x { 
                    mem::swap(&mut other_start_coord, &mut other_end_coord); 
                }


                let (larger_span, smaller_span) = if ship_size > other.len { 
                    ((start_coord, end_coord), (other_start_coord, other_end_coord))
                } else {
                    ((other_start_coord, other_end_coord), (start_coord, end_coord))
                };

                let x_overlapping = smaller_span.1.x >= larger_span.0.x && smaller_span.0.x <= larger_span.1.x;

                if both_horiz && adjacent_y_axis && x_overlapping {
                    println!("\nInvalid placement: AI algorithm performs poorly \
                            when two horizontal ships are placed next to each other");

                    if !told_user_ai_error {
                        println!("\nIt's a perfectly legal move btw,\n \
                        I just don't feel like adapting the algorithm lmao");
                        told_user_ai_error = true;
                    }

                    valid_flag = false;
                    break;
                }
            }

            if !valid_flag { continue; }

            placement_view = new_view;

            //-2 since ship placements start at size 2
            placements[ship_size-2].len = ship_size;
            placements[ship_size-2].coord = coord;
            placements[ship_size-2].orient = orient;

            ship_size += 1;
        }

        placements
    }
    

    fn turn(&mut self) -> Coord {

        println!("{}", self.view);
        println!("\n{}'s turn\nEnter coordinates:\n", self.name);

        let mut fail_count: u32 = 0;

        loop {

            let mut coord_str = String::new();
            stdin().read_line(&mut coord_str).unwrap();
            let coord = Coord::from_str(&coord_str);

            if coord.is_ok() {
                return coord.unwrap();
            }

            fail_count += 1;

            match fail_count {
                1 => println!("Input error: format is (x, y), where x and y are integers from 1-{}", SIZE),
                2 => println!("Is it really that fucking hard to understand?"),
                3 => println!("Jesus fucking christ how hard is it to enter a fucking coordinate pair"),
                _ => panic!("\nError: user is too stupid to follow simple instructions\n"),
            }
        }
    }

    fn hit_feedback(&mut self, coord: Coord, hit: bool) {
        self.view.state[coord.x][coord.y] = if hit { Hit } else { Miss };
        //println!("({}, {}) is a {}!", coord.x+1, coord.y+1, if hit { "hit" } else { "miss" });
    }

    fn count_hits(&self) -> usize {
        self.view.state.iter().flatten().filter(|&&state| state == Hit).count()
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

