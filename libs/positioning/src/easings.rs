use keyframe::EasingFunction;

/// Easing based on https://easings.net/#
#[derive(Clone, Copy)]
pub enum Easing {
    Linear,

    EaseIn,
    EaseOut,
    EaseInOut,

    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,

    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,

    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,

    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,

    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,

    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,

    EaseInBack,
    EaseOutBack,
    EaseInOutBack,

    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,

    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

impl EasingFunction for Easing {
    fn y(&self, x: f64) -> f64 {
        use keyframe::functions::*;
        match self {
            Easing::Linear => Linear.y(x),
            Easing::EaseIn => EaseIn.y(x),
            Easing::EaseOut => EaseOut.y(x),
            Easing::EaseInOut => EaseInOut.y(x),
            Easing::EaseInQuad => EaseInQuad.y(x),
            Easing::EaseOutQuad => EaseOutQuad.y(x),
            Easing::EaseInOutQuad => EaseInOutQuad.y(x),
            Easing::EaseInCubic => EaseInCubic.y(x),
            Easing::EaseOutCubic => EaseOutCubic.y(x),
            Easing::EaseInOutCubic => EaseInOutCubic.y(x),
            Easing::EaseInQuart => EaseInQuart.y(x),
            Easing::EaseOutQuart => EaseOutQuart.y(x),
            Easing::EaseInOutQuart => EaseInOutQuart.y(x),
            Easing::EaseInQuint => EaseInQuint.y(x),
            Easing::EaseOutQuint => EaseOutQuint.y(x),
            Easing::EaseInOutQuint => EaseInOutQuint.y(x),
            Easing::EaseInExpo => Self::ease_in_expo(x),
            Easing::EaseOutExpo => Self::ease_out_expo(x),
            Easing::EaseInOutExpo => Self::ease_in_out_expo(x),
            Easing::EaseInCirc => Self::ease_in_circ(x),
            Easing::EaseOutCirc => Self::ease_out_circ(x),
            Easing::EaseInOutCirc => Self::ease_in_out_circ(x),
            Easing::EaseInBack => Self::ease_in_back(x),
            Easing::EaseOutBack => Self::ease_out_back(x),
            Easing::EaseInOutBack => Self::ease_in_out_back(x),
            Easing::EaseInElastic => Self::ease_in_elastic(x),
            Easing::EaseOutElastic => Self::ease_out_elastic(x),
            Easing::EaseInOutElastic => Self::ease_in_out_elastic(x),
            Easing::EaseInBounce => Self::ease_in_bounce(x),
            Easing::EaseOutBounce => Self::ease_out_bounce(x),
            Easing::EaseInOutBounce => Self::ease_in_out_bounce(x),
        }
    }
}

impl Easing {
    pub fn from_name(name: &str) -> Option<Easing> {
        let name = name.to_lowercase();
        let easing = match name.as_str() {
            "linear" => Easing::Linear,
            "easein" => Easing::EaseIn,
            "easeout" => Easing::EaseOut,
            "easeinout" => Easing::EaseInOut,
            "easeinquad" => Easing::EaseInQuad,
            "easeoutquad" => Easing::EaseOutQuad,
            "easeinoutquad" => Easing::EaseInOutQuad,
            "easeincubic" => Easing::EaseInCubic,
            "easeoutcubic" => Easing::EaseOutCubic,
            "easeinoutcubic" => Easing::EaseInOutCubic,
            "easeinquart" => Easing::EaseInQuart,
            "easeoutquart" => Easing::EaseOutQuart,
            "easeinoutquart" => Easing::EaseInOutQuart,
            "easeinquint" => Easing::EaseInQuint,
            "easeoutquint" => Easing::EaseOutQuint,
            "easeinoutquint" => Easing::EaseInOutQuint,
            "easeinexpo" => Easing::EaseInExpo,
            "easeoutexpo" => Easing::EaseOutExpo,
            "easeinoutexpo" => Easing::EaseInOutExpo,
            "easeincirc" => Easing::EaseInCirc,
            "easeoutcirc" => Easing::EaseOutCirc,
            "easeinoutcirc" => Easing::EaseInOutCirc,
            "easeinback" => Easing::EaseInBack,
            "easeoutback" => Easing::EaseOutBack,
            "easeinoutback" => Easing::EaseInOutBack,
            "easeinelastic" => Easing::EaseInElastic,
            "easeoutelastic" => Easing::EaseOutElastic,
            "easeinoutelastic" => Easing::EaseInOutElastic,
            "easeinbounce" => Easing::EaseInBounce,
            "easeoutbounce" => Easing::EaseOutBounce,
            "easeinoutbounce" => Easing::EaseInOutBounce,
            _ => {
                return None;
            }
        };
        Some(easing)
    }

    #[inline]
    fn ease_in_expo(x: f64) -> f64 {
        if x == 0.0 {
            0.0
        } else {
            2.0f64.powf(10.0 * (x - 1.0))
        }
    }

    #[inline]
    fn ease_out_expo(x: f64) -> f64 {
        if x == 1.0 {
            1.0
        } else {
            1.0 - 2.0f64.powf(-10.0 * x)
        }
    }

    #[inline]
    fn ease_in_out_expo(x: f64) -> f64 {
        if x == 0.0 {
            0.0
        } else if x == 1.0 {
            1.0
        } else if x < 0.5 {
            2.0f64.powf(20.0 * x - 10.0) / 2.0
        } else {
            (2.0 - 2.0f64.powf(-20.0 * x + 10.0)) / 2.0
        }
    }

    #[inline]
    fn ease_in_circ(x: f64) -> f64 {
        1.0 - f64::sqrt(1.0 - x * x)
    }

    #[inline]
    fn ease_out_circ(x: f64) -> f64 {
        f64::sqrt(1.0 - (x - 1.0).powi(2))
    }

    #[inline]
    fn ease_in_out_circ(x: f64) -> f64 {
        if x < 0.5 {
            (1.0 - f64::sqrt(1.0 - (2.0 * x) * (2.0 * x))) / 2.0
        } else {
            (f64::sqrt(1.0 - (-2.0 * x + 2.0).powi(2)) + 1.0) / 2.0
        }
    }

    #[inline]
    fn ease_in_back(x: f64) -> f64 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        c3 * x * x * x - c1 * x * x
    }

    #[inline]
    fn ease_out_back(x: f64) -> f64 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
    }

    #[inline]
    fn ease_in_out_back(x: f64) -> f64 {
        let c1 = 1.70158;
        let c2 = c1 * 1.525;
        if x < 0.5 {
            ((2.0 * x).powi(2) * ((c2 + 1.0) * 2.0 * x - c2)) / 2.0
        } else {
            ((2.0 * x - 2.0).powi(2) * ((c2 + 1.0) * (x * 2.0 - 2.0) + c2) + 2.0) / 2.0
        }
    }

    #[inline]
    fn ease_in_elastic(x: f64) -> f64 {
        let c4 = 2.0 * std::f64::consts::FRAC_PI_3;

        if x == 0.0 {
            0.0
        } else if x == 1.0 {
            1.0
        } else {
            -2.0f64.powf(10.0 * x - 10.0) * f64::sin((x * 10.0 - 10.75) * c4)
        }
    }

    #[inline]
    fn ease_out_elastic(x: f64) -> f64 {
        let c4 = 2.0 * std::f64::consts::FRAC_PI_3;

        if x == 0.0 {
            0.0
        } else if x == 1.0 {
            1.0
        } else {
            2.0f64.powf(-10.0 * x) * f64::sin((x * 10.0 - 0.75) * c4) + 1.0
        }
    }

    #[inline]
    fn ease_in_out_elastic(x: f64) -> f64 {
        let c5 = (2.0 * std::f64::consts::PI) / 4.5;

        if x == 0.0 {
            0.0
        } else if x == 1.0 {
            1.0
        } else if x < 0.5 {
            -(2.0f64.powf(20.0 * x - 10.0) * f64::sin((20.0 * x - 11.125) * c5)) / 2.0
        } else {
            (2.0f64.powf(-20.0 * x + 10.0) * f64::sin((20.0 * x - 11.125) * c5)) / 2.0 + 1.0
        }
    }

    #[inline]
    fn ease_in_bounce(x: f64) -> f64 {
        1.0 - Self::ease_out_bounce(1.0 - x)
    }

    #[inline]
    fn ease_out_bounce(x: f64) -> f64 {
        let n1 = 7.5625;
        let d1 = 2.75;

        if x < 1.0 / d1 {
            n1 * x * x
        } else if x < 2.0 / d1 {
            let x_adjusted = x - 1.5 / d1;
            n1 * x_adjusted * x_adjusted + 0.75
        } else if x < 2.5 / d1 {
            let x_adjusted = x - 2.25 / d1;
            n1 * x_adjusted * x_adjusted + 0.9375
        } else {
            let x_adjusted = x - 2.625 / d1;
            n1 * x_adjusted * x_adjusted + 0.984375
        }
    }

    #[inline]
    fn ease_in_out_bounce(x: f64) -> f64 {
        if x < 0.5 {
            (1.0 - Self::ease_out_bounce(1.0 - 2.0 * x)) / 2.0
        } else {
            (1.0 + Self::ease_out_bounce(2.0 * x - 1.0)) / 2.0
        }
    }
}
