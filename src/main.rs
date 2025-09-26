use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_svg::prelude::*;
use hermanha_chess::{BOARD_COLS, BOARD_ROWS, Board, Piece as HermanhaPiece, PieceType, Position};

const TILE_SIZE: f32 = 64.0;
const PIECE_SCALE: f32 = TILE_SIZE / 45.0;
const PIECE_Z: f32 = 1.0;
const BOARD_OFFSET: f32 = (BOARD_COLS as f32 - 1.0) * 0.5;

#[derive(Resource, Deref)]
struct BoardState(Board);

#[derive(Resource, Default)]
struct SelectedSquare(Option<Position>);

#[derive(Component, Debug, Clone, Copy)]
#[require(Transform, Sprite)]
struct Piece {
    type_: PieceType,
    color: hermanha_chess::Color,
}

#[derive(Component)]
struct Highlight;

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
    let Some(selected_pos) = selected else {
        return Vec::new();
    };
    board
        .legal_moves()
        .into_iter()
        .filter(|(from, to, _)| *from == selected_pos)
        .map(|(from, to, _)| to)
        .collect()
}

fn square_color(pos: Position) -> Color {
    if (pos.row + pos.col) % 2 == 0 {
        Color::srgb(0.62, 0.42, 0.32)
    } else {
        Color::srgb(0.87, 0.81, 0.74)
    }
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
        .add_systems(Startup, (setup_camera, render_board))
        .add_systems(Update, (handle_square_selection, render_highlights, render_pieces))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn render_board(mut commands: Commands, asset_server: Res<AssetServer>) {
    let board = Board::start_pos();
    let squares = board.squares;

    for row in 0..BOARD_ROWS as usize {
        for col in 0..BOARD_COLS as usize {
            let render_pos = Position::new(row as i8, col as i8);

            let color = square_color(render_pos);
            spawn_square(&mut commands, render_pos, color);
        }
    }
}

fn render_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board: Res<BoardState>,
    selected: Res<SelectedSquare>,
    pieces: Query<Entity, With<Piece>>,
    mut initialized: Local<bool>,
) {
    for entity in pieces.iter().chain(pieces.iter()) {
        commands.entity(entity).despawn();
    }

    let selected_pos = selected.0;
    let board = &board.0;
    let legal_targets = selected_legal_targets(&board, selected_pos);

    for row in 0..BOARD_ROWS as usize {
        for col in 0..BOARD_COLS as usize {
            let render_pos = Position::new(row as i8, col as i8);
            let square = board.get(render_pos);
            if let Some(piece) = square {
                spawn_piece(&mut commands, &asset_server, piece, render_pos);
            }
        }
    }
}

fn render_highlights(
    mut commands: Commands,
    board: Res<BoardState>,
    selected: Res<SelectedSquare>,
    highlights: Query<Entity, With<Highlight>>,
) {
    for entity in highlights.iter() {
        commands.entity(entity).despawn();
    }

    let selected_pos = selected.0;
    let board = &board.0;
    let legal_targets = selected_legal_targets(board, selected_pos);

    for target in legal_targets {
        spawn_highlight(&mut commands, target);
    }
}

fn handle_square_selection(
    mut selected: ResMut<SelectedSquare>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut board: ResMut<BoardState>,
) {
    let board = &mut board.0;
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
    if let Some(moving_pos) = selected.0 {
        _ = board.play(
            (moving_pos.row, moving_pos.col),
            (position.row, position.col),
            None,
        );
        selected.0 = None;
        return;
    }

    selected.0 = Some(position);
}

fn spawn_square(commands: &mut Commands, pos: Position, color: Color) {
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(pos_to_vec3(pos, 0.0)),
    ));
}

fn spawn_highlight(commands: &mut Commands, pos: Position) {
    commands.spawn((
        Highlight,
        Sprite {
            color: Color::srgba(0.72, 0.82, 0.46, 0.6),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(pos_to_vec3(pos, 0.5)),
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
