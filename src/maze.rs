use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Maze {
    grid: Vec<Vec<char>>,
}

impl Maze {
    pub fn load(filename: &str) -> std::io::Result<Maze> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let grid: Vec<Vec<char>> = reader
            .lines()
            .filter_map(Result::ok)
            .map(|line| line.chars().collect())
            .collect();

        Ok(Maze { grid })
    }

    pub fn get_wall(&self, x: f32, y: f32) -> Option<char> {
        let xi = x as usize;
        let yi = y as usize;
        if yi < self.grid.len() && xi < self.grid[yi].len() {
            let ch = self.grid[yi][xi];
            if ch != ' ' {
                Some(ch)
            } else {
                None
            }
        } else {
            None
        }
    }
}
