mod co;

use iced::{
    widget::{self, container},
    Color, Element,
};
use slu_ipc::LauncherIpc;

#[derive(Debug, Default)]
struct State {}

#[derive(Debug, Clone, Copy)]
enum Message {
    Close,
}

fn handler(_state: &mut State, message: Message) {
    match message {
        Message::Close => {
            std::process::exit(0);
        }
    }
}

fn view(_state: &State) -> Element<'_, Message> {
    let close_btn = widget::container(
        widget::button("X")
            .on_press(Message::Close)
            .height(30)
            .width(30)
            .style(|_theme, _status| widget::button::Style {
                background: None,
                text_color: Color::from_rgb8(150, 150, 150),
                ..Default::default()
            }),
    )
    .padding(20)
    .align_right(0)
    .width(iced::Fill);

    let exe_path = std::env::current_exe().unwrap();
    let logo_path = exe_path
        .parent()
        .unwrap()
        .join("static/icons/metal_corp.svg");

    let centered_content = widget::container(
        widget::column([
            widget::svg(logo_path).height(200).width(200).into(),
            widget::text("Preparing the user interface...")
                .color(Color::from_rgb8(200, 200, 200))
                .size(18)
                .font(iced::Font {
                    family: iced::font::Family::Name("Segoe UI"),
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .into(),
        ])
        .spacing(60)
        .align_x(iced::Alignment::Center),
    )
    .width(iced::Fill)
    .height(iced::Fill)
    .align_x(iced::Center)
    .align_y(iced::Center);

    let content = widget::stack([close_btn.into(), centered_content.into()])
        .width(iced::Fill)
        .height(iced::Fill);

    container(content)
        .height(iced::Fill)
        .width(iced::Fill)
        .style(|_theme| container::Style::default().background(Color::from_rgb8(15, 15, 15)))
        .into()
}

fn main() -> iced::Result {
    LauncherIpc::start(|_msg| {
        std::process::exit(0);
    })
    .expect("Failed to start launcher ipc");

    let desktop = co::get_desktop_rect();
    let desk_width = desktop.right - desktop.left;
    let desk_height = desktop.bottom - desktop.top;

    iced::application::application("Seelen UI - Splash Screen", handler, view)
        .window(iced::window::Settings {
            size: iced::Size::new(desk_width as f32, desk_height as f32),
            position: iced::window::Position::Specific(iced::Point::new(
                desktop.left as f32,
                desktop.top as f32,
            )),
            decorations: false,
            resizable: false,
            ..Default::default()
        })
        .antialiasing(true)
        .run()
}
