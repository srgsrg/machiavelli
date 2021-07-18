//! # Machiavelli
//!
//! A simple machiavelli card game *(work in progress)*


use std::io::{ stdin, Write };
pub mod sequence_cards;
pub mod table;
pub mod sort;
pub mod encode;
pub mod lib_server;
pub mod lib_client;
pub use sequence_cards::*;
pub use table::*;

pub fn reset_style_string() -> String {
    [
        "\x1b[0m", // reset attributes
        "\x1b[30;47m", // set the foreground and background colours
        "\x1b[?25l" // hide the cursor
    ].join("")
}

/// reset the terminal output style
pub fn reset_style() {
    print!("{}", reset_style_string());
}

/// clear the terminal
pub fn clear_terminal() {
    print!("\x1b[2J\x1b[1;1H");
}


/// Structure to store the game configuration
#[derive(Debug, PartialEq)]
pub struct Config {
    pub n_decks: u8,
    pub n_jokers: u8,
    pub n_cards_to_start: u16,
    pub custom_rule_jokers: bool,
    pub n_players: u8
}


impl Config {

    /// Convert the config structure to a sequence of bytes
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::Config;
    ///
    /// let config = Config {
    ///     n_decks: 2,
    ///     n_jokers: 4,
    ///     n_cards_to_start: 13,
    ///     custom_rule_jokers: false,
    ///     n_players: 2
    /// };
    ///
    /// let config_bytes = config.to_bytes();
    ///
    /// assert_eq!(
    ///     vec![2,4,0,13,0,2], 
    ///     config_bytes);
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.n_decks,
            self.n_jokers,
            (self.n_cards_to_start >> 8) as u8,
            (self.n_cards_to_start & 255) as u8,
            self.custom_rule_jokers as u8,
            self.n_players
        ]
    }

    /// Get a config from a vector of bytes
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::Config;
    ///
    /// let bytes: Vec<u8> = vec![2,4,0,13,0,2];
    ///
    /// let config = Config::from_bytes(&bytes);
    ///
    /// let expected_config = Config {
    ///     n_decks: 2,
    ///     n_jokers: 4,
    ///     n_cards_to_start: 13,
    ///     custom_rule_jokers: false,
    ///     n_players: 2
    /// };
    ///
    /// assert_eq!(expected_config, config);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Config {
        Config {
            n_decks: bytes[0],
            n_jokers: bytes[1],
            n_cards_to_start: (bytes[2] as u16)*256 + (bytes[3] as u16),
            custom_rule_jokers: bytes[4] != 0,
            n_players: bytes[5]
        }
    }
}

/// get the vector of player names from a file
pub fn load_names(fname: &str) -> Result<Vec<String>, InvalidInputError> {
    let content = std::fs::read_to_string(fname)?;
    Ok(content.trim().split("\n").map(String::from).collect())
}

/// save the vector of player names to a file
pub fn save_names(names: &Vec<String>, fname: &str) -> Result<(), InvalidInputError> {
    let names_single_string = names.join("\n");
    let mut file = std::fs::File::create(fname)?;
    file.write_all(names_single_string.as_bytes())?;
    Ok(())
}

/// load the config from a file
pub fn get_config_from_file(fname: &str) -> Result<(Config,String),InvalidInputError> {
    
    // open the file
    let content = std::fs::read_to_string(fname)?;
    let content: Vec<&str> = content.split("\n").collect();

    // check that the file has at least the right number of lines
    if content.len() < 7 {
        return Err(InvalidInputError {});
    }

    // get the config
    let n_decks = content[1].parse::<u8>()?;
    let n_jokers = content[2].parse::<u8>()?;
    let n_cards_to_start = content[3].parse::<u16>()?;
    let custom_rule_jokers = content[4] == "1";
    let n_players = content[5].parse::<u8>()?;
    let savefile = content[6];
   
    // print the parameters
    println!("{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}",
             "Number of decks",
             n_decks,
             "Number of jokers",
             n_jokers,
             "Number of starting cards",
             n_cards_to_start,
             "Jokers can't be kept",
             custom_rule_jokers,
             "Number of players",
             n_players);

    Ok((Config {
        n_decks,
        n_jokers,
        n_cards_to_start,
        custom_rule_jokers,
        n_players
    }, savefile.to_string()))
}

/// ask the user for the game information and savefile name
pub fn get_config_and_savefile() -> Result<(Config, String),InvalidInputError> {
    let conf = get_config()?;
    println!("Name of the save file: ");
    let savefile = get_input()?.trim().to_string();
    Ok((conf, savefile))
}

/// ask the user for the game information and return a Config
pub fn get_config() -> Result<Config,InvalidInputError> {
    
    println!("Number of decks (integer between 1 and 255) (enter 0 to load a previously saved game): ");
    let mut n_decks: u8 = 0;
    let mut load = false;
    while n_decks == 0 {
        n_decks = match get_input()?.trim().parse::<u8>() {
            Ok(0) => {
                load = true;
                1
            },
            Ok(n) => n,
            Err(_) => {
                println!("Invalid input");
                0
            }
        };
    }

    if load {
        return Ok(Config {
            n_decks: 0,
            n_jokers: 0,
            n_cards_to_start: 0,
            custom_rule_jokers: false,
            n_players: 0
        });
    }
    
    println!("Number of jokers (integer between 0 and 255): ");
    let mut n_jokers: u8 = 0; 
    let mut set = false;
    while !set {
        n_jokers = match get_input()?.trim().parse::<u8>() {
            Ok(n) => {
                set = true;
                n
            },
            Err(_) => {
                println!("Invalid input");
                0
            }
        };
    }
    
    println!("Number of cards to start with (integer): ");
    let mut n_cards_to_start: u16 = 0;
    while n_cards_to_start == 0 {
        n_cards_to_start = match get_input()?.trim().parse::<u16>() {
            Ok(n) => {
                let mut res = 0;
                if n==0 {
                    println!("You need to start with at least one card");
                } else if n > ((52 * (n_decks as u16)) + (n_jokers as u16)) {
                    println!("You can't draw more cards than there are in the deck");
                } else {
                    res = n;
                }
                res
            },
            Err(_) => return Err(InvalidInputError {})
        };
    }
    
    println!("Custom rule—jokers must be played immediately (y/n): ");
    let custom_rule_jokers = match get_input()?.trim() {
        "y" => true,
        _ => false
    };
    
    println!("Number of players: ");
    let mut n_players = 0;
    while n_players == 0 {
        n_players = match get_input()?.trim().parse::<u8>() {
            Ok(0) => {
                println!("I need at least one player!");
                0
            }
            Ok(n) => n,
            Err(_) => {
                println!("Could not parse the input");
                0
            }
        };
    }

    Ok(Config {
        n_decks, 
        n_jokers,
        n_cards_to_start,
        custom_rule_jokers,
        n_players
    })
}

fn instructions() -> String {
    format!("{}\n{}\n{}\n{}\n{}\n{}\n",
        "0: Save and quit",
        "1: Pick a card",
        "2: Play a sequence",
        "3: Take from the table",
        "4: Pass",
        "5, 6: Sort cards by rank or suit"
        )
}

pub fn instructions_no_save() -> String {
    format!("{}\n{}\n{}\n{}\n",
        "e: End your turn",
        "p: Play a sequence",
        "t: Take from the table",
        "r, s: Sort cards by rank or suit"
        )
}

pub fn player_turn(table: &mut Table, hand: &mut Sequence, deck: &mut Sequence, 
                   custom_rule_jokers: bool, player_name: &String) -> bool {

    // copy the initial hand
    let hand_start_round = hand.clone();

    // get the player choice
    let mut message = String::new();
    loop {
        
        // clear the terminal
        clear_terminal();
        
        println!("\x1b[1m{}'s turn", player_name);
        reset_style();
        
        print_situation(table, hand, deck);

        // print the options
        println!("{}", &instructions());
        
        if message.len() > 0 {
            println!("\n{}", message);
            message.clear()
        }
        
        match get_input().unwrap_or_else(|_| {"".to_string()})
              .trim().parse::<u16>() {
            Ok(0) => {
                if !hand_start_round.contains(hand) {
                    message = "You can't save until you've played all the cards you've taken from the table!".to_string();
                } else if !hand.contains(&hand_start_round) {
                    message = "You need to pass before saving".to_string();
                } else {
                    return true;
                }
            },
            Ok(1) => {
                if !hand_start_round.contains(hand) {
                    message = "You can't pick a card until you've played all the cards you've taken from the table!".to_string();
                } else if !hand.contains(&hand_start_round) {
                    message = "You can't pick a card after having played something".to_string();
                } else if custom_rule_jokers && hand.contains_joker() {
                    message = "Jokers need to be played!".to_string();
                } else {
                    match pick_a_card(hand, deck) {
                        Ok(card) => println!("You have picked a {}\x1b[38;2;0;0;0;1m", &card),
                        Err(_) => println!("No more card to draw!")
                    };
                    break
                }
            },
            Ok(2) => {
                message = play_sequence(hand, table);
                print_situation(table, hand, deck);
            },
            Ok(3) => {
                message = take_sequence(table, hand);
                print_situation(table, hand, deck);
            },
            Ok(4) => {
                if !hand_start_round.contains(hand) {
                    message = "You can't pass until you've played all the cards you've taken from the table!".to_string();
                } else if hand.contains(&hand_start_round) {
                    message = "You need to play something to pass".to_string();
                } else if custom_rule_jokers && hand.contains_joker() {
                    message = "Jokers need to be played!".to_string();
                } else {
                    break
                }
            }
            Ok(5) => {
                hand.sort_by_rank();
                print_situation(table, hand, deck);
            },
            Ok(6) => {
                hand.sort_by_suit();
                print_situation(table, hand, deck);
            },
            _ => ()
        };
    }

    false
}


fn print_situation(table: &Table, hand: &Sequence, deck: &Sequence) {
    
    // print the table
    println!("\nTable:\n{}", table);

    // print the player hand
    println!("Your hand:\n{}", hand);
    reset_style();

    // print the number of remaining cards in the deck
    println!("\nRemaining cards in the deck: {}", deck.number_cards());

}

pub fn situation_to_string(table: &Table, hand: &Sequence, deck: &Sequence) -> String {
    
    format!("\n{}\n{}\n{}\n{}{}\n\n{}{}\n", 
            "Table:", table, "Your hand:", hand, reset_style_string(),
            "Remaining cards in the deck: ", deck.number_cards())
}

pub fn get_input() -> Result<String, InvalidInputError> {
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_) => (),
        Err(_) => return Err(InvalidInputError {})
    }
    Ok(buffer)
}


fn pick_a_card(hand: &mut Sequence, deck: &mut Sequence) -> Result<Card, NoMoreCards> {
    let card = match deck.draw_card() {
        Some(c) => c,
        None => return Err(NoMoreCards {})
    };
    hand.add_card(card.clone());
    Ok(card)
}


fn play_sequence(hand: &mut Sequence, table: &mut Table) -> String {
    println!("Please enter the sequence, in order, separated by spaces");
    let hand_and_indices = hand.show_indices();
    println!("{}", hand_and_indices.0);
    reset_style();
    println!("{}", hand_and_indices.1);
    let mut seq = Sequence::new();
    
    let mut s = get_input().unwrap_or_else(|_| {"".to_string()});
    s.pop();
    let mut seq_i = Vec::<usize>::new();
    for item in s.split(' ') {
        match item.parse::<usize>() {
            Ok(n) => {
                let mut n_i = 0;
                for &i in &seq_i {
                    if i < n {
                        n_i += 1;
                    }
                }
                let card = match hand.take_card(n-n_i) {
                    Some(c) => c,
                    None => continue
                };
                seq.add_card(card);
                seq_i.push(n);
            },
            Err(_) => ()
        }
    }

    if seq.is_valid() {
        table.add(seq);
        return String::new();
    } else {
        let message = format!("{} is not a valid sequence!", &seq);
        hand.merge(seq);
        return message;
    }
}


fn take_sequence(table: &mut Table, hand: &mut Sequence) -> String {
    println!("Which sequence would you like to take?");
    match get_input().unwrap_or_else(|_| {"".to_string()})
          .trim().parse::<usize>() {
        Ok(n) => match table.take(n) {
            Some(seq) => {
                hand.merge(seq);
                return String::new();
            },
            None => return "This sequence is not on the table".to_string()
        },
        Err(_) => return "Error parsing the input!".to_string()
    };
}

/// convert the game info to a sequence of bytes
pub fn game_to_bytes (player: u8, table: &Table, hands: &Vec<Sequence>, 
                      deck: &Sequence, config: &Config, player_names: &Vec<String>) -> Vec<u8> {
    
    // construct the sequence of bytes to be saved
    let mut bytes = Vec::<u8>::new();
    
    // config
    bytes.append(&mut config.to_bytes());
    
    // player about to play
    bytes.push(player);
    
    // hand of each player
    for i_player in 0..config.n_players {
        
        // number of cards in the hand as 2 u8
        let n_cards_in_hand = hands[i_player as usize].number_cards() as u16;
        bytes.push((n_cards_in_hand >> 8) as u8);
        bytes.push((n_cards_in_hand & 255) as u8);
        
        // append the hand
        bytes.append(&mut hands[i_player as usize].to_bytes());
    }

    // player names
    for i_player in 0..config.n_players {
        let name_b = player_names[i_player as usize].as_bytes();
        bytes.push(name_b.len() as u8);
        bytes.append(&mut name_b.to_vec());
    }
    
    // deck 
    let n_cards_in_deck = deck.number_cards();
    bytes.push((n_cards_in_deck >> 8) as u8);
    bytes.push((n_cards_in_deck & 255) as u8);
    bytes.append(&mut deck.to_bytes());
    
    // table 
    bytes.append(&mut table.to_bytes());

    bytes
}

/// load the game info from a sequence of bytes
pub fn load_game(bytes: &[u8]) -> Result<(Config, u8, Table, Vec<Sequence>, Sequence, Vec<String>), LoadingError> {
    let mut i_byte: usize = 0; // index of the current element in bytes

    // load the config
    let n_bytes_config: usize = 6;
    let config = Config::from_bytes(&bytes[0..n_bytes_config]);
    i_byte += n_bytes_config;
    
    // load the current player
    let player = bytes[i_byte];
    i_byte += 1;
    
    // hand of each player
    let mut hands = Vec::<Sequence>::new();
    for _i_player in 0..config.n_players {
        
        // number of cards in the hand as 2 u8
        let n_cards_in_hand = ((bytes[i_byte] as usize) << 8) + (bytes[i_byte+1] as usize);
        i_byte += 2;
 
        // append the hand
        hands.push(Sequence::from_bytes(&bytes[i_byte..i_byte+n_cards_in_hand]));
        i_byte += n_cards_in_hand;
    }
    
    // player names
    let mut player_names = Vec::<String>::new();
    for _i_player in 0..config.n_players {
        
        // number of characters in the name
        let n_chars = bytes[i_byte] as usize;
        i_byte += 1;
        
        // append the name
        player_names.push(String::from_utf8(bytes[i_byte..i_byte+n_chars].to_vec()).unwrap());
        i_byte += n_chars;
    }

    // deck
    let n_cards_in_deck = ((bytes[i_byte] as usize) << 8) + (bytes[i_byte+1] as usize);
    i_byte += 2;
    let deck = Sequence::from_bytes(&bytes[i_byte..i_byte+n_cards_in_deck]);
    i_byte += n_cards_in_deck;

    // table
    let table = Table::from_bytes(&bytes[i_byte..]);

    Ok((
        config,
        player,
        table,
        hands,
        deck,
        player_names
    ))
}

#[derive(Debug)]
pub struct InvalidInputError {}

impl<T: std::error::Error> From<T> for InvalidInputError {
    fn from(_error: T) -> Self {
        InvalidInputError {}
    }
}

pub struct NoMoreCards {}
pub struct LoadingError {}


