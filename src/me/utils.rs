pub fn extract_allocation_amount(response_text: &str) -> Option<u64> {
    let parts: Vec<&str> = response_text.splitn(2, "2:").collect();

    if parts.len() < 2 {
        return None;
    }

    let json_str = parts[1].trim();

    let key = "\"allocationAmount\":";
    if let Some(start_pos) = json_str.find(key) {
        let number_start = start_pos + key.len();
        let remainder = &json_str[number_start..];

        if let Some(end_pos) = remainder.find(|c: char| !c.is_ascii_digit()) {
            let amount_str = remainder[..end_pos].trim();
            let amount: u64 = amount_str.parse().unwrap();
            return Some(amount);
        }
    }

    None
}
