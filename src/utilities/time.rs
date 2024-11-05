pub fn humanize(mut seconds: u64) -> String {
    let hours = seconds / 3_600;
    seconds = seconds % 3_600;
    let minutes = seconds / 60;
    seconds = seconds % 60;

    let parts = vec![(hours, "h"), (minutes, "m"), (seconds, "s")];
    let duration: String = parts
        .into_iter()
        .filter_map(|(value, unit)| match unit {
            _ if value > 0 => Some(format!("{value}{unit}")),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join(" ");

    duration
}
