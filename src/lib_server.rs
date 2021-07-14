//! Library for the game server

pub use super::*;
pub use std::io::{ stdin, Read, Write };
pub use std::net::{ TcpListener, TcpStream, Shutdown };
pub use std::thread::JoinHandle;
pub use std::str::from_utf8;

const BUFFER_SIZE: usize = 50;
const MAX_N_BUFFERS: usize = 255;
const N_MILLISECONDS_WAIT: u64 = 10;
const N_MILLISECONDS_LONG_WAIT: u64 = 1000;

pub fn handle_client(mut stream: TcpStream) -> (TcpStream, String) {
    let mut player_name: String = "".to_string();
    match get_str_from_client(&mut stream) {
        Ok(s) => {
            // echo the stream data
            player_name = s.clone();
            let msg = format!("Hello {}!\nWaiting for other players to join...", &s);
            send_str_to_client(&mut stream, &msg).unwrap();
        },
        Err(_)=> {
            println!("An error occured while reading the stream; terminating connection with {}", 
                     stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
        }
    };
    (stream, player_name)
}

// TO IMPLEMENT
pub fn start_player_turn(table: &mut Table, hand: &mut Sequence, deck: &mut Sequence, 
                         custom_rule_jokers: bool, player_name: &String, stream: &mut TcpStream)
    -> Result<bool,StreamError> {
    
    // copy the initial hand
    let hand_start_round = hand.clone();
    
    // send the instructions
    send_message_to_client(stream, &instructions())?;

    // get and process the player choice
    loop {
        match get_message_from_client(stream) {
            Ok(mes) => match mes[0] {
                // value '1'
                49 => {
                    let message: String;
                    if !hand_start_round.contains(hand) {
                        message = "You can't pick a card until you've played all the cards you've taken from the table!".to_string();
                        send_message_to_client(stream, &message);
                    } else if !hand.contains(&hand_start_round) {
                        message = "You can't pick a card after having played something".to_string();
                        send_message_to_client(stream, &message);
                    } else if custom_rule_jokers && hand.contains_joker() {
                        message = "Jokers need to be played!".to_string();
                        send_message_to_client(stream, &message);
                    } else {
                        match pick_a_card(hand, deck) {
                            Ok(card) => message = format!("You have picked a {}\x1b[38;2;0;0;0;1m", &card),
                            Err(_) => message = "No more card to draw!".to_string()
                        };
                        send_message_to_client(stream, &message);
                        break
                    }
                },
                _ => send_message_to_client(stream, &"Invalid input; please try again.")?,
            },
            Err(_) => {
                send_message_to_client(stream, &"Could not get your input. Please try again.")?;
            }
        };
    }
    Ok(false)
}

pub fn send_str_to_client(stream: &mut TcpStream, s: &str) -> Result<(), StreamError> {
    send_bytes_to_client(stream, &s.as_bytes())?;
    Ok(())
}

pub fn send_bytes_to_client(stream: &mut TcpStream, bytes: &[u8]) -> Result<(), StreamError> {
    
    // ensure that the number of bytes is small enough
    if bytes.len() > MAX_N_BUFFERS * BUFFER_SIZE {
        return Err(StreamError { message: format!(
                    "Stream too long: size: {}, maximum size: {}",
                    bytes.len(), MAX_N_BUFFERS*BUFFER_SIZE
                   ) })
    }

    // the first bytes will determine the number of times the buffer should be read
    let mut n_buffers: u8 = (bytes.len() / BUFFER_SIZE) as u8;
    if bytes.len() % BUFFER_SIZE != 0 {
        n_buffers += 1;
    }
    stream.write(&[n_buffers])?;

    // write the data stream
    for i in 0..((n_buffers-1) as usize) {
        stream.write(&bytes[i*BUFFER_SIZE..(i+1)*BUFFER_SIZE])?;
    }
    stream.write(&bytes[((n_buffers-1) as usize)*BUFFER_SIZE..])?;
    
    // wait for a reply to be sent from the receiver
    while let Err(_) = stream.read_exact(&mut [0]) {}
    
    Ok(())
}

pub fn get_str_from_client(stream: &mut TcpStream) -> Result<String, StreamError> {
    let bytes = get_bytes_from_client(stream)?;
    match String::from_utf8(bytes) {
        Ok(s) => Ok(s),
        Err(_) => Err(StreamError::from(BytesToStringError {}))
    }
}

pub fn get_bytes_from_client(stream: &mut TcpStream) -> Result<Vec<u8>, StreamError> {
    
    // buffer
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    // the first bytes will determine the number of times the buffer should be read
    let mut n_buffers: [u8; 1] = [0];
    stream.read(&mut n_buffers)?;

    // vector containing the result
    let mut res = Vec::<u8>::new();

    // read the data stream
    let mut size;
    for _i in 0..n_buffers[0] {
        size = stream.read(&mut buffer)?;
        res.extend_from_slice(&buffer[..size]);
    }
    
    // send something to confirm I have received the data
    stream.write(&[0])?;
    
    // return the result
    Ok(res)
}

/// wait a moment
pub fn wait() {
    std::thread::sleep(std::time::Duration::from_millis(N_MILLISECONDS_WAIT));
}

/// wait a longer moment
pub fn long_wait() {
    std::thread::sleep(std::time::Duration::from_millis(N_MILLISECONDS_LONG_WAIT));
}

/// check that no players have the same name; if yes, rename players
pub fn ensure_names_are_different(player_names: &mut Vec<String>, client_streams: &mut Vec<TcpStream>) {
    let mut cont = true;
    while cont {
        cont = false;
        for i in 0..player_names.len() {
            for j in (i+1)..player_names.len() {
                if player_names[j] == player_names[i] {
                    cont = true;
                    player_names[j] = format!("{}_", &player_names[j]);
                    client_streams[j].write(&mut [1]).unwrap();
                    send_str_to_client(&mut client_streams[j], 
                                       &format!("Your name is already taken! You were renamed as {}\n", 
                                               &player_names[j])).unwrap();
                }
            }
        }
    }
}

fn get_message_from_client(stream: &mut TcpStream) -> Result<Vec<u8>, StreamError>{
    stream.write(&mut [4])?;
    get_bytes_from_client(stream)
}

fn send_message_to_client(stream: &mut TcpStream, msg: &str) -> Result<(), StreamError>{
    stream.write(&mut [1])?;
    send_str_to_client(stream, msg)
}

/// send a message and get the output 
pub fn send_message_get_reply(stream: &mut TcpStream, message: &str) 
    -> Result<Vec<u8>, StreamError>
{
    stream.write(&mut [3])?;
    send_str_to_client(stream, message)?;
    get_bytes_from_client(stream)
}

/// send the same message to all players
pub fn send_message_all_players(client_streams: &mut Vec<TcpStream>, message: &str) -> Result<(),StreamError> {
    for mut stream in client_streams {
        stream.write(&mut [1])?;
        send_str_to_client(&mut stream, message)?;
    }
    Ok(())
}

/// clear the screens and send the same message to all players
pub fn clear_and_send_message_all_players(client_streams: &mut Vec<TcpStream>, message: &str) 
    -> Result<(),StreamError> {
    for mut stream in client_streams {
        stream.write(&mut [2])?;
        send_str_to_client(&mut stream, message)?;
    }
    Ok(())
}

// errors

#[derive(Debug)]
pub struct StreamError {
    message: String
}

#[derive(Debug)]
pub struct BytesToStringError {}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "StreamError: {}", self.message)
    }
}

impl std::convert::From<std::io::Error> for StreamError {
    fn from(error: std::io::Error) -> Self {
        StreamError { message: format!("IO Error: {}", error) }
    }
}

impl std::convert::From<BytesToStringError> for StreamError {
    fn from(_error: BytesToStringError) -> Self {
        StreamError { message: "Could not convert the byte sequence to a string".to_string() }
    }
}
