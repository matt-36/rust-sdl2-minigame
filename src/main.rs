extern crate sdl2;
extern crate serde;
extern crate serde_json;

mod animation;
mod collision;
mod controller;
mod entity;
mod events;
mod game;
mod room;

use game::Game;
use sdl2::{event::Event, video::FullscreenType};
// use sdl2::gfx::framerate::FPSManager;
use sdl2::image::InitFlag;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use std::time::{Duration, SystemTime};

use crate::controller::{Controller, Moveset};

const GAME_SIZE_X: u32 = 768;
const GAME_SIZE_Y: u32 = 432;

fn move_anim(ticks: u32, fpm: u32) -> i32 {
    let x = 32 * ((ticks / 100) % fpm) as i32;
    println!("{:?}", x);
    x
}

fn main() -> Result<(), String> {
    let MOVEMENT_SPEED = 4;
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let _ttf_context = sdl2::ttf::init().expect("Failed to initialize ttf");
    // the window is the representation of a window in your operating system,
    // however you can only manipulate properties of that window, like its size, whether it's
    // fullscreen, ... but you cannot change its content without using a Canvas or using the
    // `surface()` method.
    let window = video_subsystem
        .window("rust game", GAME_SIZE_X, GAME_SIZE_Y)
        .position_centered()
        .allow_highdpi()
        .build()
        .map_err(|e| e.to_string())?;

    // the canvas allows us to both manipulate the property of the window and to change its content
    // via hardware or software rendering. See CanvasBuilder for more info.
    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    canvas.set_integer_scale(true)?;
    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    canvas.set_draw_color(Color::RGB(0, 0, 0));

    let texture_creator: TextureCreator<_> = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);

    let player = entity::Player::new(Rect::new(0, 40, 16, 24), 1);
    game.add_player(player, "assets/characters.png");
    let mut player2 = entity::Player::new(Rect::new(8, 8, 16, 24), 2);
    player2.set_moveset(Moveset {
        up: Keycode::Up,
        left: Keycode::Left,
        down: Keycode::Down,
        right: Keycode::Right,
    });
    game.add_player(player2, "assets/characters.png");

    let mut camera = canvas.viewport(); // to start with so that width is right and stuff
                                        //
                                        // i dont think viewport works like that ill show u
                                        // rn the viewport is moved with the character
                                        // it doesnt actually do anything other than move the render borders
                                        //
                                        // they are just rects lol
                                        // let wall_tex = texture_creator.load_texture("assets/yellow.png")?;
    let wall = collision::Collider::new(
        Rect::new(0, 0, 1200, 1200),
        Rect::new(200, 50, 120 * 4, 120 * 4),
        3,
    );
    /*
     *
     * ye look at aabb i reduced the thing cos the hitboxes wouldnt rescale
     * look at it
     *
     * texture only has to live as long as player
     */
    game.add_collider(wall, Some("assets/obama.jpg"));

    // let _swing_tex = texture_creator.load_texture("assets/swoosh.png")?;
    let timer = sdl_context.timer()?;

    let mut fullscreen = false;
    let mut event_pump = sdl_context.event_pump()?;
    let mut frame_times = [0u128; 60];
    let mut frames = 0;
    let mut last_frame_time = SystemTime::now();
    let mut fps = 0f64;
    'running: loop {
        // get the inputs here
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    ..
                } => {
                    // canvas.window_mut().set_size(1920, 1080).expect("fuck");
                    let window = canvas.window_mut();
                    window
                        .set_fullscreen(if !fullscreen {
                            FullscreenType::Desktop
                        } else {
                            FullscreenType::Off
                        })
                        .expect("failed changing to or from fullscreen");
                    let size = window.size();
                    canvas
                        .set_scale(
                            (size.0 as f32) / (GAME_SIZE_X as f32),
                            (size.1 as f32) / (GAME_SIZE_Y as f32),
                        )
                        .expect("failed resizing the canvas");
                    fullscreen ^= true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F7),
                    ..
                } => game.togglehitboxes(),
                Event::KeyDown { .. } | Event::KeyUp { .. } => game.handle(&event),
                _ => {} // direction works but not movement speed
            } // nop no move
        }

        game.tick(timer.ticks());
        // this is where it belongs because it should happen every frame
        // and to make it independent of time we should have it be
        // multiplied by the duration since last update
        // though we can have a custom function that takes game
        // or could have it be a function on game that is like... a tick advance
        // yeah
        // before i would check collision before rendering
        //  ok lets do that
        canvas.clear();
        game.update(&mut canvas)
            .expect("game failed to update/render"); // it happens here cos the function doesnt render players yet
                                                     //still nothing :(
        canvas.present(); // its cos we never define the x and y of the player
                          // println!("")
                          // fps stuff
                          // TODO create an fps manager to reduce mess like this
        let frame_time = SystemTime::now();
        frame_times[frames % 60] = frame_time
            .duration_since(last_frame_time)
            .unwrap()
            .as_nanos();
        frames += 1;
        last_frame_time = frame_time;
        fps = 60f64 / frame_times.iter().sum::<u128>() as f64 * 1_000_000_000f64; // ye it started working
        std::thread::sleep(Duration::from_millis(0));
    };
    println!("{}", fps);
    Ok(())
}
