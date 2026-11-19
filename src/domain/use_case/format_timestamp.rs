pub fn time_from_timestamp(timestamp: &str) -> Option<String> {
    let time = timestamp.split_once('T')?.1;
    let mut chars = time.chars();
    let hours = chars.by_ref().take(2).collect::<String>();
    let separator = chars.next()?;
    let minutes = chars.take(2).collect::<String>();

    if hours.len() == 2 && separator == ':' && minutes.len() == 2 {
        Some(format!("{hours}:{minutes}"))
    } else {
        None
    }
}
