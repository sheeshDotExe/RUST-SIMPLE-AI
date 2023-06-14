use crate::cell::Cell;
use crate::cell::Food;
use crate::network::Network;
use rand::Error;
use rand::{rngs::ThreadRng, Rng};
use rstar::primitives::Rectangle;
use rstar::RTree;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::video::Window;
use sdl2::{Sdl, VideoSubsystem};
use std::time::Duration;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

pub struct GameObjects {
    cells: Vec<Cell>,
    food_tree: RTree<Food>,
}
pub struct GameState {
    context: Sdl,
    video_subsystem: VideoSubsystem,
    game_objects: GameObjects,
    columns: i32,
    rows: i32,
}

pub fn game_init(columns: usize, rows: usize, rng: &mut ThreadRng) -> Result<GameState, String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut game_state = GameState {
        context: sdl_context,
        video_subsystem: video_subsystem,
        game_objects: GameObjects {
            cells: Vec::with_capacity(columns * rows),
            food_tree: RTree::new(),
        },
        columns: columns as i32,
        rows: rows as i32,
    };

    for x in 0..columns {
        for y in 0..rows {
            let random_value = rng.gen_range(0.0..=1.0);
            let alive = random_value < 0.15;
            if alive {
                game_state
                    .game_objects
                    .cells
                    .push(Cell::new(x as i32, y as i32, alive, rng));
            } else if random_value > 0.90 {
                let food = Food::new(x as i32, y as i32);
                game_state.game_objects.food_tree.insert(food);
            }
        }
    }

    return Ok(game_state);
}

fn get_food_collision(
    x: i32,
    y: i32,
    direction: &Vec<i32>,
    width: i32,
    height: i32,
    food_tree: &RTree<Food>,
) -> f64 {
    let mut iterations = 1.0;
    let mut next_x = x;
    let mut next_y = y;
    loop {
        next_x += direction[0];
        next_y += direction[1];

        if next_x < 0 {
            break;
        }
        if next_x >= width {
            break;
        }
        if next_y < 0 {
            break;
        }
        if next_y >= height {
            break;
        }

        let food_collision = food_tree.locate_at_point(&[next_x, next_y]);

        match food_collision {
            Some(_) => {
                return 1.0 / iterations;
            }
            None => {}
        }

        iterations += 1.0;
        if iterations > 9.0 {
            return 0.0;
        }
    }
    return 0.0;
}

fn do_game_tick(game_state: &mut GameState, rng: &mut ThreadRng) -> Result<i32, String> {
    let mut dead_cells = Vec::<usize>::new();

    let mut new_cells = Vec::<Cell>::new();

    for (i, cell) in game_state.game_objects.cells.iter_mut().enumerate() {
        if !cell.alive {
            continue;
        }

        let prev_x = cell.x;
        let prev_y = cell.y;

        cell.ticks_since_food += 1;

        let mut input_values = Vec::<f64>::with_capacity(8);

        let directions = vec![
            vec![1, 0],
            vec![-1, 0],
            vec![1, 0],
            vec![-1, 0],
            vec![1, 1],
            vec![-1, 1],
            vec![1, -1],
            vec![-1, -1],
        ];

        for direction in directions.iter() {
            input_values.push(get_food_collision(
                cell.x,
                cell.y,
                direction,
                game_state.columns,
                game_state.rows,
                &game_state.game_objects.food_tree,
            ))
        }

        let network_result = cell.network.feed_forward(input_values);

        let x_movement = network_result[0] * 2.0 - 1.0;
        let y_movement = network_result[1] * 2.0 - 1.0;

        let mut new_x = cell.x;
        let mut new_y = cell.y;

        if x_movement.abs() > 0.25 {
            if x_movement < 0.0 {
                // left
                if cell.x > 0 {
                    new_x -= 1;
                } else {
                    new_x = game_state.columns - 1;
                }
            } else {
                //right
                if cell.x < game_state.columns {
                    new_x += 1;
                } else {
                    new_x = 0;
                }
            }
        }
        if y_movement.abs() > 0.25 {
            if y_movement < 0.0 {
                //down
                if cell.y > 0 {
                    new_y -= 1;
                } else {
                    new_y = game_state.rows - 1;
                }
            } else {
                if cell.y < game_state.rows {
                    new_y += 1;
                } else {
                    new_y = 0;
                }
            }
        }

        cell.x = new_x;
        cell.y = new_y;

        let food_collision = game_state
            .game_objects
            .food_tree
            .locate_at_point(&[cell.x, cell.y]);

        match food_collision {
            Some(_) => {
                cell.ticks_since_food = 0;
                game_state
                    .game_objects
                    .food_tree
                    .remove_at_point(&[cell.x, cell.y]);

                new_cells.push(Cell::inherit_from(&cell, prev_x, prev_y, rng));
            }
            None => {}
        }

        if cell.ticks_since_food > 10 {
            dead_cells.push(i);
            game_state.game_objects.food_tree.insert(Food {
                x: rng.gen_range(0..game_state.columns),
                y: rng.gen_range(0..game_state.rows),
            })
        }
    }

    for (i, dead_cell_index) in dead_cells.iter().enumerate() {
        game_state.game_objects.cells.remove(*dead_cell_index - i);
    }

    for _ in 0..new_cells.len() {
        let cell = new_cells.pop().unwrap();
        game_state.game_objects.cells.push(cell);
    }

    return Ok(0);
}

fn render(canvas: &mut WindowCanvas, game_state: &mut GameState) -> Result<i32, String> {
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    for cell in game_state.game_objects.cells.iter() {
        if !cell.alive {
            continue;
        }

        canvas.set_draw_color(cell.color);

        let x = (WIDTH as f32 / game_state.columns as f32 * cell.x as f32) as i32;
        let width = (WIDTH as f32 / game_state.columns as f32) as u32;
        let y = (HEIGHT as f32 / game_state.rows as f32 * cell.y as f32) as i32;
        let height = (HEIGHT as f32 / game_state.rows as f32) as u32;

        canvas.fill_rect(Rect::new(x, y, width, height))?;
    }

    canvas.set_draw_color(Color::RGB(0, 255, 0));

    for food in game_state.game_objects.food_tree.iter() {
        let x = (WIDTH as f32 / game_state.columns as f32 * food.x as f32) as i32;
        let width = (WIDTH as f32 / game_state.columns as f32) as u32;
        let y = (HEIGHT as f32 / game_state.rows as f32 * food.y as f32) as i32;
        let height = (HEIGHT as f32 / game_state.rows as f32) as u32;

        canvas.fill_rect(Rect::new(x, y, width, height))?;
    }

    canvas.set_draw_color(Color::RGB(0, 0, 0));

    for i in 0..game_state.columns {
        canvas.draw_line(
            Point::new(
                (WIDTH as f32 / game_state.columns as f32 * i as f32) as i32,
                0,
            ),
            Point::new(
                (WIDTH as f32 / game_state.columns as f32 * i as f32) as i32,
                HEIGHT,
            ),
        )?;
    }

    for i in 0..game_state.rows {
        canvas.draw_line(
            Point::new(
                0,
                (HEIGHT as f32 / game_state.rows as f32 * i as f32) as i32,
            ),
            Point::new(
                WIDTH,
                (HEIGHT as f32 / game_state.rows as f32 * i as f32) as i32,
            ),
        )?;
    }

    canvas.present();

    return Ok(0);
}

pub fn run_game(game_state: GameState, rng: &mut ThreadRng) -> Result<i32, String> {
    let window = game_state
        .video_subsystem
        .window("rust game of life", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    let mut game_state = game_state;

    let mut event_pump = game_state.context.event_pump()?;
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        let _game_tick = do_game_tick(&mut game_state, rng).unwrap();

        let _render_status = render(&mut canvas, &mut game_state).unwrap();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 10));
    }

    return Ok(0);
}
