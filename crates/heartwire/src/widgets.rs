//! Panel button: the original CSS states (rest, hover, selected, disabled and
//! keyboard focus) as an egui widget, announced to screen readers by name.

use eframe::egui::{
    self, Color32, FontId, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo,
};

use crate::theme;

#[derive(Clone, Copy)]
pub struct Look {
    pub fill: Color32,
    pub hover_fill: Color32,
    pub stroke: Color32,
    pub text: Color32,
    pub hover_text: Color32,
}

impl Look {
    pub const fn plain(text: Color32, hover_text: Color32, hover_fill: Color32) -> Self {
        Self {
            fill: Color32::TRANSPARENT,
            hover_fill,
            stroke: Color32::TRANSPARENT,
            text,
            hover_text,
        }
    }

    pub const fn filled(
        fill: Color32,
        hover_fill: Color32,
        stroke: Color32,
        text: Color32,
    ) -> Self {
        Self {
            fill,
            hover_fill,
            stroke,
            text,
            hover_text: text,
        }
    }
}

pub struct PanelButton<'a> {
    icon: Option<char>,
    label: &'a str,
    font: FontId,
    look: Look,
    size: Vec2,
    radius: u8,
    role: Role,
    selected: Option<bool>,
}

impl<'a> PanelButton<'a> {
    pub fn new(label: &'a str, look: Look) -> Self {
        Self {
            icon: None,
            label,
            font: FontId::proportional(12.0),
            look,
            size: Vec2::new(0.0, 30.0),
            radius: 7,
            role: Role::Button,
            selected: None,
        }
    }

    pub fn icon(mut self, icon: Option<char>) -> Self {
        self.icon = icon;
        self
    }

    pub fn font(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = radius;
        self
    }

    /// Tabs and toggles report their selected state to assistive technology.
    pub fn selected(mut self, role: Role, selected: bool) -> Self {
        self.role = role;
        self.selected = Some(selected);
        self
    }
}

const GAP: f32 = 7.0;
const PADDING: f32 = 9.0;

impl Widget for PanelButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let painter = ui.painter();
        // PLACEHOLDER text takes the state colour when the galley is painted.
        let label = (!self.label.is_empty()).then(|| {
            painter.layout_no_wrap(
                self.label.to_owned(),
                self.font.clone(),
                Color32::PLACEHOLDER,
            )
        });
        let icon = self.icon.map(|glyph| {
            painter.layout_no_wrap(
                glyph.to_string(),
                theme::icon_font(self.font.size + 2.0),
                Color32::PLACEHOLDER,
            )
        });
        let content = icon
            .iter()
            .chain(&label)
            .map(|galley| galley.size().x)
            .sum::<f32>()
            + if icon.is_some() && label.is_some() {
                GAP
            } else {
                0.0
            };
        let size = Vec2::new(self.size.x.max(content + PADDING * 2.0), self.size.y);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        let enabled = ui.is_enabled();
        let hovered = enabled && response.hovered();
        let fade = |color: Color32| {
            if enabled {
                color
            } else {
                color.gamma_multiply(ui.visuals().disabled_alpha)
            }
        };
        let look = self.look;
        let fill = fade(if hovered { look.hover_fill } else { look.fill });
        let text = fade(if hovered { look.hover_text } else { look.text });
        let painter = ui.painter();
        painter.rect(
            rect,
            self.radius,
            fill,
            Stroke::new(1.0, fade(look.stroke)),
            StrokeKind::Inside,
        );
        if response.has_focus() {
            painter.rect_stroke(
                rect.expand(2.0),
                self.radius + 2,
                Stroke::new(2.0, theme::GREEN),
                StrokeKind::Outside,
            );
        }
        let mut x = rect.center().x - content / 2.0;
        for galley in icon.into_iter().chain(label) {
            let y = rect.center().y - galley.size().y / 2.0;
            let width = galley.size().x;
            painter.galley(egui::pos2(x, y), galley, text);
            x += width + GAP;
        }

        let (role, label, selected) = (self.role, self.label.to_owned(), self.selected);
        response.widget_info(|| match selected {
            Some(selected) => WidgetInfo::selected(role, enabled, selected, &label),
            None => WidgetInfo::labeled(role, enabled, &label),
        });
        response
    }
}
