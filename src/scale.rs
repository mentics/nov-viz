//! How big a card is drawn at a given zoom.
//!
//! Zooming is for two things: reading, and getting around. For reading, a
//! card's text keeps a legible size (never below `MIN_SCALE` while text is
//! shown, never above `MAX_SCALE`), so the card is drawn at that size and the
//! layout spreads out or closes up around it. Past `HIDE_BELOW` the user is
//! looking at structure, or using zoom to travel; the cards then drop to the
//! true zoom and show no text.

/// Smallest card scale at which text is still shown.
pub const MIN_SCALE: f32 = 0.8;
/// Largest card scale; zooming in further only spreads the layout.
pub const MAX_SCALE: f32 = 1.5;
/// Zoom below which cards stop holding `MIN_SCALE` and show no text.
pub const HIDE_BELOW: f32 = 0.45;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardScale {
    /// Multiplier on the layout size of a card.
    pub size: f32,
    /// Whether the card's text is drawn.
    pub text: bool,
}

pub fn card_scale(zoom: f32) -> CardScale {
    if zoom < HIDE_BELOW {
        CardScale { size: zoom, text: false }
    } else {
        CardScale {
            size: zoom.clamp(MIN_SCALE, MAX_SCALE),
            text: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_size_between_the_thresholds() {
        assert_eq!(card_scale(1.0), CardScale { size: 1.0, text: true });
        assert_eq!(card_scale(0.5), CardScale { size: MIN_SCALE, text: true });
        assert_eq!(card_scale(4.0), CardScale { size: MAX_SCALE, text: true });
    }

    #[test]
    fn pops_to_true_zoom_without_text_when_far_out() {
        assert_eq!(card_scale(0.3), CardScale { size: 0.3, text: false });
    }
}
