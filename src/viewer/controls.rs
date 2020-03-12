use iced_wgpu::Renderer;
use iced_winit::{slider, Align, Column, Element, Length, Row, Slider, Text};
use wgpu::Color;

pub struct Controls {
    sliders: [slider::State; 3],
}

#[derive(Debug)]
pub enum Message {
    BackgroundColorChanged(Color),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            sliders: Default::default(),
        }
    }

    pub fn update(&self, message: Message, state: &mut super::State) {
        match message {
            Message::BackgroundColorChanged(color) => state.background_color = color,
        }
    }

    pub fn view(&mut self, state: &super::State) -> Element<Message, Renderer> {
        let [r, g, b] = &mut self.sliders;
        let background_color = state.background_color;

        let sliders = Row::new()
            .width(Length::Units(500))
            .spacing(20)
            .push(Slider::new(
                r,
                0.0..=1.0,
                state.background_color.r as f32,
                move |r| {
                    Message::BackgroundColorChanged(Color {
                        r: r as f64,
                        ..background_color
                    })
                },
            ))
            .push(Slider::new(
                g,
                0.0..=1.0,
                state.background_color.g as f32,
                move |g| {
                    Message::BackgroundColorChanged(Color {
                        g: g as f64,
                        ..background_color
                    })
                },
            ))
            .push(Slider::new(
                b,
                0.0..=1.0,
                state.background_color.b as f32,
                move |b| {
                    Message::BackgroundColorChanged(Color {
                        b: b as f64,
                        ..background_color
                    })
                },
            ));

        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::End)
            .push(
                Column::new()
                    .width(Length::Fill)
                    .align_items(Align::End)
                    .push(
                        Column::new()
                            .padding(10)
                            .spacing(10)
                            .push(Text::new("Background color").color(iced_winit::Color::WHITE))
                            .push(sliders)
                            .push(
                                Text::new(format!("{:?}", background_color))
                                    .size(14)
                                    .color(iced_winit::Color::WHITE),
                            ),
                    ),
            )
            .into()
    }
}
