use crate::types::effects::{BlendMode, ItemEffect};

pub struct EffectsPipeline;

impl EffectsPipeline {
    /// Compiles a list of GPU shader effects into an FFmpeg video filter string
    pub fn compile_effects_to_ffmpeg(effects: &[ItemEffect]) -> Vec<String> {
        let mut filters = Vec::new();

        for item_effect in effects {
            if !item_effect.enabled {
                continue;
            }

            let eff = &item_effect.effect;
            match eff.gpu_effect_type.as_str() {
                "gpu-brightness" => {
                    let amount = eff.params.get("amount").copied().unwrap_or(0.0);
                    filters.push(format!("eq=brightness={:.3}", amount));
                }
                "gpu-contrast" => {
                    let amount = eff.params.get("amount").copied().unwrap_or(1.0);
                    filters.push(format!("eq=contrast={:.3}", amount));
                }
                "gpu-saturation" => {
                    let amount = eff.params.get("amount").copied().unwrap_or(1.0);
                    filters.push(format!("eq=saturation={:.3}", amount));
                }
                "gpu-hue-shift" => {
                    let shift = eff.params.get("shift").copied().unwrap_or(0.0);
                    let degrees = shift * 360.0;
                    filters.push(format!("hue=h={:.1}", degrees));
                }
                "gpu-grayscale" => {
                    filters.push("colorchannelmixer=.3:.4:.3:0:.3:.4:.3:0:.3:.4:.3".to_string());
                }
                "gpu-sepia" => {
                    filters.push("colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131".to_string());
                }
                "gpu-invert" => {
                    filters.push("negate".to_string());
                }
                "gpu-blur" => {
                    let radius = eff.params.get("radius").copied().unwrap_or(5.0);
                    filters.push(format!("gblur=sigma={:.1}", radius));
                }
                "gpu-vignette" => {
                    filters.push("vignette=PI/4".to_string());
                }
                _ => {}
            }
        }

        filters
    }

    /// Returns the FFmpeg blend mode filter name
    pub fn blend_mode_to_ffmpeg(mode: BlendMode) -> &'static str {
        match mode {
            BlendMode::Normal => "normal",
            BlendMode::Multiply => "multiply",
            BlendMode::Screen => "screen",
            BlendMode::Overlay => "overlay",
            BlendMode::Darken => "darken",
            BlendMode::Lighten => "lighten",
            BlendMode::ColorDodge => "dodge",
            BlendMode::ColorBurn => "burn",
            BlendMode::HardLight => "hardlight",
            BlendMode::SoftLight => "softlight",
            BlendMode::Difference => "difference",
            BlendMode::Exclusion => "exclusion",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::effects::GpuEffect;

    #[test]
    fn test_compile_effects_to_ffmpeg() {
        let eff = vec![
            ItemEffect::new("e1", GpuEffect::new("gpu-brightness").with_param("amount", 0.1)),
            ItemEffect::new("e2", GpuEffect::new("gpu-contrast").with_param("amount", 1.2)),
            ItemEffect::new("e3", GpuEffect::new("gpu-sepia")),
        ];

        let filters = EffectsPipeline::compile_effects_to_ffmpeg(&eff);
        assert_eq!(filters.len(), 3);
        assert_eq!(filters[0], "eq=brightness=0.100");
        assert_eq!(filters[1], "eq=contrast=1.200");
        assert!(filters[2].contains("colorchannelmixer="));
    }

    #[test]
    fn test_blend_mode_names() {
        assert_eq!(EffectsPipeline::blend_mode_to_ffmpeg(BlendMode::Multiply), "multiply");
        assert_eq!(EffectsPipeline::blend_mode_to_ffmpeg(BlendMode::Screen), "screen");
    }
}
