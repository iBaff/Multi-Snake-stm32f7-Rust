#![no_std]
#![no_main]
#![feature(compiler_builtins_lib)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy))]

extern crate arrayvec;
extern crate compiler_builtins;
extern crate r0;
extern crate stm32f7_discovery as stm32f7; // initialization routines for .data and .bss

use alloc::Vec;
use graphics;
use stm32f7::{lcd, touch};

use super::HEIGHT;
use super::WIDTH;

const GRID_BLOCK_SIZE: usize = 10;

/**
 * Contains all necessary state information of a game.
 */
pub struct Game {
    pub graphics: graphics::Graphics,
    grid: Vec<Vec<Tile>>,
    i2c_3: stm32f7::i2c::I2C,
    pub snake_head_position: (usize, usize),
    pub snake_body_position: Vec<(usize, usize)>,
    snake_tail_position: (usize, usize),
    former_snake_tail: (usize, usize),
    apple_position: (usize, usize),
}

/**
 * Possible tiles inside of the game grid.
 */
#[derive(PartialEq, Clone)]
enum Tile {
    Empty,
    SnakeHead(Direction),
    SnakeBody(Direction),
    SnakeTail(Direction),
    Apple,
}
#[derive(PartialEq, Clone)]
enum Direction {
    up,
    down,
    left,
    right,
}

impl Game {
    /**
     * Create a new game.
     */
    pub fn new(graphics: graphics::Graphics, i2c_3: stm32f7::i2c::I2C) -> Game {
        let game_width = WIDTH / GRID_BLOCK_SIZE;
        let game_height = HEIGHT / GRID_BLOCK_SIZE;
        let mut return_game = Game {
            graphics: graphics,
            grid: vec![vec![Tile::Empty; game_height]; game_width],
            i2c_3: i2c_3,
            snake_head_position: (25, 10),
            snake_body_position: vec![(24, 10), (23, 10), (22, 10)],
            snake_tail_position: (21, 10),
            former_snake_tail: (20, 10),
            apple_position: (10, 10),
        };
        return_game.grid[25][10] = Tile::SnakeHead(Direction::right);
        return_game
    }

    /**
     * Draws current game state to screen.
     */
    pub fn draw_game(&mut self) {
        // draw head (bmp of head)
        self.graphics.print_square_size_color_at(
            self.snake_head_position.0 * GRID_BLOCK_SIZE,
            self.snake_head_position.1 * GRID_BLOCK_SIZE,
            GRID_BLOCK_SIZE - 1,
            lcd::Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        );
        // draw body (bmp of body)
        for i in 0..self.snake_body_position.len() {
            self.graphics.print_square_size_color_at(
                self.snake_body_position[i].0 * GRID_BLOCK_SIZE,
                self.snake_body_position[i].1 * GRID_BLOCK_SIZE,
                GRID_BLOCK_SIZE - 1,
                lcd::Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
            );
        }

        // draw tail (bmp of tail)
        self.graphics.print_square_size_color_at(
            self.snake_tail_position.0 * GRID_BLOCK_SIZE,
            self.snake_tail_position.1 * GRID_BLOCK_SIZE,
            GRID_BLOCK_SIZE - 1,
            lcd::Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        );

        // erase former tail (bmp of tail)
        if self.former_snake_tail != (0, 0) {
            self.graphics.print_square_size_color_at(
                self.former_snake_tail.0 * GRID_BLOCK_SIZE,
                self.former_snake_tail.1 * GRID_BLOCK_SIZE,
                GRID_BLOCK_SIZE - 1,
                lcd::Color {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 0,
                },
            );
        }

        // draw apple (bmp of apple)

        self.graphics.print_square_size_color_at(
            self.apple_position.0 * GRID_BLOCK_SIZE,
            self.apple_position.1 * GRID_BLOCK_SIZE,
            GRID_BLOCK_SIZE - 1,
            lcd::Color {
                red: 0,
                green: 255,
                blue: 0,
                alpha: 255,
            },
        );
    }

    /**
     * Moves position of the snake in chosen direction.
     */
    fn move_up(&mut self) {
        let x = self.snake_head_position.0;
        let y = self.snake_head_position.1;

        self.grid[x][y - 1] = Tile::SnakeHead(Direction::up);
        self.grid[x][y] = Tile::Empty;
        self.former_snake_tail = self.snake_tail_position;
        self.snake_tail_position = self.snake_body_position[self.snake_body_position.len() - 1];
        for z in (0..self.snake_body_position.len() - 1).rev() {
            self.snake_body_position[z + 1] = self.snake_body_position[z];
        }
        self.snake_body_position[0] = self.snake_head_position;
        self.snake_head_position.1 = y - 1;

        return;
    }

    /**
     * Moves position of the snake in chosen direction.
     */
    fn move_down(&mut self) {
        let x = self.snake_head_position.0;
        let y = self.snake_head_position.1;

        self.grid[x][y + 1] = Tile::SnakeHead(Direction::down);
        self.grid[x][y] = Tile::Empty;
        self.former_snake_tail = self.snake_tail_position;
        self.snake_tail_position = self.snake_body_position[self.snake_body_position.len() - 1];
        for z in (0..self.snake_body_position.len() - 1).rev() {
            self.snake_body_position[z + 1] = self.snake_body_position[z];
        }
        self.snake_body_position[0] = self.snake_head_position;
        self.snake_head_position.1 = y + 1;

        return;
    }

    /**
     * Moves position of the snake in chosen direction.
     */
    fn move_right(&mut self) {
        let x = self.snake_head_position.0;
        let y = self.snake_head_position.1;

        self.grid[x + 1][y] = Tile::SnakeHead(Direction::right);
        self.grid[x][y] = Tile::Empty;
        self.former_snake_tail = self.snake_tail_position;
        self.snake_tail_position = self.snake_body_position[self.snake_body_position.len() - 1];
        for z in (0..self.snake_body_position.len() - 1).rev() {
            self.snake_body_position[z + 1] = self.snake_body_position[z];
        }
        self.snake_body_position[0] = self.snake_head_position;
        self.snake_head_position.0 = x + 1;

        return;
    }

    /**
     * Moves position of the snake in chosen direction.
     */
    fn move_left(&mut self) {
        let x = self.snake_head_position.0;
        let y = self.snake_head_position.1;

        self.grid[x - 1][y] = Tile::SnakeHead(Direction::left);
        self.grid[x][y] = Tile::Empty;
        self.former_snake_tail = self.snake_tail_position;
        self.snake_tail_position = self.snake_body_position[self.snake_body_position.len() - 1];
        for z in (0..self.snake_body_position.len() - 1).rev() {
            self.snake_body_position[z + 1] = self.snake_body_position[z];
        }
        self.snake_body_position[0] = self.snake_head_position;
        self.snake_head_position.0 = x - 1;

        return;
    }

    /**
     * Calls the correct function to turn to move the snake straight forward
     */

    pub fn move_straight(&mut self) {
        if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::up)
        {
            self.move_up();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::down)
        {
            self.move_down();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::left)
        {
            self.move_left();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::right)
        {
            self.move_right();
        }
    }

    /**
     * Calls the correct function to turn the snake to the right
     */

    pub fn turn_right(&mut self) {
        if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::up)
        {
            self.move_right();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::down)
        {
            self.move_left();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::left)
        {
            self.move_up();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::right)
        {
            self.move_down();
        }
    }

    /**
     * Calls the correct function to turn the snake to the left
     */

    pub fn turn_left(&mut self) {
        if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::up)
        {
            self.move_left();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::down)
        {
            self.move_right();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::left)
        {
            self.move_down();
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::right)
        {
            self.move_up();
        }
    }

    /**
     * Sets the direction chosen by the user
     */
    pub fn move_snake(&mut self) {
        let touches = self.get_touches();
        if touches.len() == 1 {
            for touch in touches {
                let mut x = touch.0;
                let mut y = touch.1;

                if x < 70 {
                    self.turn_left();
                } else if x > 410 {
                    self.turn_right();
                }
            }
        } else {
            self.move_straight();
        }
    }

    /**
     * checks if snake bites
     */
    pub fn snake_bite(&mut self) {
        if self.snake_head_position == self.apple_position {
            self.snake_body_position.push(self.snake_tail_position);
            self.snake_tail_position = self.former_snake_tail;
            self.former_snake_tail = (0, 0); // has to be improved
                                             // missing random apple position
        }
    }

    /**
     * returns touches array
     */
    pub fn get_touches(&mut self) -> Vec<(u16, u16)> {
        // &touch::touches(&mut self.i2c_3).unwrap()
        let mut touches = Vec::new();
        for touch in &touch::touches(&mut self.i2c_3).unwrap() {
            // .print_point_at(touch.x as usize, touch.y as usize);
            touches.push((touch.x, touch.y));
        }
        touches
    }

    /**
     * returns touches array
     */
    pub fn check_grid_edge(&mut self) {
        if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::right) && self.snake_head_position.0 == 46
        {
            self.grid[self.snake_head_position.0][self.snake_head_position.1] = Tile::Empty;
            self.snake_head_position.0 = 1;
            self.grid[self.snake_head_position.0][self.snake_head_position.1] =
                Tile::SnakeHead(Direction::right);
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::left) && self.snake_head_position.0 == 1
        {
            self.grid[self.snake_head_position.0][self.snake_head_position.1] = Tile::Empty;
            self.snake_head_position.0 = 46;
            self.grid[self.snake_head_position.0][self.snake_head_position.1] =
                Tile::SnakeHead(Direction::left);
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::up) && self.snake_head_position.1 == 1
        {
            self.grid[self.snake_head_position.0][self.snake_head_position.1] = Tile::Empty;
            self.snake_head_position.1 = 26;
            self.grid[self.snake_head_position.0][self.snake_head_position.1] =
                Tile::SnakeHead(Direction::up);
        } else if self.grid[self.snake_head_position.0][self.snake_head_position.1]
            == Tile::SnakeHead(Direction::down)
            && self.snake_head_position.1 == 26
        {
            self.grid[self.snake_head_position.0][self.snake_head_position.1] = Tile::Empty;
            self.snake_head_position.1 = 1;
            self.grid[self.snake_head_position.0][self.snake_head_position.1] =
                Tile::SnakeHead(Direction::down);
        }
    }
    /**
     * set backround color
     */
    pub fn set_backround_color(&mut self) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                self.graphics.print_square_size_color_at(
                    x,
                    y,
                    1,
                    lcd::Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 255,
                    },
                );
            }
        }
    }
}
