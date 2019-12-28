extern crate rand;
use rand::Rng;

use std::io::Write;
use std::process::exit;
use std::env;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Orientation {
    NorthSouth,
    EastWest,
}

type Room = [(usize, Direction); 3];

#[derive(Clone, Debug)]
struct Maze {
    rooms: Vec<Room>,
    bats: usize,
    pit: usize,
    wumpus: usize,

    arrows: i32,
}

use Direction::{North, South, East, West};
use Orientation::{NorthSouth, EastWest};

impl Direction {
    // This takes `self` as a receiver instead of `&self` because we
    // implemented copy.
    fn opposite(self) -> Direction {
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
        }
    }

    // Finds a direction that isn't present in this list
    /*
    fn not_present(ls: &[Self]) -> Option<Direction> {
        let mut present = [false; 4];

        for d in ls.iter() {
            match d {
                Self::North => present[0] = true,
                Self::South => present[1] = true,
                Self::East => present[2] = true,
                Self::West => present[3] = true,
            }
        }

        let from_int = |i| match i {
            0 => Self::North,
            1 => Self::South,
            2 => Self::East,
            3 => Self::West,
            _ => unreachable!(),
        };

        present
            .into_iter()
            .enumerate()
            .filter(|(_, &p)| !p)
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
            .pop()
            .map(from_int)
    }
    */

    // Gives the orientation
    fn orientation(&self) -> Orientation {
        match self {
            North => NorthSouth,
            South => NorthSouth,
            East => EastWest,
            West => EastWest,
        }
    }
}

impl Orientation {
    fn major(&self) -> Direction {
        match self {
            NorthSouth => North,
            EastWest => East,
        }
    }

    fn minor(&self) -> Direction {
        match self {
            NorthSouth => South,
            EastWest => West,
        }
    }
}

impl Maze {
    fn quad() -> Self {
        let rooms = vec![
            [(1, West), (2, North), (3, East)],
            [(0, East), (2, West), (3, North)],
            [(0, South), (1, East), (3, West)],
            [(0, West), (1, South), (2, East)],
        ];

        Self {
            rooms,
            bats: 0,
            pit: 0,
            wumpus: 0,
            arrows: STARTING_ARROWS,
        }
    }

    // Expands the maze to include more rooms at the given index
    fn expand(&mut self, idx: usize, rand_bool: bool) {
        fn link_idx(idx: usize, r: Room) -> usize {
            r.iter().position(|(i,_)| i == &idx).unwrap()
        }

        // It may be helpful to remember:
        //   self.rooms[idx] = r0
        //   r0 = [(r0[0].0, r0[0].1), (r0[1].0, r0[1].1), (r0[2].0, r0[2].1)]
        // Generally, we're taking the connections to r0 and redirecting them
        // to other nodes.
        let r0 = self.rooms[idx];

        // The indexes of the tunnel to r0 in the rooms it links to
        let r0_from_others = [
            link_idx(idx, self.rooms[r0[0].0]),
            link_idx(idx, self.rooms[r0[1].0]),
            link_idx(idx, self.rooms[r0[2].0]),
        ];

        // We're creating two new rooms: r1 and r2.
        let r1_idx = self.rooms.len();
        let r2_idx = r1_idx + 1;

        // indexes of the major/minor directions
        let (fst_maj, fst_min, snd) = {
            // directions
            let ds = r0.iter().map(|(_,d)| d).collect::<Vec<_>>();
            
            // check against the other two
            let o = ds[0].orientation();

            if ds[1].orientation() == o {
                if o.major() == *ds[0] {
                    (0, 1, 2)
                } else {
                    (1, 0, 2)
                }
            } else if ds[2].orientation() == o {
                if o.major() == *ds[0] {
                    (0, 2, 1)
                } else {
                    (2, 0, 1)
                }
            } else {
                if ds[1].orientation().major() == *ds[1] {
                    (1, 2, 0)
                } else {
                    (2, 1, 0)
                }
            }
        };

        let fst_or = r0[fst_maj].1.orientation();
        let snd_d = r0[snd].1;

        let r0_new = [
            (r0[fst_maj].0, fst_or.major()),
            (r1_idx, fst_or.minor()),
            (r2_idx, snd_d),
        ];
        
        // we don't need to set this existing room because it's already there.

        let r2_r1_d = if rand_bool {
            fst_or.major()
        } else {
            snd_d
        };

        let r1 = [
            (idx, fst_or.major()),
            (r0[snd].0, snd_d),
            (r2_idx, r2_r1_d.opposite()),
        ];

        self.rooms[r0[snd].0][r0_from_others[snd]] = (r1_idx, snd_d.opposite());

        let r2 = [
            (r1_idx, r2_r1_d), // This just continues from the last of r1. Can be chosen
            (r0[fst_min].0, fst_or.minor()),
            (idx, snd_d.opposite()),
        ];

        self.rooms[r0[fst_min].0][r0_from_others[fst_min]] = (r2_idx, fst_or.major());

        // set all of the rooms
        self.rooms[idx] = r0_new;
        self.rooms.push(r1);
        self.rooms.push(r2);
    }

    // The total number will be 4 + (2 * adds)
    fn generate(adds: u32) -> Self {
        let mut rng = rand::thread_rng();

        let mut maze = Self::quad();

        for _ in 0 .. adds {
            let idx = (rng.gen::<f32>() * maze.rooms.len() as f32) as usize;
            maze.expand(idx, rng.gen());
        }

        maze.bats = 1 + (rng.gen::<f32>() * (maze.rooms.len() -1) as f32) as usize;
        
        maze.pit = loop {
            let i = 1 + (rng.gen::<f32>() * (maze.rooms.len() -1) as f32) as usize;
            if i != maze.bats {
                break i;
            }
        };

        maze.wumpus = if adds != 0 {
            loop {
                let i = 1 + (rng.gen::<f32>() * (maze.rooms.len() -1) as f32) as usize;

                // guarantee that the wumpus isn't next to any of the starting
                // squares
                if !maze.rooms[0].iter().any(|(r,_)| r == &i) {
                    break i;
                }
            }
        } else {
            1 + (rng.gen::<f32>() * (maze.rooms.len() -1) as f32) as usize
        };

        maze
    }

    // Currently only works with up to three-digit numbers
    fn display_room(&self, room_idx: usize) {
        // the maximum length of the room numbers
        // This is for a later improvement
        //
        // let max_length = self.rooms[idx].iter()
        //     .map(|(i,_)| (*i as f32).log10().ceil() as usize)
        //     .max()
        //     .unwrap();

        // Typical display, with terminal border at the indent of the first '/'
        /*
                          .                
                      ╔═╝ . ╚═╗            
                      ╝       ║            
                     ..   7   ║            
                      ╗       ║            
              .       ╚═╗   ╔═╝       .    
          ╔═╝ . ╚═╗  ╔══╝   ╚══╗  ╔═╝ . ╚═╗
          ╝       ╚══╝         ╚══╝       ║
         ..  37          You         61   ║
          ╗       ╔══╗    3    ╔══╗       ║
          ╚═══════╝  ╚═════════╝  ╚═╗ . ╔═╝
                                      .    
        */

        // magic numbers, based on above
        const WIDTH: usize = 35;
        const HEIGHT: usize = 17;
        const SIZE: usize = (WIDTH + 1) * HEIGHT;

        // row counted down from the top, starting at 0
        fn idx(row: usize, col: usize) -> usize {
            // add one for newline
            (WIDTH + 1) * row + col
        }

        // new strings should be from top down
        fn overwrite(data: &mut [char; SIZE], new: Vec<String>, row: usize, col: usize) {
            for (r, s) in new.iter().enumerate() {
                let chars = s.chars().collect::<Vec<_>>();
                let i = idx(row + r, col);
                data[i .. i + chars.len()].copy_from_slice(&chars);
            }
        }

        // `top_bar` includes the corners and (if it's not the center) the
        // line above
        //
        // Vector has length two
        fn top_bar(r: Room, center: bool) -> Vec<String> {
            let has_room_above = r.iter().any(|(_,d)| d == &North);

            if center {
                if has_room_above {
                    vec![
                        " ╚═╗   ╔═╝ ".into(),
                        "╔══╝   ╚══╗".into(),
                    ]
                } else {
                    vec![
                        "".into(),
                        "╔═════════╗".into(),
                    ]
                }
            } else {
                if has_room_above {
                    vec![
                        "    .    ".into(),
                        "╔═╝ . ╚═╗".into(),
                    ]
                } else {
                    vec![
                        "         ".into(),
                        "╔═══════╗".into(),
                    ]
                }
            }
        }

        fn bot_bar(r: Room, center: bool) -> Vec<String> {
            let has_room_below = r.iter().any(|(_,d)| d == &South);

            if center {
                if has_room_below {
                    vec![
                        "╚══╗   ╔══╝".into(),
                        " ╔═╝   ╚═╗ ".into(),
                    ]
                } else {
                    vec!["╚═════════╝".into()]
                }
            } else {
                if has_room_below {
                    vec![
                        "╚═╗ . ╔═╝".into(),
                        "    .    ".into(),
                    ]
                } else {
                    vec![
                        "╚═══════╝".into(),
                        "         ".into(),
                    ]
                }
            }
        }

        // doesn't include top/bottom bar
        // Each string is of width two
        fn left_side(r: Room, center: bool) -> Vec<String> {
            let has_room_left = r.iter().any(|(_,d)| d == &West);

            if center {
                if has_room_left {
                    vec![
                        "╚══╝".into(),
                        "    ".into(),
                        "╔══╗".into(),
                    ]
                } else {
                    vec![
                        "   ║".into(),
                        "   ║".into(),
                        "   ║".into(),
                    ]
                }
            } else {
                if has_room_left {
                    vec![
                        " ╝".into(),
                        "..".into(),
                        " ╗".into(),
                    ]
                } else {
                    vec![
                        " ║".into(),
                        " ║".into(),
                        " ║".into(),
                    ]
                }
            }
        }

        fn right_side(r: Room, center: bool) -> Vec<String> {
            let has_room_right = r.iter().any(|(_,d)| d == &East);

            if center {
                if has_room_right {
                    vec![
                        "╚══╝".into(),
                        "    ".into(),
                        "╔══╗".into(),
                    ]
                } else {
                    vec![
                        "║".into(),
                        "║".into(),
                        "║".into(),
                    ]
                }
            } else {
                if has_room_right {
                    vec![
                        "╚ ".into(),
                        "..".into(),
                        "╔ ".into(),
                    ]
                } else {
                    vec![
                        "║ ".into(),
                        "║ ".into(),
                        "║ ".into(),
                    ]
                }
            }
        }

        // position should be of the top-left corner - so we'll expect to go
        // up and to the left
        fn set_room(data: &mut [char; SIZE], r: Room, idx: usize, center: bool, row: usize, col: usize) {
            if center {
                overwrite(data, top_bar(r, center), row-1, col);
                overwrite(data, left_side(r, center), row+1, col-3);
                overwrite(data, right_side(r, center), row+1, col+10);
            } else {
                overwrite(data, top_bar(r, center), row-1, col);
                overwrite(data, left_side(r, center), row+1, col-1);
                overwrite(data, right_side(r, center), row+1, col+8);
            }

            overwrite(data, bot_bar(r, center), row+4, col);

            // do work on the middle section
            let middle = if center {
                vec![
                    "         ".into(),
                    "   You   ".into(),
                    format!("{:^9}", idx),
                ]
            } else {
                vec![
                    "       ".into(),
                    format!("{:^7}", idx),
                    "       ".into(),
                ]
            };

            overwrite(data, middle, row+1, col+1);
        }
        
        // Initializing the thing to display.
        let mut display = [' '; SIZE];
        for r in 0 .. HEIGHT {
            display[idx(r, WIDTH)] = '\n';
        }

        let r = self.rooms[room_idx];

        for &(rr, d) in r.iter() {
            let (row, col) = match d {
                North => (1, 13),
                South => (11, 13),
                East => (6, 25),
                West => (6, 1),
            };

            set_room(&mut display, self.rooms[rr], rr, false, row, col);
        }

        set_room(&mut display, r, room_idx, true, 6, 12);

        println!("{}", display.into_iter().collect::<String>());
    }

    fn do_shoot(&mut self, player_room: usize, stdin: &std::io::Stdin) -> bool {
        let dist = loop {
            print!("How far do you want to shoot? ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();

            stdin.read_line(&mut input).unwrap();

            input = input.trim().to_lowercase();

            // attempt to parse the distance
            let dist: i32 = match input.parse() {
                Ok(d) => d,
                Err(_) => {
                    println!("Please enter a number.");
                    continue;
                },
            };

            if dist <= 0 || dist > 5 {
                println!("Please enter a number between 1 and 5.");
                continue;
            } else if dist > self.arrows {
                println!("You don't have enough arrows to shoot that far!");
                println!("You only have {}.", self.arrows);
                continue;
            }

            self.arrows -= dist;

            break dist;
        };

        let mut idx = player_room;
        for _ in 0 .. dist {
            self.display_room(idx);

            idx = loop {
                print!("Pick a direction to continue the shot: ");
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                stdin.read_line(&mut input).unwrap();
                input = input.trim().to_lowercase();

                let direction = match input.as_ref() {
                    "left" | "west" => West,
                    "right" | "east" => East,
                    "up" | "north" => North,
                    "down" | "south" => South,
                    "quit" | "exit" => exit(0),
                    _ => {
                        println!("Directions should be left/right/up/down or north/south/east/west");
                        println!("Enter 'quit' to quit");
                        continue;
                    },
                };

                let r = self.rooms[idx];
                if let Some(i) = r.iter().position(|(_,d)| d == &direction) {
                    break r[i].0;
                }

                println!("Can't shoot that way, there's a wall!");
            };

            if idx == player_room && idx == self.wumpus {
                println!("You have done the impossible!");
                println!("You've killed yourself and the Wumpus in one fell swoop!");
                return true;
            } else if idx == self.wumpus {
                println!("You killed the Wumpus!");
                return true;
            } else if idx == player_room {
                println!("You killed... yourself.");
                return true;
            }
        }

        println!("You didn't hit anything...");

        if self.arrows == 0 {
            println!("You ran out of arrows! You lose.");
            return true;
        }

        // move the wumpus
        if rand::random::<f32>() < WUMPUS_MOVE_PROB {
            let new_idx = (rand::random::<f32>() * 3.0) as usize;
            self.wumpus = self.rooms[self.wumpus][new_idx].0;
        } 

        if self.wumpus == player_room {
            println!("You woke up the wumpus and he ate you!");
            return true;
        }

        false
    }
}

const WUMPUS_MOVE_PROB: f32 = 0.75;
const STARTING_ARROWS: i32 = 5;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("You must provide exactly one argument.");
        return;
    }

    let n_rooms = match args[1].parse::<i32>() {
        Ok(n) => n,
        Err(_) => {
            println!("You must give a number for the rooms");
            return;
        },
    };

    if n_rooms < 4 || !(n_rooms % 2 == 0) {
        println!("The number of rooms must be even and ≥ 4");
        return;
    }

    let n_adds = ((n_rooms - 4) / 2) as u32;

    let mut maze = Maze::generate(n_adds);

    let stdin = std::io::stdin();

    let mut r_idx = 0;
    let mut next = true;
    loop {
        if next {
            maze.display_room(r_idx);
            next = false;
        }

        if maze.wumpus == r_idx {
            // if we don't wake it (waking it moves it)
            if rand::random::<f32>() > WUMPUS_MOVE_PROB {
                println!("You woke up the wumpus and he ate you!");
                break;
            }

            // pick a new room
            let new_idx = (rand::random::<f32>() * 3.0) as usize;
            maze.wumpus = maze.rooms[maze.wumpus][new_idx].0;
        }

        if maze.bats == r_idx {
            // pick a new room to go into
            println!("The bats whisk you away!");
            // this has the effect of do-while
            while {
                r_idx = (rand::random::<f32>() * maze.rooms.len() as f32) as usize;

                r_idx == maze.bats && r_idx == maze.pit
            } {}
            next = true;
            continue;
        }

        if maze.pit == r_idx {
            println!("You fall into a bottomless pit!");
            break;
        }

        if maze.rooms[r_idx].iter().any(|(r,_)| r == &maze.wumpus) {
            println!("You smell something terrible nearby.");
        }

        if maze.rooms[r_idx].iter().any(|(r,_)| r == &maze.bats) {
            println!("You hear a rustling.");
        }

        if maze.rooms[r_idx].iter().any(|(r,_)| r == &maze.pit) {
            println!("You feel a cold wind blowing from a nearby cavern.");
        }

        print!("Please pick a direction: ");
        std::io::stdout().flush().unwrap();
        
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        input = input.trim().to_lowercase();

        let direction = match input.as_ref() {
            "left" | "west" => West,
            "right" | "east" => East,
            "up" | "north" => North,
            "down" | "south" => South,
            "shoot" => {
                // returns true if the game should be over 
                if maze.do_shoot(r_idx, &stdin) {
                    break;
                } else {
                    next = true;
                    continue;
                }
            },
            "quit" | "exit" => break,
            _ => {
                println!("Directions should be left/right/up/down or north/south/east/west");
                println!("Enter 'quit' to quit");
                continue;
            },
        };

        let r = maze.rooms[r_idx];
        let i = match r.iter().position(|(_,d)| d == &direction) {
            Some(i) => i,
            None => {
                println!("Can't go that way!");
                continue;
            },
        };

        next = true;
        r_idx = r[i].0;
    }

    println!("GAME OVER");
}
