pub use crate::utility::*;

pub use crate::slide::*;
pub use crate::application_states::*;

const DEFAULT_LAST_WRITE_TIMER_INTERVAL : f32 = 0.20;

const DEFAULT_SLIDE_WHEN_NONE_GIVEN : &'static str = "test.slide";

pub struct ApplicationState {
    pub state: ApplicationScreen,

    pub current_working_directory: std::path::PathBuf,
    pub currently_selected_resolution: usize,
    pub currently_selected_directory: usize,
    pub last_write_timer: f32,
    pub slideshow: Option<Slide>,
}

impl ApplicationState {
    pub fn new(command_line_arguments: &Vec<String>) -> ApplicationState {
        ApplicationState {
            state: ApplicationScreen::ShowingSlide(ShowingSlideState),
            current_working_directory: std::path::PathBuf::from("./").canonicalize().unwrap(),
            currently_selected_resolution: 0,
            currently_selected_directory: 0,
            last_write_timer: 0.0,

            slideshow:
            Slide::new_from_file(
                match command_line_arguments.len() {
                    1 => {
                        DEFAULT_SLIDE_WHEN_NONE_GIVEN
                    }
                    2 => {
                        &command_line_arguments[1]
                    },
                    _ => {
                        println!("The only command line argument should be the slide file!");
                        DEFAULT_SLIDE_WHEN_NONE_GIVEN
                    }
                }
            )
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.try_and_hot_reload(delta_time);

        match self.state.clone() {
            ApplicationScreen::Quit(state) => {
                state.update(self, delta_time);
            },
            ApplicationScreen::InvalidOrNoSlide(state) => {
                state.update(self, delta_time);
            },
            ApplicationScreen::Options(state) => {
                state.update(self, delta_time);
            },
            ApplicationScreen::SelectSlideToLoad(state) => {
                state.update(self, delta_time);
            },
            ApplicationScreen::ShowingSlide(state) => {
                state.update(self, delta_time);
            },
            ApplicationScreen::ChangePage(state) => {
                state.update(self, delta_time);
            },
        }
    }

    pub fn draw(&self, graphics_context: &mut SDL2GraphicsContext) {
        match self.state.clone() {
            ApplicationScreen::Quit(state) => {
                state.draw(self, graphics_context);
            },
            ApplicationScreen::SelectSlideToLoad(state) => {
                state.draw(self, graphics_context);
            },
            ApplicationScreen::InvalidOrNoSlide(state) => {
                state.draw(self, graphics_context);
            },
            ApplicationScreen::Options(state) => {
                state.draw(self, graphics_context);
            },
            ApplicationScreen::ChangePage(state) => {
                state.draw(self, graphics_context);
            },
            ApplicationScreen::ShowingSlide(state) => {
                state.draw(self, graphics_context);
            },
        }
    }

    pub fn handle_event(&mut self, graphics_context: &mut SDL2GraphicsContext, event_pump: &mut sdl2::EventPump, delta_time: f32) {
        match self.state.clone() {
            ApplicationScreen::Quit(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
            ApplicationScreen::SelectSlideToLoad(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
            ApplicationScreen::InvalidOrNoSlide(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
            ApplicationScreen::Options(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
            ApplicationScreen::ChangePage(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            }
            ApplicationScreen::ShowingSlide(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
        }
    }

    fn try_and_hot_reload(&mut self, delta_time: f32) {
        if let Some(slideshow) = &mut self.slideshow {
            if self.last_write_timer <= 0.0 {
                self.last_write_timer = DEFAULT_LAST_WRITE_TIMER_INTERVAL;
                slideshow.reload().expect("should be successful.");
            }
        }

        self.last_write_timer -= delta_time;
    }
}
