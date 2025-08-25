use raylib::prelude::*;
use std::{thread, time::Duration};

mod framebuffer;
mod maze;
mod player;
mod textures;

use framebuffer::Framebuffer;
use maze::Maze;
use player::Player;
use textures::TextureManager;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;
const NUM_RAYS: usize = SCREEN_WIDTH as usize;
const MAX_DEPTH: f32 = 20.0;
const STEP: f32 = 0.01;
const MINIMAP_SIZE: i32 = 150;
const MINIMAP_SCALE: f32 = 10.0;

// Estados del juego
enum GameState {
    Welcome,
    Playing,
    Success,
}

// Estructura para la animación
struct Animation {
    frames: Vec<Texture2D>,
    current_frame: usize,
    frame_time: f32,
    timer: f32,
}

impl Animation {
    fn new(frames: Vec<Texture2D>, frame_time: f32) -> Self {
        Animation {
            frames,
            current_frame: 0,
            frame_time,
            timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= self.frame_time {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.timer = 0.0;
        }
    }

    fn current_texture(&self) -> &Texture2D {
        &self.frames[self.current_frame]
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Raycaster Textured")
        .build();

    // Mouse para rotación
    rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_CROSSHAIR);
    rl.hide_cursor();
    rl.enable_cursor();
    rl.set_mouse_position(Vector2::new((SCREEN_WIDTH / 2) as f32, (SCREEN_HEIGHT / 2) as f32));

    // Audio
    let audio = RaylibAudio::init_audio_device().expect("No se pudo inicializar el audio");
    let mut music = audio
        .new_music("assets/background_music.mp3")
        .expect("No se pudo cargar la música");
    let step_sound = audio
        .new_sound("assets/step.wav")
        .expect("No se pudo cargar el sonido");

    music.play_stream();
    music.set_volume(0.7);

    let mut framebuffer =
        Framebuffer::new(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize, Color::DARKBLUE);

    // Niveles
    let levels = vec!["maze.txt", "mazetky.txt"];
    let mut current_level = 0;
    let mut maze =
        Maze::load(levels[current_level]).expect("No se pudo abrir el archivo de laberinto");

    let mut player = Player::new();
    let textures = TextureManager::new(&mut rl, &thread);

    // Animación estrella
    let star_frames = vec![
        rl.load_texture(&thread, "assets/star1.png")
            .expect("No se pudo cargar la textura"),
        rl.load_texture(&thread, "assets/star2.png")
            .expect("No se pudo cargar la textura"),
        rl.load_texture(&thread, "assets/star3.png")
            .expect("No se pudo cargar la textura"),
    ];
    let mut star_animation = Animation::new(star_frames, 0.2);

    // Pantallas
    let welcome_texture = rl
        .load_texture(&thread, "assets/welcome.png")
        .expect("No se pudo cargar la textura");
    let success_texture = rl
        .load_texture(&thread, "assets/success.png")
        .expect("No se pudo cargar la textura");

    let mut game_state = GameState::Welcome;
    let mut selected_level = 0;

    let mut last_time = rl.get_time();
    let mut fps_counter = 0;
    let mut fps = 0;
    let mut fps_timer = 0.0;

    // Cooldown para pasos (evita spam de sonido)
    let mut step_timer = 0.0_f32; // segundos

    while !rl.window_should_close() {
        // Tiempo
        let current_time = rl.get_time();
        let dt = (current_time - last_time) as f32;
        last_time = current_time;

        // Música
        music.update_stream();

        // FPS
        fps_counter += 1;
        fps_timer += dt;
        if fps_timer >= 1.0 {
            fps = fps_counter;
            fps_counter = 0;
            fps_timer = 0.0;
        }

        // Animación
        star_animation.update(dt);

        match game_state {
            GameState::Welcome => {
                // --- INPUT (sin dibujar) ---
                if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                    selected_level = if selected_level > 0 {
                        selected_level - 1
                    } else {
                        levels.len() - 1
                    };
                }
                if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                    selected_level = (selected_level + 1) % levels.len();
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    current_level = selected_level;
                    maze = Maze::load(levels[current_level])
                        .expect("No se pudo abrir el archivo de laberinto");
                    player = Player::new();
                    game_state = GameState::Playing;
                }

                {
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    d.draw_texture(&welcome_texture, 0, 0, Color::WHITE);
                    d.draw_text("DOOM ARCAICO", SCREEN_WIDTH / 2 - 150, 100, 40, Color::RED);
                    d.draw_text(
                        "Selecciona un nivel:",
                        SCREEN_WIDTH / 2 - 120,
                        200,
                        20,
                        Color::WHITE,
                    );

                    for (i, level_name) in levels.iter().enumerate() {
                        let color = if i == selected_level {
                            Color::YELLOW
                        } else {
                            Color::WHITE
                        };
                        d.draw_text(
                            &format!("Nivel {}: {}", i + 1, level_name),
                            SCREEN_WIDTH / 2 - 100,
                            250 + i as i32 * 30,
                            20,
                            color,
                        );
                    }

                    d.draw_text(
                        "Presiona ARRIBA/ABAJO para seleccionar nivel",
                        SCREEN_WIDTH / 2 - 200,
                        400,
                        20,
                        Color::WHITE,
                    );
                    d.draw_text(
                        "Presiona ENTER para comenzar",
                        SCREEN_WIDTH / 2 - 150,
                        430,
                        20,
                        Color::WHITE,
                    );
                }
            }

            GameState::Playing => {
                // Sonido de pasos simple con cooldown si hay movimiento WASD
                step_timer = (step_timer - dt).max(0.0);
                if (rl.is_key_down(KeyboardKey::KEY_W)
                    || rl.is_key_down(KeyboardKey::KEY_A)
                    || rl.is_key_down(KeyboardKey::KEY_S)
                    || rl.is_key_down(KeyboardKey::KEY_D))
                    && step_timer <= 0.0
                {
                    step_sound.play();
                    step_timer = 0.25; // 4 pasos por segundo aprox.
                }

                // Actualizar jugador (lee input adentro)
                player.update(&rl, &maze);

                // Limpiar framebuffer
                framebuffer.clear();

                // Raycasting
                for i in 0..NUM_RAYS {
                    let ray_fraction = i as f32 / NUM_RAYS as f32;
                    let angle = player.a - (player.fov / 2.0) + (player.fov * ray_fraction);

                    let mut distance = 0.0;
                    let mut hit = false;
                    let mut hit_char = '#';
                    let mut hit_x = 0.0;
                    let mut hit_y = 0.0;

                    while distance < MAX_DEPTH {
                        let x = player.pos.x + distance * angle.cos();
                        let y = player.pos.y + distance * angle.sin();

                        if let Some(ch) = maze.get_wall(x, y) {
                            hit = true;
                            hit_char = ch;
                            hit_x = x;
                            hit_y = y;
                            break;
                        }

                        distance += STEP;
                    }

                    if hit {
                        let corrected_distance = distance * (player.a - angle).cos();
                        let wall_height = (SCREEN_HEIGHT as f32 * 1.5) / corrected_distance;

                        let start =
                            ((SCREEN_HEIGHT as f32) / 2.0 - wall_height / 2.0).max(0.0);
                        let end = ((SCREEN_HEIGHT as f32) / 2.0 + wall_height / 2.0)
                            .min(SCREEN_HEIGHT as f32);

                        let hit_frac =
                            if (hit_x.fract() - 0.5).abs() > (hit_y.fract() - 0.5).abs() {
                                hit_x.fract()
                            } else {
                                hit_y.fract()
                            };

                        let image = textures.get_image(hit_char).unwrap();
                        let tex_width = image.width as u32;
                        let tex_height = image.height as u32;
                        let tx = (hit_frac * tex_width as f32) as u32;

                        for y in start as usize..end as usize {
                            let ty =
                                ((y as f32 - start) / (end - start) * tex_height as f32) as u32;
                            let color = textures.get_pixel_color(hit_char, tx, ty);
                            framebuffer.set(i, y, color);
                        }
                    }
                }

                // Meta simple
                if player.pos.x > 18.0 && player.pos.y > 7.0 {
                    game_state = GameState::Success;
                }

                // --- DRAW ---
                {
                    let mut d = rl.begin_drawing(&thread);
                    framebuffer.draw(&mut d);

                    // Minimap
                    let minimap_x = SCREEN_WIDTH - MINIMAP_SIZE - 10;
                    let minimap_y = 10;
                    d.draw_rectangle(
                        minimap_x,
                        minimap_y,
                        MINIMAP_SIZE,
                        MINIMAP_SIZE,
                        Color::new(0, 0, 0, 150),
                    );

                    for y in 0..20 {
                        for x in 0..20 {
                            if let Some(ch) = maze.get_wall(x as f32, y as f32) {
                                let color = match ch {
                                    '#' => Color::GRAY,
                                    '+' => Color::RED,
                                    '-' => Color::GREEN,
                                    '|' => Color::BLUE,
                                    _ => Color::WHITE,
                                };
                                d.draw_rectangle(
                                    minimap_x + (x as f32 * MINIMAP_SCALE) as i32,
                                    minimap_y + (y as f32 * MINIMAP_SCALE) as i32,
                                    MINIMAP_SCALE as i32,
                                    MINIMAP_SCALE as i32,
                                    color,
                                );
                            }
                        }
                    }

                    // Jugador en el minimapa
                    d.draw_circle(
                        minimap_x + (player.pos.x * MINIMAP_SCALE) as i32,
                        minimap_y + (player.pos.y * MINIMAP_SCALE) as i32,
                        3.0,
                        Color::YELLOW,
                    );

                    // Dirección del jugador
                    d.draw_line(
                        minimap_x + (player.pos.x * MINIMAP_SCALE) as i32,
                        minimap_y + (player.pos.y * MINIMAP_SCALE) as i32,
                        minimap_x
                            + ((player.pos.x + player.a.cos() * 2.0) * MINIMAP_SCALE) as i32,
                        minimap_y
                            + ((player.pos.y + player.a.sin() * 2.0) * MINIMAP_SCALE) as i32,
                        Color::YELLOW,
                    );

                    // Estrella animada (meta) en minimapa
                    let star_texture = star_animation.current_texture();
                    d.draw_texture_ex(
                        star_texture,
                        Vector2 {
                            x: minimap_x as f32 + 18.0 * MINIMAP_SCALE,
                            y: minimap_y as f32 + 8.0 * MINIMAP_SCALE,
                        },
                        0.0,
                        0.2,
                        Color::WHITE,
                    );

                    // FPS
                    d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::LIME);
                }
            }

            GameState::Success => {
                // --- DRAW ---
                {
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    d.draw_texture(&success_texture, 0, 0, Color::WHITE);
                    d.draw_text("¡NIVEL COMPLETADO!", SCREEN_WIDTH / 2 - 150, 100, 30, Color::GREEN);
                    d.draw_text(
                        "Presiona ENTER para volver al menú",
                        SCREEN_WIDTH / 2 - 180,
                        400,
                        20,
                        Color::WHITE,
                    );
                    d.draw_text(
                        "Presiona ESC para salir",
                        SCREEN_WIDTH / 2 - 120,
                        430,
                        20,
                        Color::WHITE,
                    );
                }

                // INPUT después de cerrar el dibujo
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    game_state = GameState::Welcome;
                }
            }
        }

        // Limitar FPS a ~30
        let target_frame_time = 1.0 / 30.0;
        let elapsed = rl.get_time() - current_time;
        if elapsed < target_frame_time {
            thread::sleep(Duration::from_secs_f64(target_frame_time - elapsed));
        }
    }
}
