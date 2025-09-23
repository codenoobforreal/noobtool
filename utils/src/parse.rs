pub fn parse_fraction(s: &str) -> Option<f32> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return None;
    }

    let numerator = parts[0].parse::<u32>().ok()?;
    let denominator = parts[1].parse::<u32>().ok()?;
    if denominator == 0 {
        return None;
    }

    Some(numerator as f32 / denominator as f32)
}
