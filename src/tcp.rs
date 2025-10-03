use std::fmt;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

use hermanha_chess::{Board, Color, GameResult, PieceType, Position};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    Server,
    Client,
}

pub struct MoveMessage {
    pub from: Position,
    pub to: Position,
    pub promotion_piece: Option<PieceType>,
    pub result: Option<GameResult>,
    pub new_board: Board,
}

impl MoveMessage {
    fn to_string(&self) -> String {
        let mut ret = "ChessMOVE:".to_string();
        ret.push_str(&move_to_string(self.from, self.to, self.promotion_piece));
        ret.push(':');
        ret.push_str(&game_result_to_string(self.result));
        ret.push(':');
        ret.push_str(&board_to_fen(&self.new_board));
        ret.push(':');
        add_padding(&mut ret);
        ret
    }

    fn from_string(msg_str: String) -> Result<Self, String> {
        if msg_str.len() != 128 {
            return Err("Message must be 128 characters".to_string());
        }
        let parts: Vec<&str> = msg_str.split(':').collect();
        if parts.len() != 5 {
            return Err("Invalid message format".to_string());
        }
        let Ok((from, to, promotion_piece)) = move_from_string(parts[1]) else {
            return Err("Invalid move format".to_string());
        };
        let Ok(result) = game_result_from_string(parts[2]) else {
            return Err("Invalid result format".to_string());
        };
        let mut board = Board::start_pos();
        board.setup_fen(parts[3]);

        Ok(Self {
            from,
            to,
            promotion_piece,
            result,
            new_board: board,
        })
    }
}

fn pos_to_string(pos: Position) -> String {
    let file = match pos.col {
        0 => 'A',
        1 => 'B',
        2 => 'C',
        3 => 'D',
        4 => 'E',
        5 => 'F',
        6 => 'G',
        7 => 'H',
        _ => panic!("Invalid column"),
    };
    let rank = (pos.row + 1).to_string();
    format!("{}{}", file, rank)
}

fn pos_from_string(pos_str: &str) -> Result<Position, String> {
    let file = pos_str.chars().nth(0).ok_or("Invalid position string")?;
    let rank = pos_str.chars().nth(1).ok_or("Invalid position string")?;
    let row = rank.to_digit(10).ok_or("Invalid rank")? as i8 - 1;
    let col = match file {
        'A' => 0,
        'B' => 1,
        'C' => 2,
        'D' => 3,
        'E' => 4,
        'F' => 5,
        'G' => 6,
        'H' => 7,
        _ => return Err("Invalid file".to_string()),
    };
    Ok(Position::new(row, col))
}

fn piece_type_to_char(piece_type: PieceType) -> char {
    match piece_type {
        PieceType::Pawn => 'P',
        PieceType::Knight => 'N',
        PieceType::Bishop => 'B',
        PieceType::Rook => 'R',
        PieceType::Queen => 'Q',
        PieceType::King => 'K',
    }
}

fn char_to_piece_type(c: char) -> Result<PieceType, String> {
    match c {
        'P' => Ok(PieceType::Pawn),
        'N' => Ok(PieceType::Knight),
        'B' => Ok(PieceType::Bishop),
        'R' => Ok(PieceType::Rook),
        'Q' => Ok(PieceType::Queen),
        'K' => Ok(PieceType::King),
        _ => Err("Invalid piece type".to_string()),
    }
}

fn move_to_string(from: Position, to: Position, promotion_piece: Option<PieceType>) -> String {
    let from_str = pos_to_string(from);
    let to_str = pos_to_string(to);
    let promotion_str = if let Some(piece_type) = promotion_piece {
        piece_type_to_char(piece_type).to_string()
    } else {
        "0".to_string()
    };
    format!("{}{}{}", from_str, to_str, promotion_str)
}

fn move_from_string(move_str: &str) -> Result<(Position, Position, Option<PieceType>), String> {
    if move_str.len() != 5 {
        return Err("Invalid move string".to_string());
    }
    let from_str = &move_str[0..2];
    let from = pos_from_string(from_str)?;
    let to_str = &move_str[2..4];
    let to = pos_from_string(to_str)?;
    let promotion_char = move_str.chars().nth(4).unwrap();
    let promotion_piece = if promotion_char != '0' {
        char_to_piece_type(promotion_char).ok()
    } else {
        None
    };
    Ok((from, to, promotion_piece))
}

fn game_result_to_string(result: Option<GameResult>) -> String {
    match result {
        Some(GameResult::Checkmate(Color::White)) => "1-0",
        Some(GameResult::Checkmate(Color::Black)) => "0-1",
        Some(GameResult::Stalemate) => "1-1",
        None => "0-0",
    }
    .to_string()
}

fn game_result_from_string(s: &str) -> Result<Option<GameResult>, String> {
    match s {
        "1-0" => Ok(Some(GameResult::Checkmate(Color::White))),
        "0-1" => Ok(Some(GameResult::Checkmate(Color::Black))),
        "1-1" => Ok(Some(GameResult::Stalemate)),
        "0-0" => Ok(None),
        _ => Err("Invalid game result".to_string()),
    }
}

fn board_to_fen(board: &Board) -> String {
    let mut ret = String::new();
    for row in 0..8 {
        let mut empty_count = 0;
        for col in 0..8 {
            let Some(piece) = board.get(Position::new(7 - row, col)) else {
                empty_count += 1;
                continue;
            };
            if empty_count != 0 {
                ret.push_str(&format!("{empty_count}"));
                empty_count = 0;
            }
            let piece_type_char = piece_type_to_char(piece.piece_type);
            let piece_char = match piece.color {
                Color::White => piece_type_char.to_ascii_uppercase(),
                Color::Black => piece_type_char.to_ascii_lowercase(),
            };
            ret.push(piece_char);
        }
        if empty_count != 0 {
            ret.push_str(&format!("{empty_count}"));
        }
        if row != 7 {
            ret.push('/');
        }
    }
    ret
}

pub struct QuitMessage {
    pub message: Option<String>,
}

impl QuitMessage {
    fn to_string(&self) -> String {
        let msg = match &self.message {
            Some(msg) => msg.clone(),
            None => String::new(),
        };
        let mut ret = format!("ChessQUIT:{}:", msg);
        add_padding(&mut ret);
        ret
    }

    fn from_string(msg_str: String) -> Result<Self, String> {
        if msg_str.len() != 128 {
            return Err("Message must be 128 characters".to_string());
        }
        let parts: Vec<&str> = msg_str.split(':').collect();
        let msg = if parts.len() == 3 {
            Some(parts[1].to_string())
        } else {
            None
        };
        Ok(QuitMessage { message: msg })
    }
}

fn add_padding(str: &mut String) {
    let padding = "0".repeat(128 - str.len());
    str.push_str(&padding);
}

pub enum Message {
    Move(MoveMessage),
    Quit(QuitMessage),
}

#[derive(Debug)]
pub enum TcpError {
    WouldBlock,
    InvalidMessage(String),
    Io(io::Error),
}

impl fmt::Display for TcpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TcpError::WouldBlock => write!(f, "operation would block"),
            TcpError::InvalidMessage(msg) => write!(f, "invalid message: {msg}"),
            TcpError::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl Message {
    fn to_string(&self) -> String {
        match self {
            Message::Move(move_msg) => move_msg.to_string(),
            Message::Quit(quit_msg) => quit_msg.to_string(),
        }
    }

    fn from_string(msg_str: String) -> Result<Self, String> {
        if msg_str.len() != 128 {
            return Err("Message must be 128 characters".to_string());
        }
        let identifier = &msg_str[0..9];
        match identifier {
            "ChessMOVE" => {
                MoveMessage::from_string(msg_str).map(|move_msg| Message::Move(move_msg))
            }
            "ChessQUIT" => {
                QuitMessage::from_string(msg_str).map(|quit_msg| Message::Quit(quit_msg))
            }
            _ => Err(format!("Invalid message identifier")),
        }
    }
}

pub struct TcpConnection {
    stream: TcpStream,
}

impl TcpConnection {
    pub fn start_server(address: &str) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(address)?;
        let (stream, _) = listener.accept()?;
        stream
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");
        Ok(TcpConnection { stream: stream })
    }

    pub fn connect_to_server(address: &str) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(address)?;
        stream
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");
        Ok(TcpConnection {
            #[rustfmt::skip]
            stream: stream,
        })
    }

    pub fn read(&mut self) -> Result<Message, TcpError> {
        let mut buffer = [0; 128];
        match self.stream.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                return Err(TcpError::WouldBlock);
            }
            Err(err) => return Err(TcpError::Io(err)),
        }
        let msg_str = String::from_utf8_lossy(&buffer).to_string();
        Message::from_string(msg_str).map_err(|e| TcpError::InvalidMessage(e))
    }

    pub fn write(&mut self, message: Message) -> Result<(), TcpError> {
        match self.stream.write_all(message.to_string().as_bytes()) {
            Ok(_) => Ok(()),
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => Err(TcpError::WouldBlock),
            Err(err) => Err(TcpError::Io(err)),
        }
    }
}
