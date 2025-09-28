use std::process::Command;
use std::process::Stdio;
use std::io;
use std::io::prelude::*;
use std::fs;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

fn main() {

    Command::new("stty")
        .args(["raw", "-echo"])
        .stdin(Stdio::inherit())
        .status()
        .expect("failed to set raw mode");

    let stdin: io::Stdin = io::stdin();
    let stdout: io::Stdout = io::stdout();

    let mut console = Console::new(stdout);

    let mut map : HashMap<(u32,u32), Entity> = HashMap::new();

    let map_data = fs::read_to_string("map1.txt").unwrap();

    for entity in map_data.split("\n"){

        let parts : Vec<&str> = entity.trim().split(",").collect();

        if parts.len() == 3{

            let entity_name = parts[0];
            let row = parts[1].parse::<u32>().unwrap();
            let column = parts[2].parse::<u32>().unwrap();

            let entity = match entity_name {
                "wall" => {
                    Entity::wall(row, column)
                },
                "cherry" => {
                    Entity::cherry(row, column)
                },
                "ghost" => {
                    Entity::ghost(row, column)
                },
                "rock" => {
                    Entity::rock(row, column)
                },
                "bomb" => {
                    Entity::bomb(row, column)
                },
                "diamond" => {
                    Entity::diamond(row, column)
                },
                "power" => {
                    Entity::power(row, column)
                },
                _ => {
                    Entity::empty(0, 0)
                }
            };

            map.insert((row, column), entity);
        }
    }

    let input = Arc::new(Mutex::new(Direction::None));

    let input_clone1: Arc<Mutex<Direction>> = input.clone();
    let input_clone2: Arc<Mutex<Direction>> = input.clone();

    thread::spawn(move || {

        let mut score = 0;
        let mut lives = 3;
        let mut bombs = 0;
        let mut is_game_over = false;

        let mut enemies : Vec<Entity> = Vec::new();

        for (_key, entity) in map.iter(){

            if entity.entity == EntityType::Ghost {
                enemies.push(entity.clone());
            }
            
            console.draw(entity);
            console.flush();
        }

        let mut player = Entity::player(2, 3);

        loop {

            let input = {
                let input = input_clone1.lock().unwrap();
                input.clone()
            };

            let next_position = match input {
                Direction::Left => {
                    (player.position.row, player.position.column - 2)
                },
                Direction::Right => {
                    (player.position.row, player.position.column + 2)
                },
                Direction::Up => {
                    (player.position.row -1, player.position.column)
                },
                Direction::Down => {
                    (player.position.row +1, player.position.column)
                },
                _ => {
                    (0,0)
                }
            };

            let mut collision = false;

            if map.contains_key(&next_position){
                match map.get_mut(&next_position){

                    Some(entity) => {

                        match entity.entity {
                            EntityType::Wall => {
                                collision = true;
                            },
                            EntityType::Rock => {
                                if bombs == 0 {
                                    collision = true;
                                }else{
                                    bombs -= 1;
                                    map.remove(&next_position);
                                }
                            },
                            EntityType::Diamond => {
                                score += 1;
                            },
                            EntityType::Cherry => {
                                score += 100;
                            },
                            EntityType::Power => {
                                score += 500;
                            },
                            EntityType::Bomb => {
                                bombs += 1;
                            }
                            _ => {}
                        }
                    }
                    _=> {}
                }
            }

            console.write(&format!("{}", " ".repeat(80)), 0, 0);
            console.flush();

            console.write(&format!("Score: {}", score), 0, 0);
            console.write(&format!("{}", "\u{1F60B}".repeat(lives)), 0, 20);
            console.write(&format!("{}", "\u{1F4A3}".repeat(bombs)), 0, 30);

            let empty_slot = Entity::empty(player.last_position.row, player.last_position.column);
            console.draw(&empty_slot);
            console.flush();

            player.last_position = player.position.clone();

            if collision == false {
                match input {
                    Direction::Left => {
                        player.position.column -= 2;
                    },
                    Direction::Right => {
                        player.position.column += 2;
                    },
                    Direction::Up => {
                        player.position.row -= 1;
                    },
                    Direction::Down => {
                        player.position.row += 1;
                    },
                    _ => {}
                }

                map.remove(&(player.last_position.row, player.last_position.column));
            }

            *input_clone1.lock().unwrap() = Direction::None;

            console.draw(&player);
            console.flush();

            for enemy in enemies.iter_mut(){

                let mut moves : Vec<(Direction, EntityType)> = Vec::new();

                // Up
                if enemy.direction != Direction::Down {
                    if let Some(entity) = map.get(&(enemy.position.row - 1, enemy.position.column)){
                        if entity.entity == EntityType::Diamond {
                            moves.push((Direction::Up, entity.entity.clone()));
                        }
                    }else{
                        moves.push((Direction::Up, EntityType::Empty));
                    }
                }

                // Left
                if enemy.direction != Direction::Right {
                    if let Some(entity) = map.get(&(enemy.position.row, enemy.position.column - 2)){
                        if entity.entity == EntityType::Diamond {
                            moves.push((Direction::Left, entity.entity.clone()));
                        }
                    }else{
                        moves.push((Direction::Left, EntityType::Empty));
                    }
                }

                //Down
                if enemy.direction != Direction::Up {
                    if let Some(entity) = map.get(&(enemy.position.row + 1, enemy.position.column)){
                        if entity.entity == EntityType::Diamond {
                            moves.push((Direction::Down, entity.entity.clone()));
                        }
                    }else{
                        moves.push((Direction::Down, EntityType::Empty));
                    }
                }

                // Right
                if enemy.direction != Direction::Left {
                    if let Some(entity) = map.get(&(enemy.position.row, enemy.position.column + 2)){
                        if entity.entity == EntityType::Diamond {
                            moves.push((Direction::Right, entity.entity.clone()));
                        }
                    }else{
                        moves.push((Direction::Right, EntityType::Empty));
                    }
                }

                if moves.len() == 0 {
                    match enemy.direction{
                        Direction::Up => {
                            enemy.direction = Direction::Down;
                        },
                        Direction::Left => {
                            enemy.direction = Direction::Right;
                        },
                        Direction::Down => {
                            enemy.direction = Direction::Up;
                        },
                        Direction::Right => {
                            enemy.direction = Direction::Left;
                        },
                        Direction::None => {}
                    }
                    continue;
                }

                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();

                let random_index = (nanos % moves.len() as u128) as usize; 
                
                if let Some((next_move, entity_type)) = moves.get(random_index){

                    enemy.last_position = Position { row: enemy.position.row, column: enemy.position.column };

                    match next_move {
                        Direction::Up => {
                            enemy.position.row -= 1;
                            enemy.direction = Direction::Up;
                        },
                        Direction::Left => {
                            enemy.position.column -= 2;
                            enemy.direction = Direction::Left;
                        },
                        Direction::Down => {
                            enemy.position.row += 1;
                            enemy.direction = Direction::Down;
                        },
                        Direction::Right => {
                            enemy.position.column += 2;
                            enemy.direction = Direction::Right;
                        },
                        Direction::None => {}
                    }

                    console.draw(enemy);
                    console.flush();

                    let previous_slot = match entity_type {
                        EntityType::Bomb => {
                            Entity::bomb(enemy.last_position.row, enemy.last_position.column)
                        },
                        EntityType::Diamond => {
                            Entity::diamond(enemy.last_position.row, enemy.last_position.column)
                        },
                        _ => {
                            Entity::empty(enemy.last_position.row, enemy.last_position.column)
                        }
                    };

                    console.draw(&previous_slot);
                    console.flush();
                };

                if collision_with_player(&player, &enemy) {
                    if lives == 0 { 
                        is_game_over = true;
                        console.clear();
                        console.write("GAME OVER", 0, 0);
                        console.flush();

                        console.write(&format!("Score: {}", score), 0, 20);
                        console.flush();
                        break;
                    }else{
                        lives -= 1;
                    }
                }
            }

            if is_game_over {
                break;
            }

            thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    loop {
        let mut buffer = [0;1];

        stdin.lock().read_exact(&mut buffer).unwrap();

        match buffer[0]{
            97 => {
                *input_clone2.lock().unwrap() = Direction::Left;
            },
            100 => {
                *input_clone2.lock().unwrap() = Direction::Right;
            },
            115 => {
                *input_clone2.lock().unwrap() = Direction::Down;
            },
            119 => {
                *input_clone2.lock().unwrap() = Direction::Up;
            },
            113 => {
                std::process::exit(0);
            },
            _=> {}
        }
    }

}

fn collision_with_player(player : &Entity, enemy : &Entity) -> bool {
    if player.position.row == enemy.position.row
        && player.position.column == enemy.position.column {
        true
    }else{
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Direction {
    None,
    Up,
    Down,
    Left, 
    Right
}

struct Console{
    stdout : io::Stdout
}

impl Console {

    fn new(mut stdout : io::Stdout) -> Self {

        // Clear screen
        write!(stdout, "\x1b[2J").unwrap();

        // Hide cursor
        write!(stdout, "\x1b[?25l").unwrap();

        // Move cursor to start
        write!(stdout,"\x1b[H").unwrap();

        stdout.flush().unwrap();

        Self {
            stdout
        }
    }

    fn write(&mut self, str : &str, row : u32, column : u32){
        write!(self.stdout, "\x1b[{};{}H{}", row, column, str).unwrap();
    }

    fn draw(&mut self, entity : &Entity){
        write!(self.stdout, "\x1b[{};{}H{}", entity.position.row + 1, entity.position.column, entity.unicode).unwrap();
    }

    fn flush(&mut self){
        self.stdout.flush().unwrap();
    }

    fn clear(&mut self){
        write!(self.stdout, "\x1b[2J").unwrap();
    }
}

#[derive(Debug,Clone)]
struct Entity {
    unicode : String,
    position : Position,
    last_position : Position,
    entity : EntityType,
    direction : Direction
}

impl Entity {

    fn empty(row : u32, column : u32) -> Self {
        Self {
            unicode : " ".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Empty,
            direction : Direction::None
        }
    }

    fn player(row : u32, column : u32) -> Self {
        Self {
            unicode : "\u{1F60B}".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Player,
            direction : Direction::None
        }
    }

    fn wall(row : u32, column : u32) -> Self {
        Self {
            unicode : "\u{1F9F1}".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Wall,
            direction : Direction::None
        }
    }

    fn cherry(row : u32, column : u32) -> Self {
        Self {
            unicode : "\u{1F352}".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Cherry,
            direction : Direction::None
        }
    }

    fn ghost(row : u32, column : u32) -> Self {
        Self {
            unicode : "\u{1F47B}".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Ghost,
            direction : Direction::None
        }
    }

    fn rock(row : u32, column : u32) -> Self {
        Self {
            unicode : "\u{1FAA8}".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Rock,
            direction : Direction::None
        }
    }

    fn bomb(row : u32, column : u32) -> Self {
        Self {
            unicode : "\u{1F4A3}".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Bomb,
            direction : Direction::None
        }
    }

    fn diamond(row : u32, column : u32) -> Self {
        Self {
            unicode : "\u{1F538}".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Diamond,
            direction : Direction::None
        }
    }

    fn power(row : u32, column : u32) -> Self {
        Self {
            unicode : "\u{1F31F}".to_string(),
            position : Position { row,  column },
            last_position : Position { row,  column },
            entity : EntityType::Power,
            direction : Direction::None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum EntityType {
    Empty,
    Player,
    Wall,
    Cherry,
    Ghost,
    Rock,
    Bomb,
    Diamond,
    Power
}

#[derive(Debug, Clone)]
struct Position {
    row : u32,
    column : u32
}


