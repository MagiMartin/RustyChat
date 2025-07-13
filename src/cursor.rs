use crate::app::App;

const IRC_FORMATTING_CODES: [char; 5] = ['\u{2}', '\u{3}', '\u{1D}', '\u{1F}', '\u{F}'];

pub struct PromptCursorMap {
    // Map: visible char index → byte index in the raw prompt string
    pub visible_to_raw: Vec<usize>,
    // Map: each byte index in raw string → visible char index (for non-formatting positions)
    //pub raw_to_visible: Vec<usize>,
}

pub fn build_prompt_cursor_map(prompt: &str) -> PromptCursorMap {
    let mut visible_to_raw = Vec::new();
    let mut raw_to_visible = vec![0; prompt.len() + 1];
    let mut chars = prompt.char_indices().peekable();
    let mut visible_char_count = 0;
    while let Some((raw_idx, c)) = chars.next() {
        if IRC_FORMATTING_CODES.contains(&c) {
            if c == '\u{3}' {
                // skip fg color code digits!
                if let Some(&(_, fg1)) = chars.peek() {
                    if fg1.is_digit(10) {
                        chars.next();
                        if let Some(&(_, fg2)) = chars.peek() {
                            if fg2.is_digit(10) {
                                chars.next();
                            }
                        }
                    }
                }
                // skip optional comma and bg color code digits
                if let Some(&(_, ',')) = chars.peek() {
                    chars.next();
                    if let Some(&(_, bg1)) = chars.peek() {
                        if bg1.is_digit(10) {
                            chars.next();
                            if let Some(&(_, bg2)) = chars.peek() {
                                if bg2.is_digit(10) {
                                    chars.next();
                                }
                            }
                        }
                    }
                }
            }
            continue;
        }
        visible_to_raw.push(raw_idx);
        for off in 0..c.len_utf8() {
            raw_to_visible[raw_idx + off] = visible_char_count;
        }
        visible_char_count += 1;
    }
    raw_to_visible[prompt.len()] = visible_char_count;
    PromptCursorMap { visible_to_raw }
}

pub fn byte_index(visible_cursor: usize, map: &PromptCursorMap, prompt: &str) -> usize {
    map.visible_to_raw.get(visible_cursor).cloned().unwrap_or(prompt.len())
}

pub fn clamp_cursor(new_cursor_pos: isize, map: &PromptCursorMap) -> usize {
    new_cursor_pos.clamp(0, map.visible_to_raw.len() as isize) as usize
}

pub fn enter_char(app: &mut App, new_char: char) {
    // Always rebuild the map after prompt changes, but you must build it first!
    let map = build_prompt_cursor_map(&app.prompt);
    let idx = byte_index(app.character_index, &map, &app.prompt);
    app.prompt.insert(idx, new_char);
    //REBUILD the mapping (prompt has changed)
    let map = build_prompt_cursor_map(&app.prompt);
    // Move cursor right (visible char)
    //move_cursor_right(app, &map);
    app.character_index = clamp_cursor(app.character_index as isize + 1, &map);
}

pub fn move_cursor_right(app: &mut App, map: &PromptCursorMap) {
    let cursor_moved_right = app.character_index as isize + 1;
    app.character_index = clamp_cursor(cursor_moved_right, map);
}

pub fn move_cursor_left(app: &mut App, map: &PromptCursorMap) {
    let cursor_moved_left = app.character_index as isize - 1;
    app.character_index = clamp_cursor(cursor_moved_left, map);
}

pub fn reset_cursor(app: &mut App) {
    app.character_index = 0;
}

pub fn delete_char(app: &mut App) {
    let map = build_prompt_cursor_map(&app.prompt);
    if app.character_index != 0 {
        // Get the raw string byte range to delete (from prev visible char to current)
        let idx = byte_index(app.character_index, &map, &app.prompt);
        let prev_idx = byte_index(app.character_index - 1, &map, &app.prompt);
        app.prompt.replace_range(prev_idx..idx, "");
        move_cursor_left(app, &map);

        // After deleting, if now at the beginning, also delete escape if present
        if app.character_index == 0 {
            let mut chars = app.prompt.char_indices().peekable();
            if let Some((i, c)) = chars.peek().copied() {
                if IRC_FORMATTING_CODES.contains(&c) {
                    let mut end = i + c.len_utf8();
                    chars.next(); // consume escape

                    // Handle color code escape \u{3}
                    if c == '\u{3}' {
                        // One or two fg digits
                        for _ in 0..2 {
                            if let Some(&(_, fg_digit)) = chars.peek() {
                                if fg_digit.is_digit(10) {
                                    end += fg_digit.len_utf8();
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        // Optionally a comma and one or two bg digits
                        if let Some(&(_, ',')) = chars.peek() {
                            end += 1;
                            chars.next();
                            for _ in 0..2 {
                                if let Some(&(_, bg_digit)) = chars.peek() {
                                    if bg_digit.is_digit(10) {
                                        end += bg_digit.len_utf8();
                                        chars.next();
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    // Now remove from 0 to end
                    app.prompt.replace_range(0..end, "");
                    // character_index already 0
                }
            }
        }
    }
    // else: already at 0, do nothing
}
