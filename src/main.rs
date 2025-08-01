use iced::{Element, Task, Size, window, Point};
use iced::alignment::{Horizontal, Vertical};

mod screen;

pub fn main() -> iced::Result {
    // Detect actual screen dimensions
    let screen_info = screen::ScreenInfo::detect();

    println!("Detected screen size: {}x{}", screen_info.width, screen_info.height);
    let window_width = (screen_info.width * 0.75) as f32;
    let window_height = 100.0;

    println!("Calculated window size: {}x{}", window_width, window_height);
    
    // Position window in the upper-center area where focused windows typically appear
    // This mimics where OS usually opens new focused windows
    let x_position = (screen_info.width - window_width) / 2.0; // Center horizontally
    let y_position = screen_info.height / 4.0; // Upper quarter of screen

    println!("Calculated window position: ({}, {})", x_position, y_position);

    iced::application(App::default, App::update, App::view)
        .window(window::Settings {
            size: Size::new(window_width, window_height),
            position: window::Position::Specific(Point::new(x_position, y_position)),
            resizable: false,
            ..Default::default()
        })
        .run()
}

struct App;

#[derive(Debug, Clone)]
enum Message {}

impl App {
    fn new() -> Self {
        App
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Empty view - just background
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
        Self::new()
    }
}
