use std::time::Duration;

use ratatui::style::Color;

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

pub fn format_rate(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec.max(0.0) as u64))
}

pub fn format_frequency_mhz(mhz: u64) -> String {
    if mhz >= 1000 {
        format!("{:.2} GHz", mhz as f64 / 1000.0)
    } else {
        format!("{} MHz", mhz)
    }
}

pub fn format_duration(d: Duration) -> String {
    let total = d.as_secs();
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}

pub fn format_uptime(d: Duration) -> String {
    let total = d.as_secs();
    let days = total / 86400;
    let hours = (total % 86400) / 3600;
    let minutes = (total % 3600) / 60;
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn celsius_to_fahrenheit(c: f32) -> f32 {
    c * 9.0 / 5.0 + 32.0
}

pub fn format_temp(celsius: f32, fahrenheit: bool) -> String {
    if fahrenheit {
        format!("{:.0}°F", celsius_to_fahrenheit(celsius))
    } else {
        format!("{:.0}°C", celsius)
    }
}

pub fn usage_color(pct: f32) -> Color {
    if pct < 40.0 {
        Color::Green
    } else if pct < 70.0 {
        Color::Yellow
    } else if pct < 90.0 {
        Color::LightRed
    } else {
        Color::Red
    }
}

pub fn temp_color(c: f32) -> Color {
    if c < 60.0 {
        Color::Green
    } else if c < 75.0 {
        Color::Yellow
    } else if c < 85.0 {
        Color::LightRed
    } else {
        Color::Red
    }
}

pub fn battery_color(pct: f32) -> Color {
    if pct > 50.0 {
        Color::Green
    } else if pct >= 20.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

pub fn bar(value: f32, max: f32, width: usize) -> String {
    let ratio = (value / max).clamp(0.0, 1.0);
    let filled = (ratio * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    let mut s = String::with_capacity(width);
    for _ in 0..filled {
        s.push('█');
    }
    for _ in 0..empty {
        s.push('░');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }
    #[test]
    fn bytes_kb() {
        assert_eq!(format_bytes(1024), "1.0 KB");
    }
    #[test]
    fn bytes_gb() {
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    }
    #[test]
    fn frequency_low() {
        assert_eq!(format_frequency_mhz(800), "800 MHz");
    }
    #[test]
    fn frequency_ghz() {
        assert_eq!(format_frequency_mhz(4500), "4.50 GHz");
    }
    #[test]
    fn duration_hours() {
        assert_eq!(format_duration(Duration::from_secs(3661)), "1:01:01");
    }
    #[test]
    fn temp_color_thresholds() {
        assert_eq!(temp_color(59.0), Color::Green);
        assert_eq!(temp_color(60.0), Color::Yellow);
        assert_eq!(temp_color(76.0), Color::LightRed);
        assert_eq!(temp_color(86.0), Color::Red);
    }
    #[test]
    fn usage_color_thresholds() {
        assert_eq!(usage_color(0.0), Color::Green);
        assert_eq!(usage_color(50.0), Color::Yellow);
        assert_eq!(usage_color(95.0), Color::Red);
    }
}
