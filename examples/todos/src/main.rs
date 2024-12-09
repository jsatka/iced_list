use iced::widget::{
    column, container, row, Button, Checkbox, Container, Rule, Space, Text, TextInput, Toggler,
};
use iced::{Center, Element, Length::*, Padding, Task};
use iced_reorderable::Column;

pub fn main() -> iced::Result {
    iced::application("Todos", Todos::update, Todos::view)
        .window_size((360.0, 640.0))
        .run()
}

struct Todos {
    items: Vec<(String, bool)>,
    input: String,
    dragged: Option<usize>,
    options: Options,
}

struct Options {
    drop_position_marker: bool,
    drag_follow: bool,
    drag_lateral: bool,
    drag_center: bool,
}

impl Default for Todos {
    fn default() -> Self {
        Self {
            items: vec![
                ("Eat".to_string(), false),
                ("Work".to_string(), false),
                ("Sleep".to_string(), false),
            ],
            input: "".to_string(),
            dragged: None,
            options: Options::default(),
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            drop_position_marker: true,
            drag_follow: true,
            drag_lateral: true,
            drag_center: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Grab(usize),
    Drag(usize, usize),
    Drop(usize, usize),
    Cancel(usize),
    Remove(usize),
    Add,
    ToggleItemChecked(usize, bool),
    TypeInput(String),
    SetDropPositionMarker(bool),
    SetDragFollow(bool),
    SetDragLateral(bool),
    SetDragCenter(bool),
}

impl Todos {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Grab(key) => {
                self.dragged = Some(key);
            }
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
            Message::Remove(key) => {
                self.items.remove(key);
            }
            Message::Add => {
                if !self.input.is_empty() {
                    self.items.push((self.input.clone(), false));
                    self.input.clear();
                }
            }
            Message::ToggleItemChecked(key, checked) => {
                if let Some(item) = self.items.get_mut(key) {
                    item.1 = checked;
                }
            }
            Message::TypeInput(s) => {
                self.input = s;
            }
            Message::SetDropPositionMarker(value) => {
                self.options.drop_position_marker = value;
            }
            Message::SetDragFollow(value) => {
                self.options.drag_follow = value;
            }
            Message::SetDragLateral(value) => {
                self.options.drag_lateral = value;
            }
            Message::SetDragCenter(value) => {
                self.options.drag_center = value;
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let options = column![
            row![Toggler::new(self.options.drop_position_marker)
                .label("Show drop position marker")
                .on_toggle(|v| Message::SetDropPositionMarker(v))],
            row![Toggler::new(self.options.drag_follow)
                .label("Dragged item follows cursor")
                .on_toggle(|v| Message::SetDragFollow(v))],
            row![
                Space::with_width(Fixed(20.0)),
                Toggler::new(self.options.drag_lateral)
                    .label("Dragged item can move laterally")
                    .on_toggle_maybe(if self.options.drag_follow {
                        Some(|v| Message::SetDragLateral(v))
                    } else {
                        None
                    })
            ],
            row![
                Space::with_width(Fixed(20.0)),
                Toggler::new(self.options.drag_center)
                    .label("Dragged item centered on cursor")
                    .on_toggle_maybe(if self.options.drag_follow {
                        Some(|v| Message::SetDragCenter(v))
                    } else {
                        None
                    })
            ],
        ]
        .spacing(10)
        .padding(10);

        let add_item_row = row![
            TextInput::new("Add item ...", &self.input)
                .on_input(Message::TypeInput)
                .on_submit(Message::Add)
                .padding(10)
                .width(Fill),
            Button::new(Text::new("Add"))
                .on_press(Message::Add)
                .padding(10),
        ]
        .padding(Padding::new(10.0).top(20.0))
        .spacing(10);

        let reorderable_items =
            Column::from_iter(self.items.iter().enumerate().map(|(index, item)| {
                let remove_button = Button::new(Text::new("⌫").size(24))
                    .on_press(Message::Remove(index))
                    .padding(Padding {
                        top: 2.0,
                        right: 4.0,
                        bottom: 2.0,
                        left: 4.0,
                    })
                    .style(style::remove_button);
                let row = row![
                    Checkbox::new(&item.0, item.1)
                        .on_toggle(move |checked| Message::ToggleItemChecked(index, checked)),
                    Space::with_width(Fill),
                    remove_button
                ]
                .spacing(10)
                .padding(Padding {
                    top: 2.0,
                    right: 8.0,
                    bottom: 2.0,
                    left: 8.0,
                })
                .align_y(Center);

                let item_style = if Some(index) == self.dragged {
                    style::item_dragged
                } else {
                    style::item_idle
                };
                let content = Container::new(row).style(item_style);

                (index, content.into())
            }))
            .spacing(12)
            .padding(10)
            .on_grab(Message::Grab)
            .on_drag(Message::Drag)
            .on_drop(Message::Drop)
            .on_cancel(Message::Cancel)
            .drop_position_marker(self.options.drop_position_marker)
            .drag_follow(self.options.drag_follow)
            .drag_lateral(self.options.drag_lateral)
            .drag_center(self.options.drag_center);

        container(column![
            container(options.height(Shrink).width(Fill)).style(style::options_container),
            Rule::horizontal(1),
            add_item_row.height(Shrink),
            reorderable_items.height(Fill)
        ])
        .center(Fill)
        .into()
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
    use iced::{
        widget::{button, container},
        Theme,
    };

    pub fn options_container(theme: &Theme) -> container::Style {
        container::Style {
            background: Some(theme.extended_palette().secondary.weak.color.into()),
            ..Default::default()
        }
    }

    pub fn item_idle(theme: &Theme) -> container::Style {
        container::Style {
            text_color: None,
            background: Some(theme.palette().background.into()),
            border: iced::Border {
                color: theme.extended_palette().secondary.weak.color.into(),
                width: 1.0,
                radius: 5.0.into(),
            },
            shadow: Default::default(),
        }
    }

    pub fn item_dragged(theme: &Theme) -> container::Style {
        container::Style {
            text_color: None,
            background: Some(theme.palette().background.into()),
            border: iced::Border {
                color: theme.extended_palette().primary.strong.color.into(),
                width: 1.0,
                radius: 5.0.into(),
            },
            shadow: Default::default(),
        }
    }

    pub fn remove_button(theme: &Theme, status: button::Status) -> button::Style {
        use button::{Status, Style};

        let palette = theme.extended_palette();
        let base = Style {
            text_color: palette.danger.base.color,
            ..Default::default()
        };
        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: palette.danger.strong.color,
                ..base
            },
            Status::Disabled => Style {
                text_color: palette.secondary.weak.color,
                ..base
            },
        }
    }
}
