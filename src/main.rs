// main.rs
use bevy::prelude::*;

const TILE_SIZE: f32 = 64.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_board)
        .run();
}

fn is_dark_square(row: i32, col: i32) -> bool {
    (row + col) % 2 == 0
}

fn all_positions() -> [(i32, i32); 64] {
    let mut ret = [(0, 0); 64];
    for row in 1..=8 {
        for col in 1..=8 {
            let index = (row - 1) * 8 + (col - 1);
            ret[index as usize] = (col, row);
        }
    }
    ret
}

fn setup_board(mut commands: Commands) {
    // 2D camera
    commands.spawn(Camera2d::default());

    for (row, col) in all_positions() {
        let color = if is_dark_square(row, col) {
            Color::BLACK
        } else {
            Color::WHITE
        };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            Transform::from_xyz(
                (col as f32 - 4.5) * TILE_SIZE,
                (row as f32 - 4.5) * TILE_SIZE,
                0.0,
            ),
        ));
    }
}
