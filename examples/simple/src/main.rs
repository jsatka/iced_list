use iced::widget::{column, Container, Text};
use iced::{Length, Padding, Task};
use iced_reorderable::Column;

pub fn main() -> iced::Result {
    iced::application("Reorderable column", Simple::update, Simple::view)
        .window_size((320.0, 300.0))
        .run()
}

struct Simple {
    items: Vec<String>,
    dragged: Option<usize>,
}

impl Default for Simple {
    fn default() -> Self {
        Self {
            items: vec![
                "Tomato".to_string(),
                "Lettuce".to_string(),
                "Broccoli".to_string(),
                "Carrot".to_string(),
                "Cucumber".to_string(),
            ],
            dragged: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Drag(usize, usize),
    Drop(usize, usize),
    Cancel(usize),
}

impl Simple {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Drag(key, _loc) => {
                self.dragged = Some(key);
            }
            Message::Drop(key, loc) => {
                self.drop_item(key, loc);
                self.dragged = None;
            }
            Message::Cancel(_key) => {
                self.dragged = None;
            }
        }

        Task::none()
    }

    fn view(&self) -> Container<Message> {
        const ITEM_PADDING: Padding = Padding {
            top: 5.0,
            right: 8.0,
            bottom: 5.0,
            left: 8.0,
        };

        let reorderable_items =
            Column::from_iter(self.items.iter().enumerate().map(|(index, item)| {
                let item_style = if Some(index) == self.dragged {
                    style::item_dragged
                } else {
                    style::item_idle
                };
                let content = Container::new(Text::new(item))
                    .padding(ITEM_PADDING)
                    .style(item_style);

                (index, content.into())
            }))
            .spacing(10)
            .padding(Padding::default())
            .on_drag(|key, index| Message::Drag(key, index))
            .on_drop(|key, index| Message::Drop(key, index))
            .on_cancel(|key| Message::Cancel(key))
            .drop_position_marker(true);

        let title = Text::new("Drag and drop to rank the vegetables");
        let content = column!(title, reorderable_items).spacing(24).padding(12);

        Container::new(content).center(Length::Fill)
    }

    fn drop_item(&mut self, key: usize, mut loc: usize) {
        if key > self.items.len() || key == loc || key + 1 == loc {
            return;
        }
        if loc > key {
            loc -= 1;
        }
        let slot = self.items.remove(key);
        if loc < self.items.len() {
            self.items.insert(loc, slot);
        } else {
            self.items.push(slot);
        }
    }
}

mod style {
    use iced::widget::container;
    use iced::Theme;

    pub fn item_idle(theme: &Theme) -> container::Style {
        container::Style {
            border: iced::Border {
                color: theme.extended_palette().secondary.weak.color.into(),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn item_dragged(theme: &Theme) -> container::Style {
        container::Style {
            border: iced::Border {
                color: theme.extended_palette().primary.strong.color.into(),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        }
    }
}
