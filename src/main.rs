use core::fmt;
use std::{fs, path::PathBuf};

use bevy::{
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    text::{BreakLineOn, Text2dBounds},
};
use grid::{Grid, GridSize};

const CELL_SIZE: Vec2 = Vec2::new(60.0, 60.0);
const SPACE_BETWEEN_CELLS: f32 = 5.0;

mod grid;

#[derive(Resource)]
pub struct PuzzlePaths(Vec<PathBuf>);

#[derive(Resource)]
pub struct Puzzle {
    pub game_grid: Grid,
    solution_grid: Grid,
}

#[derive(Resource, PartialEq, Eq, Clone)]
pub enum GameState {
    Playing,
    Won,
    Menu,
}

#[derive(Event, PartialEq, Eq)]
pub struct ChangeGameState(GameState);

#[derive(Component)]
pub struct Cursor;

#[derive(Component)]
pub struct WinSprite;

#[derive(Component)]
pub struct Cell(CellState);

#[derive(Component, PartialEq, Eq)]
pub struct GridComponent {
    pub row: usize,
    pub col: usize,
}

#[derive(Component, PartialEq, Eq, Copy, Clone, Debug)]
pub enum CellState {
    Blank,
    Island,
    River,
    Value(i8),
}

impl CellState {
    pub fn next(&self) -> CellState {
        match self {
            CellState::Blank => CellState::River,
            CellState::Island => CellState::Blank,
            CellState::River => CellState::Island,
            _ => *self,
        }
    }

    pub fn is_same(&self, other: CellState) -> bool {
        match self {
            CellState::Blank | CellState::Value(_) | CellState::Island => other != CellState::River,
            CellState::River => other == CellState::River,
        }
    }
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            CellState::Blank => ' ',
            CellState::Island => '.',
            CellState::River => 'X',
            CellState::Value(v) => char::from_u32((*v as u32) + 48).unwrap(),
        };
        write!(f, "{}", c)
    }
}

impl Into<usize> for CellState {
    fn into(self) -> usize {
        match self {
            CellState::Blank => 0,
            CellState::Island => 10,
            CellState::River => 11,
            CellState::Value(v) => v as usize,
        }
    }
}

impl GridComponent {
    pub fn new(row: usize, col: usize) -> Self {
        GridComponent { row, col }
    }

    pub fn splat(val: usize) -> Self {
        GridComponent { row: val, col: val }
    }

    pub fn clamp(&self, grid_size: &GridSize) -> GridComponent {
        GridComponent::new(
            self.row.clamp(0, grid_size.rows - 1),
            self.col.clamp(0, grid_size.cols - 1),
        )
    }
}

fn get_offset(grid_size: &GridSize) -> Vec2 {
    -Vec2::new(
        (grid_size.cols - 1) as f32 / 2.0 * (CELL_SIZE.x + SPACE_BETWEEN_CELLS),
        (grid_size.rows - 1) as f32 / 2.0 * (CELL_SIZE.y + SPACE_BETWEEN_CELLS),
    )
}

/// Close the focused window when both menu buttons are pressed.
fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }
        if keys.any_pressed([KeyCode::KeyQ, KeyCode::Escape]) {
            commands.entity(window).despawn();
        }
    }
}

fn load_puzzle(mut commands: Commands) {
    if let Ok(files) = fs::read_dir("./assets/puzzles") {
        let mut puzzles = Vec::new();
        for path in files {
            if let Ok(path) = path {
                if let Some(extension) = path.path().extension() {
                    if extension == "txt" {
                        puzzles.push(path.path());
                    }
                }
            }
        }

        let path = puzzles.get(0).unwrap();
        println!("{:?}", path);
        if let Ok(puzzle_str) = fs::read_to_string(path.clone()) {
            if let Ok(solution_str) = fs::read_to_string(path.with_extension("txt.text")) {
                commands.insert_resource(Puzzle {
                    game_grid: Grid::from_puzzle_string(puzzle_str),
                    solution_grid: Grid::from_solution_string(solution_str.clone()),
                });
                println!("{}", Grid::from_solution_string(solution_str));
            }
        }
        // dbg!(puzzles.clone());
        commands.insert_resource(PuzzlePaths(puzzles));
    }
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    puzzle: Res<Puzzle>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let grid = &puzzle.game_grid;
    // camera
    commands.spawn(Camera2dBundle::default());

    // instructions
    let font = asset_server.load("FiraSans-Regular.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 30.0,
        ..default()
    };
    let instruction_text ="Move the cursor with WASD/arrow keys, and press space to toggle the selected cell.\nEach numbered cell is an island cell, the number in it is the number of cells in that island.\nEach island must contain exactly one numbered cell.\nThere must be only one sea, which is not allowed to contain \"pools\", i.e. 2x2 areas of black cells.";
    let box_size = Vec2::new(240.0, 1200.0);
    let box_pos = Vec2::new(-500.0, 00.0);
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(box_size.x, box_size.y)),
                ..default()
            },
            transform: Transform::from_translation(box_pos.extend(0.0)),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(instruction_text, text_style.clone())],
                    justify: JustifyText::Left,
                    linebreak_behavior: BreakLineOn::WordBoundary,
                },
                text_2d_bounds: Text2dBounds {
                    // Wrap text in the rectangle
                    size: box_size,
                },
                // ensure the text is drawn on top of the box
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            });
        });

    // cursor
    commands.spawn((
        SpriteBundle {
            texture: asset_server
                .load_with_settings("cursor.png", |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest()
                }),
            transform: Transform {
                scale: (CELL_SIZE / 16.0).extend(1.0),
                ..default()
            },
            ..default()
        },
        Cursor,
        GridComponent::splat(0),
    ));

    let texture = asset_server
        .load_with_settings("tile_sheet.png", |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest()
        });
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        3,
        4,
        Some(UVec2::splat(2)),
        Some(UVec2::splat(1)),
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let grid_size = grid.grid_size;
    let offset = get_offset(&grid_size);

    // grid
    for row in 0..grid_size.rows {
        for column in 0..grid_size.cols {
            let brick_position = Vec2::new(
                offset.x + column as f32 * (CELL_SIZE.x + SPACE_BETWEEN_CELLS),
                offset.y + row as f32 * (CELL_SIZE.y + SPACE_BETWEEN_CELLS),
            );

            // cell
            commands.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: brick_position.extend(0.0),
                        scale: (CELL_SIZE / 16.0).extend(1.0),
                        ..default()
                    },
                    texture: texture.clone(),
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    ..default()
                },
                Cell(grid.get(row, column)),
                GridComponent::new(row, column),
            ));
        }
    }

    commands.insert_resource(grid_size);
    commands.insert_resource(GameState::Playing);
}

fn update_cursor_location(
    mut cursor: Query<(&mut Transform, &GridComponent), With<Cursor>>,
    grid_size: Res<GridSize>,
) {
    let (mut transform, location) = cursor.single_mut();
    let offset = get_offset(&grid_size);
    transform.translation = Vec3::new(
        offset.x + location.col as f32 * (CELL_SIZE.x + SPACE_BETWEEN_CELLS),
        offset.y + location.row as f32 * (CELL_SIZE.y + SPACE_BETWEEN_CELLS),
        1.0,
    );
}

fn reset_puzzle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut tile_query: Query<&mut Cell>,
    mut puzzle: ResMut<Puzzle>,
    game_state: Res<GameState>,
) {
    if *game_state != GameState::Playing {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for mut tile in &mut tile_query {
            tile.0 = match tile.0 {
                CellState::Blank | CellState::Island | CellState::River => CellState::Blank,
                CellState::Value(_) => tile.0,
            };
        }
        for row in 0..puzzle.game_grid.grid_size.rows {
            for col in 0..puzzle.game_grid.grid_size.cols {
                let tile = puzzle.game_grid.get(row, col);
                puzzle.game_grid.set(
                    &GridComponent::new(row, col),
                    match tile {
                        CellState::Blank | CellState::Island | CellState::River => CellState::Blank,
                        CellState::Value(_) => tile,
                    },
                );
            }
        }
    }
}

fn toggle_cell(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor_query: Query<&GridComponent, With<Cursor>>,
    mut tile_query: Query<(&mut Cell, &GridComponent)>,
    mut puzzle: ResMut<Puzzle>,
    game_state: Res<GameState>,
    mut change_game_state_ev: EventWriter<ChangeGameState>,
) {
    if *game_state != GameState::Playing {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        let cursor_loc = cursor_query.single();
        for (mut cell, tile_loc) in &mut tile_query {
            let next_state = cell.0.next();
            if cursor_loc == tile_loc {
                cell.0 = next_state;
                puzzle.game_grid.set(cursor_loc, next_state);
                // check puzzle solved
                println!("{}", puzzle.game_grid.check(&puzzle.solution_grid));
                if puzzle.game_grid.check(&puzzle.solution_grid) {
                    change_game_state_ev.send(ChangeGameState(GameState::Won));
                }
                break;
            }
        }
    }
}

fn update_game_state(
    mut game_state: ResMut<GameState>,
    mut change_game_state_ev: EventReader<ChangeGameState>,
) {
    for ev in change_game_state_ev.read() {
        *game_state = ev.0.clone();
    }
}

fn game_win(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // game_state: Res<GameState>,
    mut change_game_state_ev: EventReader<ChangeGameState>,
) {
    for ev in change_game_state_ev.read() {
        if ev.0 == GameState::Won {
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load_with_settings(
                        "tada.png",
                        |settings: &mut ImageLoaderSettings| {
                            settings.sampler = ImageSampler::nearest()
                        },
                    ),
                    transform: Transform {
                        scale: (CELL_SIZE / 4.0).extend(1.0),
                        translation: Vec3::new(0.0, 0.0, 2.0),
                        ..default()
                    },
                    ..default()
                },
                WinSprite,
            ));
        }
    }
}

fn update_cell(mut tile_query: Query<(&mut TextureAtlas, &Cell)>) {
    for (mut texture_atlas, cell) in &mut tile_query {
        texture_atlas.index = cell.0.into();
    }
}

fn move_cursor(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cursor: Query<&mut GridComponent, With<Cursor>>,
    grid_size: Res<GridSize>,
    game_state: Res<GameState>,
) {
    if *game_state != GameState::Playing {
        return;
    }
    let mut location = cursor.single_mut();
    let mut temp = IVec2 {
        x: location.row as i32,
        y: location.col as i32,
    };

    if keyboard_input.any_just_released(vec![KeyCode::ArrowLeft, KeyCode::KeyA]) {
        temp.y -= 1;
    }
    if keyboard_input.any_just_released(vec![KeyCode::ArrowRight, KeyCode::KeyD]) {
        temp.y += 1;
    }
    if keyboard_input.any_just_released(vec![KeyCode::ArrowUp, KeyCode::KeyW]) {
        temp.x += 1;
    }
    if keyboard_input.any_just_released(vec![KeyCode::ArrowDown, KeyCode::KeyS]) {
        temp.x -= 1;
    }
    *location = GridComponent::new(
        temp.x.clamp(0, (grid_size.rows - 1) as i32) as usize,
        temp.y.clamp(0, (grid_size.cols - 1) as i32) as usize,
    );
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (load_puzzle, setup).chain())
        .add_event::<ChangeGameState>()
        .add_systems(
            Update,
            (
                close_on_esc,
                update_cursor_location,
                move_cursor,
                toggle_cell,
                reset_puzzle,
                update_cell,
                update_game_state,
                game_win,
            ),
        )
        .run();
}
