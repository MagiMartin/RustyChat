use ratatui::text::Span;
use unicode_width::UnicodeWidthChar;
use ratatui::style::Style;
use ratatui::style::Color;
use ratatui::style::Modifier;
use crate::app::App;
use ratatui::text::Line;
use textwrap::{wrap, Options};
use std::borrow::Cow;
use crate::cursor;


#[derive(Clone)]
struct StyleState {
    bold: bool,
    italic: bool,
    underline: bool,
}
impl StyleState {
    pub fn as_style(&self, base: Style) -> Style {
        let mut style = base;
        if self.bold        { style = style.add_modifier(Modifier::BOLD); }
        if self.italic      { style = style.add_modifier(Modifier::ITALIC); }
        if self.underline   { style = style.add_modifier(Modifier::UNDERLINED); }
        style
    }
}
impl Default for StyleState {
    fn default() -> Self {
        Self { bold:false, italic:false, underline:false }
    }
}


fn parse_color_code(app: &App, code: &str) -> Option<Color> {
    match code {
        "0" | "00" => Some(Color::White),
        "1" | "01" => Some(Color::Black),
        "2" | "02" => Some(Color::Blue),
        "3" | "03" => Some(Color::Green),
        "4" | "04" => Some(Color::Red),
        "5" | "05" => Some(Color::LightRed),
        "6" | "06" => Some(Color::Magenta),
        "7" | "07" => Some(Color::LightYellow),
        "8" | "08" => Some(Color::Yellow),
        "9" | "09" => Some(Color::LightGreen),
        "10" => Some(Color::Cyan),
        "11" => Some(Color::LightCyan),
        "12" => Some(Color::LightBlue),
        "13" => Some(Color::LightMagenta),
        "14" => Some(Color::DarkGray),
        "15" => Some(Color::Gray),
        _ => {
            let (bgr, bgg, bgb) = app.style_bg;
            Some(Color::Rgb(bgr, bgg, bgb))
        },
    }
}

fn line_wrap(width: usize, data: &str) -> Vec<Cow<str>> {
    let options = Options::new(width).break_words(false);
    let data_wrap = wrap(data, options);
    return data_wrap;
}

fn text_style<'a, F>(line: &Cow<str>, mut spans: Vec<Span<'a>>, mut current_style: Style, app: &App, mut on_color_change: F) -> Vec<Span<'a>> where F: FnMut(&str, &str) {
    let mut text = String::new();
    let mut chars = line.chars().peekable();
    let mut styles = StyleState::default();
    let start_default = current_style;

    while let Some(c) = chars.next() {
        match c {
            '\u{1D}' => {
                if !text.is_empty() {
                    spans.push(Span::styled(text.clone(), styles.as_style(current_style)));
                    text.clear();
                }
                styles.italic = !styles.italic;
               }
            '\u{1F}' => {
                if !text.is_empty() {
                    spans.push(Span::styled(text.clone(), styles.as_style(current_style)));
                    text.clear();
                }
                styles.underline = !styles.underline;
                }
            '\u{2}' => {
                if !text.is_empty() {
                    spans.push(Span::styled(text.clone(), styles.as_style(current_style)));
                    text.clear();
                }
                styles.bold = !styles.bold;
              }
            '\u{3}' => {
                let mut fg_code = String::new();
                let mut bg_code = String::new();

                // Read one or two digits for the foreground color
                if let Some(fg1) = chars.peek() {
                    if fg1.is_digit(10) {
                        fg_code.push(chars.next().unwrap());

                    if let Some(&next_char) = chars.peek() {
                        if next_char.is_digit(10) {
                            fg_code.push(chars.next().unwrap());
                        }
                    }
                    } else {
                        fg_code.push('1');
                        fg_code.push('4');
                    }
                }

                // Read the comma
                if let Some(',') = chars.peek() {
                    chars.next();
                    // Read one or two digits for the background color
                    if let Some(bg1) = chars.next() {
                        bg_code.push(bg1);
                        if let Some(&next_char) = chars.peek() {
                            if next_char.is_digit(10) {
                                bg_code.push(chars.next().unwrap());
                            }
                        }
                    }
                }

                on_color_change(&fg_code, &bg_code);

                if !text.is_empty() {
                    spans.push(Span::styled(text.clone(), styles.as_style(current_style)));
                    text.clear();
                }

                if let Some(fg_color) = parse_color_code(app, &fg_code) {
                    current_style = current_style.fg(fg_color);
                }
                if let Some(bg_color) = parse_color_code(app, &bg_code) {
                    current_style = current_style.bg(bg_color);
                }
            }
            '\u{1}' => {
                //u{1} CTCP VERSION Request send NOTICE response
                //Consume the char
            }
            '\u{F}' => {
                if !text.is_empty() {
                    spans.push(Span::styled(text.clone(), styles.as_style(current_style)));
                    text.clear();
                }
                styles = StyleState::default();
                current_style = start_default;
                if let Some(bg_color) = parse_color_code(app, "99") {
                    current_style = current_style.bg(bg_color);
                }

            }
            _ => {
                text.push(c);
            }
        }
    }
    if !text.is_empty() {
        spans.push(Span::styled(text.to_owned(), styles.as_style(current_style)));
    }
    spans
}

pub fn chat_style(app: &App, server_id: String, channel_id: String) -> Vec<Line> {

    let mut chat_lines: Vec<Line> = Vec::new();

    if let Some(server) = app.server_list.get(&server_id) {
        if let Some(channel) = server.channels.get(&channel_id) {

            for (sender, line) in channel.chat_list.clone() {
                //let (sender, line) = lines;


                let nick_width = 10;

                // Truncate and pad nicknames
                let trimmed_nick = if sender.chars().count() > nick_width {
                    // Truncate to max_nick_width - 1 and add '…'
                    format!("{:.1$}…", sender, nick_width - 1)
                } else {
                    format!("{:<width$}", sender, width = nick_width)
                };
                let prefix = format!("{}: ", trimmed_nick);

                let wrap_width;
                //chat window horizontal "linewrap"
                let (on, _, _, _, _) = app.split;
                if on == true {
                    wrap_width = (app.w as usize / 2) - 6 - prefix.len();
                } else {
                    wrap_width = app.w as usize-4 - prefix.len();
                }

                let (tr, tg, tb) = app.style_txt;
                let current_style = Style::new().fg(Color::Rgb(tr, tg, tb));

                let data_wrap = line_wrap(wrap_width, &line);

                for (i, line) in data_wrap.iter().enumerate() {

                    let mut spans = Vec::new();
                    // Add the prefix span to the start of this line EG Nick with spacing
                    if i == 0 {
                        let (hr, hg, hb) = app.style_highlight;
                        spans.push(Span::styled(
                            prefix.clone(),
                            Style::default().fg(Color::Rgb(hr, hg, hb)).add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        // Add blank prefix of the same width for alignment
                        spans.push(Span::raw(" ".repeat(prefix.len())));
                    }

                    let spans = text_style(line, spans, current_style, app, |_,_| {});
                    chat_lines.push(Line::from(spans));
                }
            }
        }
    }

    return chat_lines;
}

pub fn visible_prompt_and_cursor_offset<'a>(prompt: &'a str, max_width: usize, app: &mut App) -> (Vec<Span<'a>>, usize) {
    // TRACK SELECTORS from parsed color codes
    let mut fg_selector = 0;
    let mut bg_selector = 0;

    let map = cursor::build_prompt_cursor_map(prompt);
    let visible_len = map.visible_to_raw.len();

    let visible_cursor_index = app.character_index.clamp(0, visible_len);

    let style = Style::new().fg(Color::Rgb(app.style_txt.0, app.style_txt.1, app.style_txt.2));
    let styled_spans = text_style(&Cow::Borrowed(prompt),Vec::new(),style,app, |fg_code: &str, bg_code: &str| {
            fg_selector = fg_code.parse::<usize>().unwrap_or(0);
            bg_selector = bg_code.parse::<usize>().unwrap_or(0);
        },
    );
    app.color_state_fg.select(Some(fg_selector));
    app.color_state_bg.select(Some(bg_selector));

    let mut stripped = String::new();
    for &raw_byte_idx in &map.visible_to_raw {
        if let Some(c) = prompt[raw_byte_idx..].chars().next() {
            stripped.push(c);
        }
    }

    let mut width = 0;
    let mut byte_start = stripped.len();
    for (i, c) in stripped.char_indices().rev() {
        let w = UnicodeWidthChar::width(c).unwrap_or(0);
        if width + w > max_width { break; }
        width += w;
        byte_start = i;
    }

    let visible_char_start = stripped[..byte_start].chars().count();

    let cursor_offset = if visible_cursor_index >= visible_char_start {
    stripped.chars()
        .skip(visible_char_start)
        .take(visible_cursor_index - visible_char_start)
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
        .sum()
    } else {
        0
    };


    let mut visible_spans = Vec::new();
    let mut total_char_pos = 0; // Tracks visible characters seen.
    let mut chars_needed = stripped[byte_start..].chars().count();
    

    for span in &styled_spans {
        let s = span.content.as_ref();
        let span_len = s.chars().count();

        if total_char_pos + span_len <= visible_char_start {
            // This span is entirely before visible window.
            total_char_pos += span_len;
            continue;
        }

        // Where to start in this span
        let start_in_span = if visible_char_start > total_char_pos {
            visible_char_start - total_char_pos
        } else {
            0
        };
        // How many to take in this span
        let n = chars_needed.min(span_len - start_in_span);

        // Extract substring by chars
        let span_substr = s.chars().skip(start_in_span).take(n).collect::<String>();
        if !span_substr.is_empty() {
            visible_spans.push(Span::styled(span_substr, span.style));
        }

        chars_needed -= n;
        total_char_pos += span_len;
        if chars_needed == 0 {
            break;
        }
    }

    (visible_spans, cursor_offset)
}

