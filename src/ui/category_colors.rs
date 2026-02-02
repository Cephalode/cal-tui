use ratatui::style::Color;

/// Get a color for a category name
pub fn category_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "work" => Color::Blue,
        "personal" => Color::Green,
        "meeting" => Color::Yellow,
        "reminder" => Color::Magenta,
        "appointment" => Color::Cyan,
        "task" => Color::LightCyan,
        "birthday" => Color::LightRed,
        "holiday" => Color::LightYellow,
        "travel" => Color::LightGreen,
        "exercise" => Color::Red,
        _ => Color::Gray, // Default color
    }
}

/// Get a list of predefined category names
pub fn predefined_categories() -> Vec<&'static str> {
    vec![
        "Work",
        "Personal",
        "Meeting",
        "Reminder",
        "Appointment",
        "Task",
        "Birthday",
        "Holiday",
        "Travel",
        "Exercise",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_color_work() {
        assert_eq!(category_color("work"), Color::Blue);
        assert_eq!(category_color("Work"), Color::Blue);
        assert_eq!(category_color("WORK"), Color::Blue);
    }

    #[test]
    fn test_category_color_personal() {
        assert_eq!(category_color("personal"), Color::Green);
    }

    #[test]
    fn test_category_color_meeting() {
        assert_eq!(category_color("meeting"), Color::Yellow);
    }

    #[test]
    fn test_category_color_default() {
        assert_eq!(category_color("unknown"), Color::Gray);
        assert_eq!(category_color("random"), Color::Gray);
    }

    #[test]
    fn test_predefined_categories() {
        let categories = predefined_categories();
        assert!(categories.contains(&"Work"));
        assert!(categories.contains(&"Personal"));
        assert!(categories.contains(&"Meeting"));
        assert_eq!(categories.len(), 10);
    }
}
