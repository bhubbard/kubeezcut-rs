use crate::types::keyframe::{BezierControlPoints, EasingType, Keyframe, SpringParameters};

pub struct KeyframeEvaluator;

impl KeyframeEvaluator {
    /// Evaluates the value of an animated property at a target frame `t` given a slice of keyframes.
    pub fn evaluate(keyframes: &[Keyframe], target_frame: i64) -> f64 {
        if keyframes.is_empty() {
            return 0.0;
        }
        if keyframes.len() == 1 {
            return keyframes[0].value;
        }

        // Before first keyframe
        if target_frame <= keyframes[0].frame {
            return keyframes[0].value;
        }

        // After last keyframe
        let last_idx = keyframes.len() - 1;
        if target_frame >= keyframes[last_idx].frame {
            return keyframes[last_idx].value;
        }

        // Find the bounding keyframe segment
        for i in 0..last_idx {
            let k1 = &keyframes[i];
            let k2 = &keyframes[i + 1];

            if target_frame >= k1.frame && target_frame <= k2.frame {
                let range = (k2.frame - k1.frame) as f64;
                if range <= 0.0 {
                    return k1.value;
                }

                let progress = (target_frame - k1.frame) as f64 / range;
                let eased_progress = Self::apply_easing(progress, k1.easing, k1.easing_config.as_ref());
                return k1.value + eased_progress * (k2.value - k1.value);
            }
        }

        keyframes[last_idx].value
    }

    /// Computes easing progress in [0.0, 1.0]
    pub fn apply_easing(
        t: f64,
        easing: EasingType,
        config: Option<&crate::types::keyframe::EasingConfig>,
    ) -> f64 {
        let t_clamped = t.clamp(0.0, 1.0);

        match easing {
            EasingType::Linear => t_clamped,
            EasingType::EaseIn => t_clamped * t_clamped,
            EasingType::EaseOut => t_clamped * (2.0 - t_clamped),
            EasingType::EaseInOut => {
                if t_clamped < 0.5 {
                    2.0 * t_clamped * t_clamped
                } else {
                    -1.0 + (4.0 - 2.0 * t_clamped) * t_clamped
                }
            }
            EasingType::CubicBezier => {
                let default_pts = BezierControlPoints::default();
                let pts = config
                    .and_then(|c| c.bezier)
                    .unwrap_or(default_pts);
                Self::solve_cubic_bezier(t_clamped, pts)
            }
            EasingType::Spring => {
                let default_spring = SpringParameters::default();
                let spring = config
                    .and_then(|c| c.spring)
                    .unwrap_or(default_spring);
                Self::solve_spring(t_clamped, spring)
            }
        }
    }

    /// Solves Cubic Bezier with bisection search
    pub fn solve_cubic_bezier(x_target: f64, pts: BezierControlPoints) -> f64 {
        let mut low = 0.0;
        let mut high = 1.0;
        let mut s = x_target;

        for _ in 0..16 {
            let x = 3.0 * (1.0 - s).powi(2) * s * pts.x1 + 3.0 * (1.0 - s) * s.powi(2) * pts.x2 + s.powi(3);
            if (x - x_target).abs() < 1e-5 {
                break;
            }
            if x < x_target {
                low = s;
            } else {
                high = s;
            }
            s = (low + high) / 2.0;
        }

        // Evaluate Y(s)
        3.0 * (1.0 - s).powi(2) * s * pts.y1 + 3.0 * (1.0 - s) * s.powi(2) * pts.y2 + s.powi(3)
    }

    /// Solves damped spring physics
    pub fn solve_spring(t: f64, spring: SpringParameters) -> f64 {
        // Damped harmonic oscillator equation
        let omega = (spring.tension / spring.mass).sqrt();
        let zeta = spring.friction / (2.0 * (spring.tension * spring.mass).sqrt());

        if zeta < 1.0 {
            // Underdamped
            let omega_d = omega * (1.0 - zeta * zeta).sqrt();
            1.0 - (-zeta * omega * t * 4.0).exp()
                * ((omega_d * t * 4.0).cos() + (zeta / (1.0 - zeta * zeta).sqrt()) * (omega_d * t * 4.0).sin())
        } else {
            // Overdamped / critically damped
            1.0 - (-(1.0 + zeta) * omega * t * 4.0).exp()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_keyframe_eval() {
        let kfs = vec![
            Keyframe::new("k1", 0, 0.0, EasingType::Linear),
            Keyframe::new("k2", 100, 200.0, EasingType::Linear),
        ];

        assert_eq!(KeyframeEvaluator::evaluate(&kfs, 0), 0.0);
        assert_eq!(KeyframeEvaluator::evaluate(&kfs, 50), 100.0);
        assert_eq!(KeyframeEvaluator::evaluate(&kfs, 100), 200.0);
        assert_eq!(KeyframeEvaluator::evaluate(&kfs, 150), 200.0); // clamped at last
        assert_eq!(KeyframeEvaluator::evaluate(&kfs, -20), 0.0);   // clamped at first
    }

    #[test]
    fn test_easing_curves() {
        assert_eq!(KeyframeEvaluator::apply_easing(0.0, EasingType::EaseIn, None), 0.0);
        assert_eq!(KeyframeEvaluator::apply_easing(1.0, EasingType::EaseIn, None), 1.0);
        assert_eq!(KeyframeEvaluator::apply_easing(0.5, EasingType::EaseIn, None), 0.25);

        assert_eq!(KeyframeEvaluator::apply_easing(0.5, EasingType::EaseOut, None), 0.75);
    }

    #[test]
    fn test_cubic_bezier_solver() {
        let pts = BezierControlPoints::default();
        let y_mid = KeyframeEvaluator::solve_cubic_bezier(0.5, pts);
        assert!(y_mid > 0.0 && y_mid < 1.0);
    }
}
