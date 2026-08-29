use niri::layout::focus_ring::FocusRing;
use niri_config::{
    Color, CornerRadius, Gradient, GradientInterpolation, GradientRelativeTo, GradientShape,
};
use smithay::backend::renderer::element::RenderElement;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::utils::{Physical, Point, Rectangle, Size};

use super::{Args, TestCase};

pub struct GradientInward {
    border: FocusRing,
}

impl GradientInward {
    pub fn new(_args: Args) -> Self {
        let gradient = Gradient {
            from: Color::from_rgba8_unpremul(0, 128, 255, 255),
            to: Color::from_rgba8_unpremul(0, 128, 255, 0),
            angle: 0,
            relative_to: GradientRelativeTo::Window,
            in_: GradientInterpolation::default(),
            shape: GradientShape::Inward,
        };
        let border = FocusRing::new(niri_config::FocusRing {
            off: false,
            width: 48.,
            active_color: Color::default(),
            inactive_color: Color::default(),
            urgent_color: Color::default(),
            active_gradient: Some(gradient),
            inactive_gradient: None,
            urgent_gradient: None,
        });

        Self { border }
    }
}

impl TestCase for GradientInward {
    fn render(
        &mut self,
        renderer: &mut GlesRenderer,
        size: Size<i32, Physical>,
    ) -> Vec<Box<dyn RenderElement<GlesRenderer>>> {
        let margin = 96;
        let window_size = Size::from((size.w - margin * 2, size.h - margin * 2)).to_f64();
        let location = Point::from((margin, margin)).to_f64();

        self.border.update_render_elements(
            window_size,
            true,
            true,
            false,
            Rectangle::default(),
            CornerRadius::from(64.),
            1.,
            1.,
        );

        let mut elements = Vec::new();
        self.border.render(renderer, location, &mut |element| {
            elements.push(Box::new(element) as _)
        });
        elements
    }
}
