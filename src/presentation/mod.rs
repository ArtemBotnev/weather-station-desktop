pub mod app;
pub mod mapper;
pub mod message;
pub mod state;

pub fn run() -> iced::Result {
    iced::application(app::APP_TITLE, app::update, app::view)
        .subscription(app::subscription)
        .theme(app::theme)
        .run_with(app::init)
}
