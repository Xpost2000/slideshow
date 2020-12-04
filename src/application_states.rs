use sdl2::event::Event as SDLEvent;
use sdl2::keyboard::Keycode as SDLKeycode;

pub use crate::utility::*;
pub use crate::slide::*;
pub use crate::color::*;
pub use crate::graphics_context::*;
pub use crate::application::*;

pub trait ApplicationScreenState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
    }

    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
    }
}

// Unit structs for trait implementation.
#[derive(Clone)]
pub struct InvalidOrNoSlideState;
#[derive(Clone)]
pub struct OptionsState;
#[derive(Clone)]
pub struct ShowingSlideState;
#[derive(Clone)]
pub struct SelectSlideToLoadState;
#[derive(Clone)]
pub struct ChangePageState {pub from: isize, pub to: isize,}
#[derive(Clone)]
pub struct QuitState;
#[derive(Clone)]
pub enum ApplicationScreen {
    InvalidOrNoSlide(InvalidOrNoSlideState),
    Options(OptionsState),
    ShowingSlide(ShowingSlideState),
    SelectSlideToLoad(SelectSlideToLoadState),
    ChangePage(ChangePageState),
    Quit(QuitState),
}

impl ApplicationScreenState for QuitState {}

impl ApplicationScreenState for InvalidOrNoSlideState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                    app.state = ApplicationScreen::Options(OptionsState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                    app.state = ApplicationScreen::SelectSlideToLoad(SelectSlideToLoadState);
                },
                _ => {}
            }
        }
    }
    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
    }
    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
    }
}

impl ApplicationScreenState for OptionsState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        let resolutions = graphics_context.get_avaliable_resolutions();
        let resolution_count = resolutions.iter().count();
        app.currently_selected_resolution = clamp(app.currently_selected_resolution,
                                                   0, resolution_count - 1);
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::F), ..} => {
                    graphics_context.toggle_fullscreen();
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Return), .. } =>  {
                    let resolution_list = graphics_context.get_avaliable_resolutions();
                    if let Some(resolution_pair) = resolution_list.get(app.currently_selected_resolution) {
                        graphics_context.set_resolution((resolution_pair.0 as u32, resolution_pair.1 as u32));
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                    if app.currently_selected_resolution > 0 {
                        app.currently_selected_resolution -= 1;
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                    app.currently_selected_resolution += 1;
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                    app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                    app.state = ApplicationScreen::SelectSlideToLoad(SelectSlideToLoadState);
                },
                _ => {}
            }
        }
    }
    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
    }
    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
    } 
}

impl ApplicationScreenState for ChangePageState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown {..} => {
                    app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                    if let Some(slideshow) = &mut app.slideshow {
                        slideshow.finish_transition();
                    }
                },
                _ => {}
            }
        }
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
    }

    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
    }
}

impl ApplicationScreenState for ShowingSlideState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {
                    graphics_context.clear_resources();
                    app.slideshow = None;
                    app.state = ApplicationScreen::InvalidOrNoSlide(InvalidOrNoSlideState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::F), ..} => {
                    graphics_context.toggle_fullscreen();
                },
                #[cfg(debug_assertions)]
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Num0), .. } => {
                    graphics_context.camera.scale = 1.0;
                },
                #[cfg(debug_assertions)]
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                    graphics_context.camera.scale += 1.0 * delta_time;
                },
                #[cfg(debug_assertions)]
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                    graphics_context.camera.scale -= 1.0 * delta_time;
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::R), .. } => {
                    if let Some(slideshow) = &mut app.slideshow {
                        graphics_context.clear_resources();
                        slideshow.reload().expect("should be successful");
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. } => {
                    if let Some(slideshow) = &mut app.slideshow {
                        app.state = ApplicationScreen::ChangePage(
                            ChangePageState{
                                from: slideshow.current_page(),
                                to: slideshow.next_page()
                            });
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                    if let Some(slideshow) = &mut app.slideshow {
                        app.state = ApplicationScreen::ChangePage(
                            ChangePageState{
                                from: slideshow.current_page(),
                                to: slideshow.previous_page()
                            });
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                    app.state = ApplicationScreen::SelectSlideToLoad(SelectSlideToLoadState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                    app.state = ApplicationScreen::Options(OptionsState);
                },
                _ => {}
            }
        }
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
    }

    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
    }
}

impl ApplicationScreenState for SelectSlideToLoadState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        let directory_listing = std::fs::read_dir(&app.current_working_directory).expect("Failed to get directory listing?");
        app.currently_selected_directory = clamp(app.currently_selected_directory,
                                                  0, directory_listing.into_iter().count()-1);
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Return), .. } |
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. }  =>  {
                    let directory_listing = std::fs::read_dir(&app.current_working_directory).expect("Failed to get directory listing?");
                    let selected_path = directory_listing.into_iter().nth(app.currently_selected_directory);

                    if let Some(path) = selected_path {
                        let path = path.expect("bad permission?");
                        let file_type = path.file_type().unwrap();

                        if file_type.is_dir() {
                            app.current_working_directory = path.path();
                            app.currently_selected_directory = 0;
                        } else {
                            graphics_context.clear_resources();

                            let new_slide = Slide::new_from_file(path.path().to_str().expect("bad unicode"));
                            app.slideshow = new_slide;

                            app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                        }
                    }
                },

                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                    app.current_working_directory.pop();
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                    if app.currently_selected_directory > 0 {
                        app.currently_selected_directory -= 1;
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                    app.currently_selected_directory += 1;
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                    app.state = ApplicationScreen::Options(OptionsState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                    app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                },
                _ => {}
            }
        }
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
    }

    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
    }
}
