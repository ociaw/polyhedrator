use iced_wgpu::Renderer;
use iced_winit::{button, text_input, Align, Button, Column, Element, Length, Radio, Row, Text, TextInput};
use crate::seeds::{Seed, Platonic};
use crate::{operators, Operator};
use super::generator::Generator;

pub struct Controls {
    seed: Seed,
    operations: Vec<Operator>,
    notation_input: text_input::State,
    update_button: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    SeedSelected(Seed),
    NotationChanged(String),
    UpdatePressed,
}

impl Controls {
    pub fn new() -> Controls {
        let kis = operators::Kis::scale_apex(0.0);
        let operations = vec![
            Operator::Kis(kis),
            Operator::Dual,
            Operator::Kis(kis),
            Operator::Dual,
            Operator::Kis(kis),
            Operator::Dual,
            Operator::Kis(kis),
            Operator::Dual,
        ];
        Controls {
            seed: Seed::Platonic(Platonic::Dodecahedron),
            operations,
            notation_input: text_input::State::focused(),
            update_button: Default::default(),
        }
    }

    pub fn update(&mut self, message: Message, state: &mut super::State, device: &wgpu::Device) {
        match message {
            Message::SeedSelected(seed) => self.seed = seed,
            Message::UpdatePressed => {
                let mut generator = Generator::seed(self.seed.polyhedron(2.0));
                generator.apply_iter(self.operations.iter().rev().cloned());
                let update = super::render::Update {
                    mesh: Some(generator.to_mesh()), .. Default::default()
                };
                state.apply_update(device, update);
            },
            Message::NotationChanged(notation) => {
                if let Ok(operations) = Operator::try_parse(&notation) {
                    self.operations = operations;
                }
            },
        }
    }

    pub fn view(&mut self) -> Element<Message, Renderer> {
        let mut seed_column = Column::new().width(Length::Units(170)).spacing(10)
            .push(Text::new("Seed"));
        for seed in Platonic::all().iter().cloned().map(|p| Seed::Platonic(p)) {
            let radio = Radio::new(seed, &seed.to_string(), Some(self.seed), Message::SeedSelected);
            seed_column = seed_column.push(radio);
        }
        seed_column = seed_column.push(Button::new(&mut self.update_button, Text::new("Update"))
            .on_press(Message::UpdatePressed));

        let notation_text = self.operations.iter().fold(String::with_capacity(self.operations.len()), |mut notation, op| -> String {
            let str: String = (*op).into();
            notation.push_str(&str);
            notation
        });

        let notation_element = TextInput::new(&mut self.notation_input, "e.g. dkdkdk", &notation_text, |text| Message::NotationChanged(text.to_owned()));

        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::End)
            .push(seed_column)
            .push(notation_element)
            .into()
    }
}
