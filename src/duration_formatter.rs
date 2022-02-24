pub fn format_duration(duration_string: String) -> String {
    let duration_without_pt = duration_string[2..].to_string();

    match duration_without_pt.contains("H") {
        true => duration_without_pt.replace("H", ":").replace("M", ":").replace("S", ""),
        false => String::from("00:") + &duration_without_pt.replace("M", ":").replace("S", "")
    }
}