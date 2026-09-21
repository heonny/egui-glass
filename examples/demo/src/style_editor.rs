use egui_glass::GlassStyle;

pub(super) struct StyleEditor {
    pub style: GlassStyle,
    pub name: &'static str,
    baseline: GlassStyle,
}

impl StyleEditor {
    pub fn new(name: &'static str, style: GlassStyle) -> Self {
        Self { style, name, baseline: style }
    }

    pub fn modified(&self) -> bool {
        self.style != self.baseline
    }

    pub fn reset(&mut self) {
        self.style = self.baseline;
    }
}

pub(super) fn rust_code(style: GlassStyle) -> String {
    let GlassStyle {
        corner_radius, corner_smoothing, blur, refraction, edge_width, chromatic, tint,
        brightness, saturation, specular, border, shadow, shadow_radius, shadow_offset, shadow_spread,
    } = style;
    let mut code = String::from("egui_glass::GlassStyle {\n");
    for (name, value) in [
        ("corner_radius", corner_radius), ("corner_smoothing", corner_smoothing),
        ("blur", blur), ("refraction", refraction), ("edge_width", edge_width), ("chromatic", chromatic),
        ("brightness", brightness), ("saturation", saturation), ("specular", specular), ("border", border),
        ("shadow", shadow), ("shadow_radius", shadow_radius), ("shadow_offset", shadow_offset), ("shadow_spread", shadow_spread),
    ] {
        let literal = match value {
            f32::INFINITY => "f32::INFINITY".to_owned(),
            f32::NEG_INFINITY => "f32::NEG_INFINITY".to_owned(),
            value if value.is_nan() => "f32::NAN".to_owned(),
            value => format!("{value:?}"),
        };
        code.push_str(&format!("    {name}: {literal},\n"));
    }
    let [r, g, b, a] = tint.to_array();
    code.push_str(&format!("    tint: egui::Color32::from_rgba_premultiplied({r}, {g}, {b}, {a}),\n}}"));
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_restores_the_selected_or_imported_style() {
        let mut editor = StyleEditor::new("Regular", GlassStyle::regular());
        editor.style.blur = 25.0;
        assert!(editor.modified());
        editor.reset();
        assert_eq!(editor.style, GlassStyle::regular());
        assert!(!editor.modified());
        let imported = GlassStyle { blur: 17.25, ..GlassStyle::dark() };
        editor = StyleEditor::new("Imported", imported);
        editor.style.brightness = 1.5;
        editor.reset();
        assert_eq!(editor.style, imported);
        assert_eq!(editor.name, "Imported");
        assert!(!editor.modified());
    }

    #[test]
    fn returning_to_baseline_values_clears_modified_state() {
        let mut editor = StyleEditor::new("Clear", GlassStyle::clear());
        editor.style.blur = 40.0;
        assert!(editor.modified());
        editor.style.blur = 2.0;
        assert!(!editor.modified());
    }

    #[test]
    fn rust_export_preserves_float_precision_and_premultiplied_tint() {
        let style = GlassStyle {
            blur: 17.123457,
            corner_radius: f32::INFINITY,
            tint: egui::Color32::from_rgba_premultiplied(12, 23, 34, 87),
            ..GlassStyle::regular()
        };
        let code = rust_code(style);
        assert!(code.contains("blur: 17.123457,"));
        assert!(code.contains("corner_radius: f32::INFINITY,"));
        assert!(code.contains("egui::Color32::from_rgba_premultiplied(12, 23, 34, 87)"));
        assert_eq!(code.matches(": ").count(), 15);
    }
}
