pub fn get_visible_text(text: &str, max_height: usize) -> String {
    let mut line_breaks = text.char_indices().rev().filter_map(|(ix,c)| (c=='\n').then(|| ix));
    let first_line = line_breaks.nth(max_height as usize).map(|n| n + 1).unwrap_or(0);
    String::from(&text[first_line..])
}