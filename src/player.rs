use raylib::prelude::*;
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32,
    speed: f32,
    rot_speed: f32,
    collision_radius: f32,
}

impl Player {
    pub fn new() -> Self {
        Player {
            pos: Vector2 { x: 3.5, y: 3.5 },
            a: 0.0,
            fov: std::f32::consts::FRAC_PI_3, // 60 grados
            speed: 20.0,
            rot_speed: 10.0,
            collision_radius: 0.2,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, maze: &Maze) {
        let dt = 1.0 / 60.0;

        // Rotación
        if rl.is_key_down(KeyboardKey::KEY_A) {
            self.a -= self.rot_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            self.a += self.rot_speed * dt;
        }

        // Rotación con el mouse
        let mouse_delta = rl.get_mouse_delta();
        if mouse_delta.x != 0.0 {
            self.a += mouse_delta.x * 0.003;
        }

        // Movimiento frontal
        let mut dx = 0.0;
        let mut dy = 0.0;

        if rl.is_key_down(KeyboardKey::KEY_W) {
            dx += self.a.cos();
            dy += self.a.sin();
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            dx -= self.a.cos();
            dy -= self.a.sin();
        }

        // Movimiento lateral (strafe)
        if rl.is_key_down(KeyboardKey::KEY_E) {
            dx += (self.a + std::f32::consts::FRAC_PI_2).cos();
            dy += (self.a + std::f32::consts::FRAC_PI_2).sin();
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            dx += (self.a - std::f32::consts::FRAC_PI_2).cos();
            dy += (self.a - std::f32::consts::FRAC_PI_2).sin();
        }

        // Normalizar el vector de movimiento si no es cero
        if dx != 0.0 || dy != 0.0 {
            let length = (dx * dx + dy * dy).sqrt();
            dx /= length;
            dy /= length;
        }

        // Calcular nueva posición
        let new_x = self.pos.x + dx * self.speed * dt;
        let new_y = self.pos.y + dy * self.speed * dt;

        // Comprobar colisiones en X
        if !self.check_collision(new_x, self.pos.y, maze) {
            self.pos.x = new_x;
        }

        // Comprobar colisiones en Y
        if !self.check_collision(self.pos.x, new_y, maze) {
            self.pos.y = new_y;
        }
    }

    fn check_collision(&self, x: f32, y: f32, maze: &Maze) -> bool {
        // Comprobar colisiones en varios puntos alrededor del jugador
        let check_points = [
            (x + self.collision_radius, y),
            (x - self.collision_radius, y),
            (x, y + self.collision_radius),
            (x, y - self.collision_radius),
            (x + self.collision_radius * 0.7, y + self.collision_radius * 0.7),
            (x - self.collision_radius * 0.7, y + self.collision_radius * 0.7),
            (x + self.collision_radius * 0.7, y - self.collision_radius * 0.7),
            (x - self.collision_radius * 0.7, y - self.collision_radius * 0.7),
        ];

        for (check_x, check_y) in check_points.iter() {
            if maze.get_wall(*check_x, *check_y).is_some() {
                return true;
            }
        }

        false
    }
}
