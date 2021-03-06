
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use num::{Float, NumCast, ToPrimitive};
use position::{self, Depth, Dimensions, HorizontalAlign, Position, VerticalAlign};
use theme::Theme;
use ui::{UiId, Ui};
use utils::{clamp, percentage, value_from_perc};
use widget::{self, Widget};


/// Linear value selection. If the slider's width is greater than it's height, it will
/// automatically become a horizontal slider, otherwise it will be a vertical slider. Its reaction
/// is triggered if the value is updated or if the mouse button is released while the cursor is
/// above the rectangle.
pub struct Slider<'a, T, F> {
    value: T,
    min: T,
    max: T,
    pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    dim: Dimensions,
    depth: Depth,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Styling for the Slider, necessary for constructing its renderable Element.
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<f64>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_label_color: Option<Color>,
    pub maybe_label_font_size: Option<u32>,
}

/// Represents the state of the Slider widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State<T> {
    value: T,
    min: T,
    max: T,
    maybe_label: Option<String>,
    interaction: Interaction,
}

/// The ways in which the Slider can be interacted with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}


impl<T> State<T> {
    /// Return the color associated with the state.
    fn color(&self, color: Color) -> Color {
        match self.interaction {
            Interaction::Normal => color,
            Interaction::Highlighted => color.highlighted(),
            Interaction::Clicked => color.clicked(),
        }
    }
}

/// Check the current state of the slider.
fn get_new_interaction(is_over: bool, prev: Interaction, mouse: Mouse) -> Interaction {
    use mouse::ButtonState::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left) {
        (true,  Normal,  Down) => Normal,
        (true,  _,       Down) => Clicked,
        (true,  _,       Up)   => Highlighted,
        (false, Clicked, Down) => Clicked,
        _ => Normal,
    }
}

impl<'a, T, F> Slider<'a, T, F> {

    /// Construct a new Slider widget.
    pub fn new(value: T, min: T, max: T) -> Slider<'a, T, F> {
        Slider {
            value: value,
            min: min,
            max: max,
            pos: Position::default(),
            maybe_h_align: None,
            maybe_v_align: None,
            dim: [192.0, 48.0],
            depth: 0.0,
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Set the reaction for the Slider. It will be triggered if the value is updated or if the
    /// mouse button is released while the cursor is above the rectangle.
    pub fn react(mut self, reaction: F) -> Slider<'a, T, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

}

impl<'a, T, F> Widget for Slider<'a, T, F>
    where
        F: FnMut(T),
        T: ::std::any::Any + ::std::fmt::Debug + Float + NumCast + ToPrimitive,
{
    type State = State<T>;
    type Style = Style;
    fn unique_kind(&self) -> &'static str { "Slider" }
    fn init_state(&self) -> State<T> {
        State {
            value: self.value,
            min: self.min,
            max: self.max,
            maybe_label: None,
            interaction: Interaction::Normal,
        }
    }
    fn style(&self) -> Style { self.style.clone() }

    /// Update the state of the Button.
    fn update<C>(mut self,
                 prev_state: &widget::State<State<T>>,
                 style: &Style,
                 ui_id: UiId,
                 ui: &mut Ui<C>) -> widget::State<Option<State<T>>>
        where
            C: CharacterCache,
    {
        use utils::{is_over_rect, map_range};

        let widget::State { ref state, .. } = *prev_state;
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.align.horizontal);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.align.vertical);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let mouse = ui.get_mouse_state(ui_id).relative_to(xy);
        let is_over = is_over_rect([0.0, 0.0], mouse.xy, dim);
        let new_interaction = 
            if self.enabled {
                get_new_interaction(is_over, state.interaction, mouse)
            } else {
                //Slider is disabled, so pretend the interaction is normal
                Interaction::Normal
            };

        let frame = style.frame(&ui.theme);
        let frame_2 = frame * 2.0;
        let (inner_w, inner_h) = (dim[0] - frame_2, dim[1] - frame_2);
        let (half_inner_w, half_inner_h) = (inner_w / 2.0, inner_h / 2.0);

        let is_horizontal = dim[0] > dim[1];

        let new_value = if is_horizontal {
            // Horizontal.
            let w = match (is_over, state.interaction, new_interaction) {
                (true, Interaction::Highlighted, Interaction::Clicked) |
                (_, Interaction::Clicked, Interaction::Clicked) => {
                    let w = map_range(mouse.xy[0], -half_inner_w, half_inner_w, 0.0, inner_w);
                    clamp(w, 0.0, inner_w)
                },
                _ => {
                    let value_percentage = percentage(self.value, self.min, self.max);
                    clamp(value_percentage as f64 * inner_w, 0.0, inner_w)
                },
            };
            value_from_perc((w / inner_w) as f32, self.min, self.max)
        } else {
            // Vertical.
            let h = match (is_over, state.interaction, new_interaction) {
                (true, Interaction::Highlighted, Interaction::Clicked) |
                (_, Interaction::Clicked, Interaction::Clicked) => {
                    let h = map_range(mouse.xy[1], -half_inner_h, half_inner_h, 0.0, inner_h);
                    clamp(h, 0.0, inner_h)
                },
                _ => {
                    let value_percentage = percentage(self.value, self.min, self.max);
                    clamp(value_percentage as f64 * inner_h, 0.0, inner_h)
                },
            };
            value_from_perc((h / inner_h) as f32, self.min, self.max)
        };

        // React.
        match self.maybe_react {
            Some(ref mut react) => {
                if self.value != new_value || match (state.interaction, new_interaction) {
                    (Interaction::Highlighted, Interaction::Clicked) |
                    (Interaction::Clicked, Interaction::Highlighted) => true,
                    _ => false,
                } { react(new_value) }
            }, None => (),
        }

        // A function for constructing a new state.
        let new_state = || {
            State {
                interaction: new_interaction,
                value: self.value,
                min: self.min,
                max: self.max,
                maybe_label: self.maybe_label.as_ref().map(|label| label.to_string()),
            }
        };

        // Check whether or not the state has changed since the previous update.
        let state_has_changed = state.interaction != new_interaction
            || state.value != self.value
            || state.min != self.min || state.max != self.max
            || state.maybe_label.as_ref().map(|string| &string[..]) != self.maybe_label;

        // Construct the new state if there was a change.
        let maybe_new_state = if state_has_changed { Some(new_state()) } else { None };

        widget::State { state: maybe_new_state, dim: dim, xy: xy, depth: self.depth }
    }

    /// Construct an Element from the given Button State.
    fn draw<C>(new_state: &widget::State<State<T>>, style: &Style, ui: &mut Ui<C>) -> Element
        where
            C: CharacterCache,
    {
        use elmesque::form::{collage, rect, text};

        let widget::State { ref state, dim, xy, .. } = *new_state;
        let frame = style.frame(&ui.theme);
        let (inner_w, inner_h) = (dim[0] - frame * 2.0, dim[1] - frame * 2.0);
        let frame_color = state.color(style.frame_color(&ui.theme));
        let color = state.color(style.color(&ui.theme));

        let new_value = NumCast::from(state.value).unwrap();
        let is_horizontal = dim[0] > dim[1];
        let (pad_rel_xy, pad_dim) = if is_horizontal {
            // Horizontal.
            let value_percentage = percentage(new_value, state.min, state.max);
            let w = clamp(value_percentage as f64 * inner_w, 0.0, inner_w);
            let rel_xy = [-(inner_w - w) / 2.0, 0.0];
            (rel_xy, [w, inner_h])
        } else {
            // Vertical.
            let value_percentage = percentage(new_value, state.min, state.max);
            let h = clamp(value_percentage as f64 * inner_h, 0.0, inner_h);
            let rel_xy = [0.0, -(inner_h - h) / 2.0];
            (rel_xy, [inner_w, h])
        };

        // Rectangle frame / backdrop Form.
        let frame_form = rect(dim[0], dim[1])
            .filled(frame_color);
        // Slider rectangle Form.
        let pad_form = rect(pad_dim[0], pad_dim[1])
            .filled(color)
            .shift(pad_rel_xy[0], pad_rel_xy[1]);

        // Label Form.
        let maybe_label_form = state.maybe_label.as_ref().map(|label_text| {
            use elmesque::text::Text;
            use label;
            const TEXT_PADDING: f64 = 10.0;
            let label_color = style.label_color(&ui.theme);
            let size = style.label_font_size(&ui.theme);
            let label_w = label::width(ui, size, &label_text);
            let is_horizontal = dim[0] > dim[1];
            let l_pos = if is_horizontal {
                let x = position::align_left_of(dim[0], label_w) + TEXT_PADDING;
                [x, 0.0]
            } else {
                let y = position::align_bottom_of(dim[1], size as f64) + TEXT_PADDING;
                [0.0, y]
            };
            text(Text::from_string(label_text.clone()).color(label_color).height(size as f64))
                .shift(l_pos[0].floor(), l_pos[1].floor())
                .shift(xy[0].floor(), xy[1].floor())
        });

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pad_form).into_iter())
            .map(|form| form.shift(xy[0], xy[1]))
            .chain(maybe_label_form.into_iter());

        // Collect the Forms into a renderable Element.
        collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
    }

}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_slider.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_slider.as_ref().map(|style| {
            style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_slider.as_ref().map(|style| {
            style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_slider.as_ref().map(|style| {
            style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_slider.as_ref().map(|style| {
            style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

}


impl<'a, T, F> Colorable for Slider<'a, T, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, T, F> Frameable for Slider<'a, T, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, T, F> Labelable<'a> for Slider<'a, T, F> {
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.style.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_label_font_size = Some(size);
        self
    }
}

impl<'a, T, F> position::Positionable for Slider<'a, T, F> {
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        Slider { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        Slider { maybe_v_align: Some(v_align), ..self }
    }
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
}

impl<'a, T, F> position::Sizeable for Slider<'a, T, F> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        Slider { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        Slider { dim: [w, h], ..self }
    }
}

