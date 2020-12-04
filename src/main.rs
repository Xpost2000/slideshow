#![windows_subsystem="windows"]
/*
    beginnings of a slideshow program?

TODO: Please rewrite the tokenizer.
*/
#[allow(dead_code)]
mod markup;
#[allow(dead_code)]
mod utility;
pub use self::utility::*;
#[allow(dead_code)]
mod graphics_context;
use self::graphics_context::*;
mod color;
use self::color::*;
mod slide_parser;
mod slide;
mod application;
use self::application::*;

mod application_states;
mod invalid_or_no_slide_state;
mod options_state;
mod change_page_state;
mod showing_slide_state;
mod select_slide_to_load_state;

const DEFAULT_WINDOW_WIDTH : u32 = 1024;
const DEFAULT_WINDOW_HEIGHT : u32 = 768;
const DEFAULT_SLIDE_WHEN_NONE_GIVEN : &'static str = "test.slide";

fn main() {
    let sdl2_context = sdl2::init().expect("SDL2 failed to initialize?");
    let video_subsystem = sdl2_context.video().unwrap();

    let sdl2_ttf_context = sdl2::ttf::init()
        .expect("SDL2 ttf failed to initialize?");
    let sdl2_image_context = sdl2::image::init(sdl2::image::InitFlag::PNG | sdl2::image::InitFlag::JPG)
        .expect("SDL2 image failed to initialize?");

    let window = video_subsystem.window("stupid slideshow",
                                        DEFAULT_WINDOW_WIDTH,
                                        DEFAULT_WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .expect("Window failed to open?");

    let mut graphics_context = SDL2GraphicsContext::new(window,
                                                        &sdl2_ttf_context,
                                                        &sdl2_image_context,
                                                        &video_subsystem);
    graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");

    let mut event_pump = sdl2_context.event_pump().unwrap();

    use std::env;
    let arguments : Vec<String> = env::args().collect();
    let mut application_state = ApplicationState::new(&arguments);

    let mut sdl2_timer = sdl2_context.timer().unwrap();
    let mut delta_time = 0;

    'running: loop {
        let start_time = sdl2_timer.ticks();

        graphics_context.clear_color(Color::new(0, 0, 0, 255));
        graphics_context.enable_alpha_blending();
        graphics_context.use_viewport_default();

        if let ApplicationScreen::Quit(_) = application_state.state {
            break 'running;
        } else {
            let delta_time = delta_time as f32 / 1000.0;

            application_state.handle_event(&mut graphics_context, &mut event_pump, delta_time);
            application_state.update(delta_time);
            application_state.draw(&mut graphics_context);
        }

        graphics_context.present();

        let end_time = sdl2_timer.ticks();
        delta_time = end_time - start_time;

        #[cfg(debug_assertions)]
        {println!("{}", 1.0/(delta_time as f32/1000.0));}
    }
}
