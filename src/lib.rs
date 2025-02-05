use iced::advanced::layout;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Shell;
use iced::alignment::{self, Alignment};
use iced::border::Radius;
use iced::mouse;
use iced::touch;
use iced::Border;
use iced::Color;
use iced::Event;
use iced::Point;
use iced::Theme;
use iced::{Element, Length, Padding, Pixels, Rectangle, Size, Vector};

/// A container that distributes its contents vertically and allows dragging
/// and dropping its keyed children.
///
/// # Example
/// ```no_run
/// use super::Column;
///
/// let mut data = vec![
///    "First item",
///    "Second item",
///    "Third item",
///    "Fourth item",
/// ];
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     Dropped(usize, usize),
/// }
///
/// fn move_on_drop(&mut data: Vec<>, msg: Message) {
///     if let Message::Dropped(src_index, dst_index) = msg {
///         if src_index > self.items.len() || src_index == dst_index || src_index + 1 == dst_index {
///             return;
///         }
///         if dst_index > src_index {
///             dst_index -= 1;
///         }
///         let slot = self.items.remove(src_index);
///         if dst_index < self.items.len() {
///             self.items.insert(dst_index, slot);
///         } else {
///             self.items.push(slot);
///         }
///     }
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     Column::with_children(data.iter().enumerate())
///     .on_drop(Message::Dropped)
///     .into()
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Column<'a, Key, Message, Theme, Renderer>
where
    Key: Copy + PartialEq,
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    spacing: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    align: Alignment,
    clip: bool,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    keys: Vec<Key>,
    class: Theme::Class<'a>,
    on_grab: Option<Box<dyn Fn(Key) -> Message + 'a>>,
    on_drag: Option<Box<dyn Fn(Key, usize) -> Message + 'a>>,
    on_drop: Option<Box<dyn Fn(Key, usize) -> Message + 'a>>,
    on_cancel: Option<Box<dyn Fn(Key) -> Message + 'a>>,
    drop_position_marker: bool,
    drag_follow: bool,
    drag_lateral: bool,
    drag_center: bool,
}

impl<'a, Key, Message, Theme, Renderer> Column<'a, Key, Message, Theme, Renderer>
where
    Key: Copy + PartialEq,
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    /// Creates an empty [`Column`].
    pub fn new() -> Self {
        Self::from_vecs(Vec::new(), Vec::new())
    }

    /// Creates a [`Column`] with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from_vecs(Vec::with_capacity(capacity), Vec::with_capacity(capacity))
    }

    /// Creates a [`Column`] with the given keys and elements.
    pub fn with_children(
        children: impl IntoIterator<Item = (Key, Element<'a, Message, Theme, Renderer>)>,
    ) -> Self {
        let iterator = children.into_iter();

        Self::with_capacity(iterator.size_hint().0).extend(iterator)
    }

    /// Creates a [`Column`] from an already allocated [`Vec`].
    ///
    /// Keep in mind that the [`Column`] will not inspect the [`Vec`], which means
    /// it won't automatically adapt to the sizing strategy of its contents.
    ///
    /// If any of the children have a [`Length::Fill`] strategy, you will need to
    /// call [`Column::width`] or [`Column::height`] accordingly.
    pub fn from_vecs(keys: Vec<Key>, children: Vec<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            align: Alignment::Start,
            clip: false,
            keys,
            children,
            class: Theme::default(),
            on_grab: None,
            on_drag: None,
            on_drop: None,
            on_cancel: None,
            drop_position_marker: true,
            drag_follow: false,
            drag_lateral: false,
            drag_center: false,
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`Column`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Column`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`Column`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Column`] .
    pub fn align_x(mut self, align: impl Into<alignment::Horizontal>) -> Self {
        self.align = Alignment::from(align.into());
        self
    }

    /// Sets whether the contents of the [`Column`] should be clipped on
    /// overflow.
    ///
    /// Note that a dragged child element will not be clipped and can be drawn
    /// outside the bounds of the [`Column`], if dragged outside the column.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Adds an element to the [`Column`].
    pub fn push(
        mut self,
        key: Key,
        child: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let child = child.into();
        let child_size = child.as_widget().size_hint();

        self.width = self.width.enclose(child_size.width);
        self.height = self.height.enclose(child_size.height);

        self.keys.push(key);
        self.children.push(child);
        self
    }

    /// Adds an element to the [`Column`], if `Some`.
    pub fn push_maybe(
        self,
        key: Key,
        child: Option<impl Into<Element<'a, Message, Theme, Renderer>>>,
    ) -> Self {
        if let Some(child) = child {
            self.push(key, child)
        } else {
            self
        }
    }

    /// Extends the [`Column`] with the given children.
    pub fn extend(
        self,
        children: impl IntoIterator<Item = (Key, Element<'a, Message, Theme, Renderer>)>,
    ) -> Self {
        children
            .into_iter()
            .fold(self, |items, (key, child)| items.push(key, child))
    }

    /// Sets the style of the [`Column`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Column`].
    // #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the message that will be produced when a child element on [`Column`] is grabbed
    /// for dragging.
    ///
    /// The message will be produced with the key of the grabbed child element.
    pub fn on_grab<F>(mut self, message: F) -> Self
    where
        F: Fn(Key) -> Message + 'a,
    {
        self.on_grab = Some(Box::new(message));
        self
    }

    /// Sets the message that will be produced when dragging starts after clicking a child
    /// element or dragged child element has been dragged to another position in the [`Column`].
    ///
    /// The message will be produced with the key of the dragged child element and the index
    /// of the drag position among the [`Column`] children.
    pub fn on_drag<F>(mut self, message: F) -> Self
    where
        F: Fn(Key, usize) -> Message + 'a,
    {
        self.on_drag = Some(Box::new(message));
        self
    }

    /// Sets the message that will be produced when the dragged child element is dropped in
    /// a valid drop location on the [`Column`].
    ///
    /// The message will be produced with the key of the dragged child element and the index
    /// of the drop position among the [`Column`] children.
    pub fn on_drop<F>(mut self, message: F) -> Self
    where
        F: Fn(Key, usize) -> Message + 'a,
    {
        self.on_drop = Some(Box::new(message));
        self
    }

    /// Sets the message that will be produced when the user cancels active dragging by
    /// right-clicking or when the dragging touch is lost.
    ///
    /// The message will be produced with the key of the child element that was being dragged.
    pub fn on_cancel<F>(mut self, message: F) -> Self
    where
        F: Fn(Key) -> Message + 'a,
    {
        self.on_cancel = Some(Box::new(message));
        self
    }

    /// Sets whether a marker line will be shown for the position among the [`Column`] children,
    /// where the dragged child element would be dropped if mouse button press or touch was
    /// released at current position.
    pub fn drop_position_marker(mut self, drop_position_marker: bool) -> Self {
        self.drop_position_marker = drop_position_marker;
        self
    }

    /// Sets whether a child element should follow the cursor or touch while being dragged.
    pub fn drag_follow(mut self, drag_follow: bool) -> Self {
        self.drag_follow = drag_follow;
        self
    }

    /// Sets whether a child element should follow the cursor laterally on the cross axis of the
    /// [`Column`] while being dragged.
    ///
    /// If set to `false`, child elements will only follow the cursor vertically along the main
    /// axis of the [`Column`] while being dragged.
    ///
    /// This has no effect if [`Column::drag_follow`] is set to `false`.
    pub fn drag_lateral(mut self, drag_lateral: bool) -> Self {
        self.drag_lateral = drag_lateral;
        self
    }

    /// Sets whether a child element should be centered on the cursor while being dragged.
    ///
    /// This has no effect if [`Column::drag_follow`] is set to `false`.
    pub fn drag_center(mut self, drag_center: bool) -> Self {
        self.drag_center = drag_center;
        self
    }
}

impl<'a, Key, Message, Theme, Renderer> Default for Column<'a, Key, Message, Theme, Renderer>
where
    Key: Copy + PartialEq,
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Key, Message, Theme, Renderer> FromIterator<(Key, Element<'a, Message, Theme, Renderer>)>
    for Column<'a, Key, Message, Theme, Renderer>
where
    Key: Copy + PartialEq,
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    fn from_iter<T: IntoIterator<Item = (Key, Element<'a, Message, Theme, Renderer>)>>(
        iter: T,
    ) -> Self {
        Self::with_children(iter)
    }
}

impl<'a, Key, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Column<'a, Key, Message, Theme, Renderer>
where
    Key: Copy + PartialEq + 'static,
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(State::<Key>::default())
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State<Key>>()
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let drag_state = tree.state.downcast_ref::<State<Key>>().drag;
        if let Some((event, cursor)) = propagage_event_to_children(&drag_state, &event, cursor) {
            for ((child, state), item_layout) in self
                .children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                child.as_widget_mut().update(
                    state,
                    event,
                    item_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }

        let state = tree.state.downcast_mut::<State<Key>>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if !shell.is_event_captured() && cursor.is_over(layout.bounds()) {
                    let mut position = cursor.position().unwrap();
                    for (key, item_layout) in self.keys.iter().zip(layout.children()) {
                        if cursor.is_over(item_layout.bounds()) {
                            if let Some(on_grab) = &self.on_grab {
                                shell.publish(on_grab(*key));
                            };
                            if self.drag_center {
                                let origin = item_layout.bounds().center();
                                if !self.drag_lateral {
                                    position.x = origin.x;
                                }
                                let drop_location = drop_location(&layout, position);
                                if let Some(on_drag) = self.on_drag.as_deref() {
                                    if Some(drop_location) != state.drag.drop_location() {
                                        let message = (on_drag)(*key, drop_location);
                                        shell.publish(message);
                                    }
                                }
                                state.drag = DragState::Dragged {
                                    key: *key,
                                    origin,
                                    position,
                                    drop_location,
                                };
                            } else {
                                let origin = position;
                                state.drag = DragState::Grabbed { key: *key, origin };
                            };
                            shell.request_redraw();
                            break;
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if let Some(key) = state.drag.key() {
                    state.drag = DragState::Idle;
                    if let Some(on_cancel) = &self.on_cancel {
                        shell.publish(on_cancel(key));
                    }
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => match state.drag {
                DragState::Grabbed { key, origin: _ } => {
                    if let Some(on_cancel) = &self.on_cancel {
                        shell.publish(on_cancel(key));
                    }
                    state.drag = DragState::Idle;
                }
                DragState::Dragged {
                    key,
                    origin: _,
                    position,
                    drop_location: _,
                } => {
                    if let Some(on_drop) = self.on_drop.as_deref() {
                        let drop_index = drop_location(&layout, position);
                        let message = (on_drop)(key, drop_index);
                        shell.publish(message);
                    }
                    state.drag = DragState::Idle;
                }
                _ => (),
            },
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => match state.drag {
                DragState::Grabbed { key, origin } | DragState::Dragged { key, origin, .. } => {
                    if cursor.position() == state.drag.last_position() {
                        return;
                    } else if let Some(mut position) = cursor.position() {
                        if !self.drag_lateral {
                            position.x = origin.x;
                        }
                        let drop_location = drop_location(&layout, position);
                        if let Some(on_drag) = self.on_drag.as_deref() {
                            if Some(drop_location) != state.drag.drop_location() {
                                let message = (on_drag)(key, drop_location);
                                shell.publish(message);
                            }
                        }
                        state.drag = DragState::Dragged {
                            key,
                            origin,
                            position,
                            drop_location,
                        };
                        if self.drag_follow {
                            shell.request_redraw();
                        }
                    }
                }
                _ => {
                    if cursor.is_over(layout.bounds()) {
                        shell.request_redraw();
                    }
                }
            },
            _ => {}
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.max_width(self.max_width);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.width,
            self.height,
            self.padding,
            self.spacing,
            self.align,
            &self.children,
            &mut tree.children,
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), item_layout)| {
                    child
                        .as_widget()
                        .operate(state, item_layout, renderer, operation);
                });
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let drag_state = tree.state.downcast_ref::<State<Key>>().drag;
        if !drag_state.is_idle() {
            return mouse::Interaction::Grabbing;
        }

        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), item_layout)| {
                let child_interaction = child.as_widget().mouse_interaction(
                    state,
                    item_layout,
                    cursor,
                    viewport,
                    renderer,
                );
                if self.on_drop.is_some() && cursor.is_over(item_layout.bounds()) {
                    mouse::Interaction::Pointer.max(child_interaction)
                } else {
                    child_interaction
                }
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let viewport = if self.clip {
                &clipped_viewport
            } else {
                viewport
            };
            let state = tree.state.downcast_ref::<State<Key>>();

            let mut deferred_drop_marker_y = None;
            let mut deferred_dragged_elem_key = None;
            let mut deferred_dragged_elem_translation = Vector::ZERO;

            if let DragState::Dragged {
                key,
                origin,
                position,
                drop_location,
            } = state.drag
            {
                if self.drop_position_marker {
                    deferred_drop_marker_y =
                        drop_location_marker_y(&layout, self.spacing, drop_location);
                }
                if self.drag_follow {
                    deferred_dragged_elem_key = Some(key);
                    deferred_dragged_elem_translation = position - origin;
                }
            }

            let mut deferred_dragged_elem = None;

            for (((child, key), state), item_layout) in self
                .children
                .iter()
                .zip(&self.keys)
                .zip(&tree.children)
                .zip(layout.children())
                .filter(|(_, item_layout)| item_layout.bounds().intersects(viewport))
            {
                if Some(*key) == deferred_dragged_elem_key {
                    deferred_dragged_elem = Some((child, state, item_layout));
                    continue;
                }

                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    item_layout,
                    cursor,
                    viewport,
                );
            }

            if deferred_drop_marker_y.is_some() || deferred_dragged_elem.is_some() {
                renderer.with_layer(*viewport, |renderer| {
                    if let Some(line_y) = deferred_drop_marker_y {
                        let line_color = theme.style(&self.class).color;
                        let line_width = 2.0;
                        let circle_outer_radius = 4.0;
                        let circle_inner_radius = circle_outer_radius - line_width;

                        // Draw line
                        let marker_line_bounds = Rectangle {
                            x: layout.bounds().x + self.padding.left + circle_inner_radius,
                            y: line_y - line_width * 0.5,
                            width: layout.bounds().width
                                - self.padding.horizontal()
                                - circle_inner_radius,
                            height: line_width,
                        };
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: marker_line_bounds,
                                ..renderer::Quad::default()
                            },
                            line_color,
                        );

                        // Draw circle at the start of the line
                        let marker_circle_bounds = Rectangle {
                            x: layout.bounds().x + self.padding.left - circle_outer_radius,
                            y: line_y - circle_outer_radius,
                            width: circle_outer_radius * 2.0,
                            height: circle_outer_radius * 2.0,
                        };
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: marker_circle_bounds,
                                border: Border {
                                    radius: Radius::new(circle_outer_radius),
                                    color: line_color,
                                    width: line_width,
                                },
                                ..renderer::Quad::default()
                            },
                            Color::TRANSPARENT,
                        );
                    }
                    if let Some((child, state, layout)) = deferred_dragged_elem {
                        renderer.with_translation(deferred_dragged_elem_translation, |renderer| {
                            child
                                .as_widget()
                                .draw(state, renderer, theme, style, layout, cursor, viewport);
                        });
                    }
                });
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer, translation)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct State<K>
where
    K: Copy + PartialEq,
{
    drag: DragState<K>,
}

impl<Key> Default for State<Key>
where
    Key: Copy + PartialEq,
{
    fn default() -> Self {
        Self {
            drag: DragState::Idle,
        }
    }
}

/// The current dragging state of a [`Column`].
#[derive(Default, Clone, Copy, PartialEq, Debug)]
enum DragState<K>
where
    K: Copy + PartialEq,
{
    /// No child element is being dragged.
    #[default]
    Idle,
    /// A [`Column`] child element is grabbed for dragging,
    /// but has not been moved yet.
    Grabbed { key: K, origin: Point },
    /// A [`Column`] child element is being dragged.
    Dragged {
        key: K,
        origin: Point,
        position: Point,
        drop_location: usize,
    },
}

impl<K> DragState<K>
where
    K: Copy + PartialEq,
{
    fn key(&self) -> Option<K> {
        match self {
            Self::Idle => None,
            Self::Grabbed { key, .. } => Some(*key),
            Self::Dragged { key, .. } => Some(*key),
        }
    }

    fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    fn last_position(&self) -> Option<Point> {
        match self {
            Self::Idle => None,
            Self::Grabbed { origin, .. } => Some(*origin),
            Self::Dragged { position, .. } => Some(*position),
        }
    }

    fn drop_location(&self) -> Option<usize> {
        match self {
            Self::Dragged { drop_location, .. } => Some(*drop_location),
            _ => None,
        }
    }
}

impl<'a, Key, Message, Theme, Renderer> From<Column<'a, Key, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Key: Copy + PartialEq + 'static,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: Column<'a, Key, Message, Theme, Renderer>) -> Self {
        Self::new(widget)
    }
}

/// Returns whether to propagate an [`Event`] to children of a [`Column`].
///
/// Will return `false` for mouse and touch events if a child element is being dragged.
fn propagage_event_to_children<'a, Key>(
    drag_state: &DragState<Key>,
    event: &'a Event,
    cursor: mouse::Cursor,
) -> Option<(&'a Event, mouse::Cursor)>
where
    Key: Copy + PartialEq,
{
    if !drag_state.is_idle() {
        match event {
            Event::Touch(touch::Event::FingerMoved { .. })
            | Event::Mouse(mouse::Event::CursorMoved { .. }) => None,
            _ => Some((event, mouse::Cursor::Unavailable)),
        }
    } else {
        Some((event, cursor))
    }
}

/// Returns the index of the drop location among the children of a [`Column`]
/// at given `position`.
fn drop_location(layout: &Layout, position: Point) -> usize {
    let mut index = 0;
    for item_layout in layout.children() {
        if position.y < item_layout.bounds().center_y() {
            break;
        }
        index += 1;
    }
    index
}

/// Returns Y-position for drop location marker on the `[Column]`.
fn drop_location_marker_y(layout: &Layout, spacing: f32, drop_location: usize) -> Option<f32> {
    if layout.children().count() == 0 {
        None
    } else if drop_location < layout.children().count() {
        let child_bounds_below = layout.children().nth(drop_location).unwrap().bounds();
        Some(child_bounds_below.y - spacing * 0.5)
    } else {
        let last_child_bounds = layout.children().last().unwrap().bounds();
        Some(last_child_bounds.y + last_child_bounds.height + spacing * 0.5)
    }
}

/// The appearance of of a [`Column`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The color of the drop position marker line indicating drop placement.
    pub color: Color,
}

/// The theme catalog of a [`Column`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Column`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>) -> Style {
        class(self)
    }
}

/// The default style of a [`Column`].
pub fn default(theme: &Theme) -> Style {
    Style {
        color: theme.palette().primary,
    }
}
