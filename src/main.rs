use bevy::prelude::*;
use bevy_svg::prelude::*;
use hermanha_chess::{BOARD_COLS, BOARD_ROWS, Board, Piece as HermanhaPiece, PieceType, Position};

const TILE_SIZE: f32 = 64.0;
const PIECE_SCALE: f32 = TILE_SIZE / 45.0;
const PIECE_Z: f32 = 1.0;
const BOARD_OFFSET: f32 = (BOARD_COLS as f32 - 1.0) * 0.5;

#[derive(Component, Debug, Clone, Copy)]
#[require(Transform, Sprite)]
struct Square {
    pos: Position,
}

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
#[require(Transform, Sprite)]
struct Piece {
    type_: PieceType,
    color: hermanha_chess::Color,
}

fn pos_to_vec3(pos: Position, z: f32) -> Vec3 {
    Vec3::new(
        (pos.col as f32 - BOARD_OFFSET) * TILE_SIZE,
        (pos.row as f32 - BOARD_OFFSET) * TILE_SIZE,
        z,
    )
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SvgPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let board = Board::start_pos();

    for row in 0..BOARD_ROWS as usize {
        for col in 0..BOARD_COLS as usize {
            let render_pos = Position::new(row as i8, col as i8);

            spawn_square(&mut commands, render_pos);

            if let Some(piece) = board.squares[row][col] {
                spawn_piece(&mut commands, &asset_server, piece, render_pos);
            }
        }
    }
}

fn is_dark_square(pos: Position) -> bool {
    (pos.row + pos.col) % 2 == 0
}

fn spawn_square(commands: &mut Commands, pos: Position) {
    let color = if is_dark_square(pos) {
        Color::srgb(0.62, 0.42, 0.32)
    } else {
        Color::srgb(0.87, 0.81, 0.74)
    };

    commands.spawn((
        Square { pos },
        Sprite {
            color,
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(pos_to_vec3(pos, 0.0)),
    ));
}

fn spawn_piece(
    commands: &mut Commands,
    asset_server: &AssetServer,
    piece: HermanhaPiece,
    pos: Position,
) {
    let svg = match (piece.color, piece.piece_type) {
        (hermanha_chess::Color::White, PieceType::Pawn) => {
            asset_server.load("pieces/Chess_plt45.svg")
        }
        (hermanha_chess::Color::White, PieceType::Rook) => {
            asset_server.load("pieces/Chess_rlt45.svg")
        }
        (hermanha_chess::Color::White, PieceType::Knight) => {
            asset_server.load("pieces/Chess_nlt45.svg")
        }
        (hermanha_chess::Color::White, PieceType::Bishop) => {
            asset_server.load("pieces/Chess_blt45.svg")
        }
        (hermanha_chess::Color::White, PieceType::Queen) => {
            asset_server.load("pieces/Chess_qlt45.svg")
        }
        (hermanha_chess::Color::White, PieceType::King) => {
            asset_server.load("pieces/Chess_klt45.svg")
        }
        (hermanha_chess::Color::Black, PieceType::Pawn) => {
            asset_server.load("pieces/Chess_pdt45.svg")
        }
        (hermanha_chess::Color::Black, PieceType::Rook) => {
            asset_server.load("pieces/Chess_rdt45.svg")
        }
        (hermanha_chess::Color::Black, PieceType::Knight) => {
            asset_server.load("pieces/Chess_ndt45.svg")
        }
        (hermanha_chess::Color::Black, PieceType::Bishop) => {
            asset_server.load("pieces/Chess_bdt45.svg")
        }
        (hermanha_chess::Color::Black, PieceType::Queen) => {
            asset_server.load("pieces/Chess_qdt45.svg")
        }
        (hermanha_chess::Color::Black, PieceType::King) => {
            asset_server.load("pieces/Chess_kdt45.svg")
        }
    };

    commands.spawn((
        Piece {
            type_: piece.piece_type,
            color: piece.color,
        },
        Svg2d(svg),
        Origin::Center,
        Transform {
            translation: pos_to_vec3(pos, PIECE_Z),
            scale: Vec3::splat(PIECE_SCALE),
            ..default()
        },
    ));
}
