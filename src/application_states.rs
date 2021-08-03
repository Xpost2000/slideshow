pub use sdl2::event::Event as SDLEvent;
pub use sdl2::keyboard::Keycode as SDLKeycode;

pub use crate::utility::*;
pub use crate::slide::*;
pub use crate::color::*;
pub use crate::graphics_context::*;
pub use crate::application::*;

pub trait ApplicationScreenState {
    fn handle_event(&self,
                    _app: &mut ApplicationState,
                    _graphics_context: &mut SDL2GraphicsContext,
                    _event_pump: &mut sdl2::EventPump,
                    _delta_time: f32) {
    }

    fn draw(&self,
            _app: &ApplicationState,
            _graphics_context: &mut SDL2GraphicsContext) {
    }

    fn update(&self,
              _app: &mut ApplicationState,
              _delta_time: f32) {
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

pub use crate::invalid_or_no_slide_state::*;
pub use crate::options_state::*;
pub use crate::change_page_state::*;
pub use crate::showing_slide_state::*;
pub use crate::select_slide_to_load_state::*;

