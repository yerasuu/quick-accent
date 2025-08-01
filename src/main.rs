use iced::alignment::{Horizontal, Vertical};
use iced::{Element, Point, Size, Task, window};

mod config;
mod screen;

pub fn main() -> iced::Result {
    // Load configuration (creates default if not exists)
    let config = match config::app::AppConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {}, using defaults", e);
            config::app::AppConfig::default()
        }
    };

    // Detect actual screen dimensions
    let screen_info = screen::ScreenInfo::detect();

    if config.screen.debug_screen_detection {
        println!(
            "Detected screen size: {}x{}",
            screen_info.width, screen_info.height
        );
    }

    // Calculate window size and position using config
    let (window_width, window_height) =
        config.calculate_window_size(screen_info.width, screen_info.height);
    let (x_position, y_position) =
        config.calculate_window_position(screen_info.width, screen_info.height, window_width);

    if config.screen.debug_screen_detection {
        println!("Calculated window size: {}x{}", window_width, window_height);
        println!(
            "Calculated window position: ({}, {})",
            x_position, y_position
        );
    }

    // Clone config for use in closure
    let config_for_app = config.tool.clone();

    iced::application(
        move || App::new(config_for_app.clone()),
        App::update,
        App::view,
    )
    .window(window::Settings {
        size: Size::new(window_width, window_height),
        position: window::Position::Specific(Point::new(x_position, y_position)),
        resizable: false,
        ..Default::default()
    })
    .run()
}

struct App {
    config: config::tool::ToolConfig,
}

#[derive(Debug, Clone)]
enum Message {}

impl App {
    fn new(config: config::tool::ToolConfig) -> Self {
        App { config }
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Display the configured text
        println!("ToolConfig: {:?}", self.config);
        iced::widget::container(iced::widget::text("some text"))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

impl Default for App {
    fn default() -> Self {
        let config = match config::app::AppConfig::load() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to load config: {}, using defaults", e);
                config::app::AppConfig::default()
            }
        };
        Self::new(config.tool)
    }
}
