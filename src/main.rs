use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
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

#[derive(Resource, Deref)]
struct BoardState(Board);

#[derive(Resource, Default)]
struct SelectedSquare(Option<Position>);

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

fn cursor_to_board_position(
    cursor_position: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Position> {
    let world_position = camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .ok()?;
    let col = ((world_position.x / TILE_SIZE) + BOARD_OFFSET).round() as i8;
    let row = ((world_position.y / TILE_SIZE) + BOARD_OFFSET).round() as i8;
    Some(Position::new(row, col))
}

fn selected_legal_targets(board: &Board, selected: Option<Position>) -> Vec<Position> {
    selected
        .map(|pos| {
            board
                .legal_moves()
                .into_iter()
                .filter_map(|(from, to, _)| (from == pos).then_some(to))
                .collect()
        })
        .unwrap_or_default()
}

fn square_color_for_state(
    pos: Position,
    selected: Option<Position>,
    legal_targets: &[Position],
) -> Color {
    if Some(pos) == selected {
        Color::srgb(0.86, 0.87, 0.35)
    } else if legal_targets.contains(&pos) {
        Color::srgb(0.72, 0.82, 0.46)
    } else {
        square_color(pos)
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SvgPlugin))
        .insert_resource(BoardState(Board::start_pos()))
        .init_resource::<SelectedSquare>()
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (handle_square_selection, render_board))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn render_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board: Res<BoardState>,
    selected: Res<SelectedSquare>,
    squares: Query<Entity, With<Square>>,
    pieces: Query<Entity, With<Piece>>,
    mut initialized: Local<bool>,
) {
    if *initialized && !board.is_changed() && !selected.is_changed() {
        return;
    }

    *initialized = true;

    for entity in squares.iter().chain(pieces.iter()) {
        commands.entity(entity).despawn();
    }

    let selected_pos = selected.0;
    let legal_targets = selected_legal_targets(&board, selected_pos);

    for row in 0..BOARD_ROWS as usize {
        for col in 0..BOARD_COLS as usize {
            let render_pos = Position::new(row as i8, col as i8);

            let color = square_color_for_state(render_pos, selected_pos, &legal_targets);
            spawn_square(&mut commands, render_pos, color);

            if let Some(piece) = board.squares[row][col] {
                spawn_piece(&mut commands, &asset_server, piece, render_pos);
            }
        }
    }
}

fn handle_square_selection(
    mut selected: ResMut<SelectedSquare>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    board: Res<BoardState>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some((camera, camera_transform)) = camera_q.iter().next() else {
        return;
    };

    let Some(position) = cursor_to_board_position(cursor_position, camera, camera_transform) else {
        return;
    };

    if !board.pos_on_board(position) {
        selected.0 = None;
        return;
    }

    if board.get(position).is_some() {
        selected.0 = Some(position);
    } else {
        selected.0 = None;
    }
}

fn is_dark_square(pos: Position) -> bool {
    (pos.row + pos.col) % 2 == 0
}

fn square_color(pos: Position) -> Color {
    if is_dark_square(pos) {
        Color::srgb(0.62, 0.42, 0.32)
    } else {
        Color::srgb(0.87, 0.81, 0.74)
    }
}

fn spawn_square(commands: &mut Commands, pos: Position, color: Color) {
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
