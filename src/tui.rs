// tui.rs
use crate::app::App;
use crate::app::Popup;
use crate::textstyle;
use ratatui::{Frame, widgets::{Block, Borders, Paragraph, Tabs, RenderDirection, Sparkline}};
use ratatui::widgets::{List, Clear, Wrap, BorderType, ListItem};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::layout::Flex;
use ratatui::prelude::{Rect, Layout, Position, Alignment}; 
use ratatui::prelude::Constraint::{Percentage, Fill, Min, Length};
use ratatui::symbols::bar::Set;
use strum_macros::{FromRepr, EnumIter, Display};
use strum::IntoEnumIterator;
use unicode_width::UnicodeWidthStr;

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = " F1-Help ")]
    Tab1,
    #[strum(to_string = " F2-Users ")]
    Tab2,
    #[strum(to_string = " F3-Overview ")]
    Tab3,
}

impl SelectedTab {
    /// Return tab's name as a styled `Line`
    fn title(self, r: u8, g: u8, b: u8) -> Line<'static> {
        Span::from(format!("{self}"))
            .style(Style::new().fg(Color::Rgb(r, g, b)))
            .into()
    }
}

struct Colors {
    bg: (u8,u8,u8),
    fg: (u8,u8,u8),
    txt: (u8,u8,u8),
    highlight: (u8,u8,u8),
    notif: (u8,u8,u8),
}

const CUSTOM_SET_SPARK: ratatui::symbols::bar::Set = Set {
        full: "⠸",
        seven_eighths: "⠸",
        three_quarters: "⠸",
        five_eighths: "⠘",
        half: "⠘",
        three_eighths: "⠘",
        one_quarter: "⠈",
        one_eighth: "⠈",
        empty: "⠈",
};

pub fn draw(frame: &mut Frame, app: &mut App) {

    let color_map = get_colors(app);

    let area = frame.area();
    let block = Block::default().style(Style::default().bg(Color::Rgb(color_map.bg. 0,color_map.bg.1, color_map.bg.2))).borders(Borders::NONE);
    frame.render_widget(block, area);

    let threshold = app.w as usize - 12 - UnicodeWidthStr::width(app.active_channel.as_str()) - UnicodeWidthStr::width(app.active_nick.as_str()) - UnicodeWidthStr::width(app.active_server.as_str());
    let binding = app.prompt.clone();
    let (visible_prompt, cursor_offset_x) = textstyle::visible_prompt_and_cursor_offset(&binding, threshold, app);
    let input_title: Vec<Span> = itertools::Itertools::intersperse(app.input_mode.clone().into_iter(), Span::from("|"),).collect();
    let input = Paragraph::new(Line::from(visible_prompt)).block(Block::bordered().title(Line::from(input_title.clone()).right_aligned()).border_type(BorderType::default()).border_style(Style::new().fg(Color::Rgb(color_map.fg. 0,color_map.fg.1, color_map.fg.2))).borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM));

    let vertical_layout = Layout::vertical([Length(1), Min(0), Length(3)]);
    let [info_bar, stream_area, input_horizontal_area] = vertical_layout.areas(frame.area()); 

    let horizontal_info_layout = Layout::horizontal([Fill(1), Length(38), Fill(1)]);
    let [ spark_area, tab_area, spark2_area] = horizontal_info_layout.areas(info_bar);
    let horizontal_input_layout = Layout::horizontal([Length(10+app.active_channel.len().try_into().unwrap_or(0)+app.active_nick.len().try_into().unwrap_or(0)+app.active_server.len().try_into().unwrap_or(0)), Fill(1)]);
    let [nick_area, input_area] = horizontal_input_layout.areas(input_horizontal_area);

    let nick_layout = Paragraph::new(Span::from(format!("{} 🮥 {} 🮥 {} 🮥", app.active_server, app.active_channel,  app.active_nick.trim())).style(Style::new().fg(Color::Rgb(color_map.txt.0, color_map.txt.1, color_map.txt.2)))).block(Block::bordered().border_type(BorderType::default()).border_style(Style::new().fg(Color::Rgb(color_map.fg. 0,color_map.fg.1, color_map.fg.2))).borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM));
    let sparkline = Sparkline::default().bar_set(CUSTOM_SET_SPARK).data(&app.spark_data.clone()).style(ratatui::style::Style::default().fg(Color::Rgb(color_map.fg. 0,color_map.fg.1, color_map.fg.2)));
    let sparkline_rev = Sparkline::default().direction(RenderDirection::RightToLeft).bar_set(CUSTOM_SET_SPARK).data(&app.spark_data.clone()).style(ratatui::style::Style::default().fg(Color::Rgb(color_map.fg. 0,color_map.fg.1, color_map.fg.2)));
    let tab_titles = SelectedTab::iter().map(|tab| tab.title(color_map.txt.0, color_map.txt.1, color_map.txt.2));
    let tabs = Tabs::new(tab_titles).select(app.active_tab).highlight_style(Style::new().fg(Color::Rgb(color_map.fg. 0,color_map.fg.1, color_map.fg.2)).add_modifier(Modifier::BOLD)).padding(" ", " ").divider("");

    frame.render_widget(input, input_area);
    frame.render_widget(sparkline, spark_area);
    frame.render_widget(sparkline_rev, spark2_area);
    frame.render_widget(tabs, tab_area);
    frame.render_widget(nick_layout, nick_area);

    if app.split.0 {
        render_split_main(frame, app, &color_map, stream_area);
    } else {
        render_main(frame, app, &color_map, stream_area);
    }

    draw_popup(frame, app, &color_map);

    frame.set_cursor_position(Position::new(input_area.x + cursor_offset_x as u16, input_area.y + 1));
}

fn draw_popup(frame: &mut Frame, app: &mut App, colors: &Colors) {
    match app.popup {
        Popup::None => {}
        Popup::Color   => render_color_pop(frame, app, colors),
        Popup::List    => render_list_pop(frame, app, colors),
        Popup::Help    => render_help_pop(frame, colors),
        Popup::User    => render_user_pop(frame, app, colors),
        Popup::Channel => render_chan_pop(frame, app, colors),
    }
}

fn render_color_pop(frame: &mut Frame, app: &mut App, c: &Colors) {
    let color_fg_block = List::new(color_lines()).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2))).block(Block::bordered().style(Style::default().fg(Color::Rgb(c.fg.0, c.fg.1, c.fg.2)).bg(Color::Rgb(c.bg.0 - 10, c.bg.1 - 10, c.bg.2 - 10))).title(Line::from("Color Fg").centered())).highlight_symbol("🮥 ").highlight_style(Style::default().fg(Color::Rgb(c.highlight.0, c.highlight.1, c.highlight.2)));
    let color_fg_popup_area = color_pop_fgarea(frame.area(), 20, 40);
    frame.render_widget(Clear, color_fg_popup_area); //this clears out the background
    frame.render_stateful_widget(color_fg_block, color_fg_popup_area, &mut app.color_state_fg);
    let color_bg_block = List::new(color_lines()).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2))).block(Block::bordered().style(Style::default().fg(Color::Rgb(c.fg.0, c.fg.1, c.fg.2)).bg(Color::Rgb(c.bg.0 - 10, c.bg.1 - 10, c.bg.2 - 10))).title(Line::from("Color Bg").centered())).highlight_symbol("🮥 ").highlight_style(Style::default().fg(Color::Rgb(c.highlight.0, c.highlight.1, c.highlight.2)));
    let color_bg_popup_area = color_pop_bgarea(frame.area(), 20, 40);
    frame.render_widget(Clear, color_bg_popup_area); //this clears out the background
    frame.render_stateful_widget(color_bg_block, color_bg_popup_area, &mut app.color_state_bg);
}

fn render_list_pop(frame: &mut Frame, app: &mut App, c: &Colors) {
    let mut list_lines: Vec<Line> = Vec::new();
    let list_length = app.list_response.len();
    for line in app.list_response.clone() {
        list_lines.push(Line::from(line));
    }

    let list_block = if list_length > ((app.h as usize * 70) / 100) - 1 {
        let slice = &list_lines[app.list_pos..((app.h as usize * 70) / 100) - 1 + app.list_pos];
        Paragraph::new(slice.to_vec()).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2))).block(Block::bordered().style(Style::default().fg(Color::Rgb(c.fg.0, c.fg.1, c.fg.2)).bg(Color::Rgb(c.bg.0 - 10, c.bg.1 - 10, c.bg.2 - 10))).title(Line::from(list_length.to_string()).right_aligned()))
    } else {
        Paragraph::new(list_lines).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2))).block(Block::bordered().style(Style::default().fg(Color::Rgb(c.fg.0, c.fg.1, c.fg.2)).bg(Color::Rgb(c.bg.0 - 10, c.bg.1 - 10, c.bg.2 - 10))).title(Line::from(list_length.to_string()).right_aligned()))
    };

    let list_popup_area = pop_area(frame.area(), 80, 70);
    frame.render_widget(Clear, list_popup_area); //this clears out the background
    frame.render_widget(list_block, list_popup_area);
}

fn render_help_pop(frame: &mut Frame, c: &Colors) {
    let help_block = Paragraph::new(help_lines()).wrap(Wrap { trim: true }).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2))).block(Block::bordered().style(Style::default().fg(Color::Rgb(c.fg.0, c.fg.1, c.fg.2)).bg(Color::Rgb(c.bg.0 - 10, c.bg.1 - 10, c.bg.2 - 10))).title(Line::from("Help").centered()));
    let help_popup_area = pop_area(frame.area(), 60, 60);
    frame.render_widget(Clear, help_popup_area); //this clears out the background
    frame.render_widget(help_block, help_popup_area);
}

fn render_user_pop(frame: &mut Frame, app: &mut App, c: &Colors) {
    let mut user_lines: Vec<Line> = Vec::new();
    let mut user_length: usize = 0;
    if let Some(server) = app.server_list.get(&app.active_server) {
        if let Some(channel) = server.channels.get(&app.active_channel) {
            user_length = channel.user_list.len();
            for user in &channel.user_list {
                user_lines.push(Line::from(Span::from(user).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2)))));
            }
        }
    }

    let user_block = if user_length > ((app.h as usize * 70) / 100) - 1 {
        let slice = &user_lines[app.menu_pos..((app.h as usize * 70) / 100) - 1 + app.menu_pos];
        Paragraph::new(slice.to_vec()).block(Block::bordered().style(Style::default().fg(Color::Rgb(c.fg.0, c.fg.1, c.fg.2)).bg(Color::Rgb(c.bg.0 - 10, c.bg.1 - 10, c.bg.2 - 10))).title(Line::from(user_length.to_string()).right_aligned()).title(Line::from("Users").left_aligned()))
    } else {
        Paragraph::new(user_lines).block(Block::bordered().style(Style::default().fg(Color::Rgb(c.fg.0, c.fg.1, c.fg.2)).bg(Color::Rgb(c.bg.0 - 10, c.bg.1 - 10, c.bg.2 - 10))).title(Line::from(user_length.to_string()).right_aligned()).title(Line::from("Users").left_aligned()))
    };

    let user_popup_area = pop_area(frame.area(), 30, 60);
    frame.render_widget(Clear, user_popup_area); //this clears out the background
    frame.render_widget(user_block, user_popup_area);
}

fn render_chan_pop(frame: &mut Frame, app: &mut App, c: &Colors) {
    let mut channel_lines: Vec<Line> = Vec::new();
    let mut index = 0;
    let mut state_index = 0;

    for (outer_key, inner_map) in &app.server_list {
        channel_lines.push(Line::from(outer_key.to_owned()).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2))));
        state_index += 1;
        for (inner_key, data) in &inner_map.channels {
            if data.notification == true && inner_key != "Status" {
                channel_lines.push(Line::from(vec![Span::from(format!("[!] ")).style(Style::new().fg(Color::Rgb(c.notif.0, c.notif.1, c.notif.2))), Span::from(format!("{}: {}", index, inner_key)).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2)))]));
            } else {
                channel_lines.push(Line::from(format!("    {}: {}", index, inner_key)).style(Style::new().fg(Color::Rgb(c.txt.0, c.txt.1, c.txt.2))));
            }

            if *inner_key == app.active_channel && *outer_key == app.active_server {
                app.channel_state.select(Some(state_index));
            }

            index += 1;
            state_index += 1;
        }
    }

    let channel_block = List::new(channel_lines).highlight_symbol("🮥 ").highlight_style(Style::default().fg(Color::Rgb(c.highlight.0, c.highlight.1, c.highlight.2))).block(Block::bordered().style(Style::default().fg(Color::Rgb(c.fg.0, c.fg.1, c.fg.2)).bg(Color::Rgb(c.bg.0 - 10, c.bg.1 - 10, c.bg.2 - 10))).title(Line::from("Server/Channels").centered()));
    let channel_popup_area = pop_area(frame.area(), 30, 60);
    frame.render_widget(Clear, channel_popup_area); //this clears out the background
    frame.render_stateful_widget(channel_block, channel_popup_area, &mut app.channel_state);
}

fn render_main(frame: &mut Frame, app: &mut App, colors: &Colors, area: Rect) {
    let mut lines: Vec<Line> = textstyle::chat_style(app, app.active_server.clone(), app.active_channel.clone());

    if lines.len() > app.h as usize - 6 {
        if let Some(server) = app.server_list.get(&app.active_server) {
            if let Some(channel) = server.channels.get(&app.active_channel) {
                let chat_slice_start = lines.len().saturating_sub(app.h as usize - 6).saturating_sub(channel.chat_pos);
                lines = lines[chat_slice_start..].to_vec();
            }
        }
    }

    let message_layout = List::new(lines).block(Block::bordered().border_style(Style::new().fg(Color::Rgb(colors.fg. 0,colors.fg.1, colors.fg.2))));
    frame.render_widget(message_layout, area);
}

fn render_split_main(frame: &mut Frame, app: &mut App, colors: &Colors, area: Rect) {
    let (_, server_left, left, server_right, right) = app.split.clone();
    let split_chat = Layout::horizontal([Fill(1), Fill(1)]);
    let [split_left, split_right] = split_chat.areas(area);

    let mut lines_left: Vec<Line> = textstyle::chat_style(app, server_left.clone(), left.clone());

    if lines_left.len() > app.h as usize - 6 {
        if let Some(server) = app.server_list.get(&server_left) {
            if let Some(channel) = server.channels.get(&left) {
                let chat_slice_start = lines_left.len().saturating_sub(app.h as usize - 6).saturating_sub(channel.chat_pos);
                lines_left = lines_left[chat_slice_start..].to_vec();
            }
        }
    }

    if app.active_server == server_left && app.active_channel == left {
        let message_layout_left = List::new(lines_left).block(Block::bordered().title_top(left).border_style(Style::new().fg(Color::Rgb(colors.highlight. 0,colors.highlight.1, colors.highlight.2))));
        frame.render_widget(message_layout_left, split_left);
    } else {
        let message_layout_left = List::new(lines_left).block(Block::bordered().title_top(left).border_style(Style::new().fg(Color::Rgb(colors.fg. 0,colors.fg.1, colors.fg.2))));
        frame.render_widget(message_layout_left, split_left);
    }


    let mut lines_right: Vec<Line> = textstyle::chat_style(app, server_right.clone(), right.clone());

    if lines_right.len() > app.h as usize - 6 {
        if let Some(server) = app.server_list.get(&server_right) {
            if let Some(channel) = server.channels.get(&right) {
                let chat_slice_start = lines_right.len().saturating_sub(app.h as usize - 6).saturating_sub(channel.chat_pos);
                lines_right = lines_right[chat_slice_start..].to_vec();
            }
        }
    }

    if app.active_server == server_right && app.active_channel == right{
        let message_layout_right = List::new(lines_right).block(Block::bordered().title_top(right).title_alignment(Alignment::Right).border_style(Style::new().fg(Color::Rgb(colors.highlight. 0,colors.highlight.1, colors.highlight.2))));
        frame.render_widget(message_layout_right, split_right);
    } else {
        let message_layout_right = List::new(lines_right).block(Block::bordered().title_top(right).title_alignment(Alignment::Right).border_style(Style::new().fg(Color::Rgb(colors.fg. 0,colors.fg.1, colors.fg.2))));
        frame.render_widget(message_layout_right, split_right);
    }
}

fn get_colors(app: &App) -> Colors {
    Colors {
        bg: app.style_bg,
        fg: app.style_fg,
        txt: app.style_txt,
        highlight: app.style_highlight,
        notif: app.style_notif,
    }
}

fn color_pop_fgarea(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    // Step 1: vertically center a band
    let vertical = Layout::vertical([
        Percentage((100 - percent_y) / 2),
        Percentage(percent_y),
        Percentage((100 - percent_y) / 2),
    ]);
    let [_, band, _] = vertical.areas(area);

    // Step 2: horizontally center two blocks as a group
    let side_pct = (100 - (2 * percent_x)) / 2;
    let horizontal = Layout::horizontal([
        Percentage(side_pct),
        Percentage(percent_x),
        Percentage(percent_x),
        Percentage(side_pct),
    ]);
    let [_, fg, _, _] = horizontal.areas(band); // fg is the 2nd block
    fg
}

fn color_pop_bgarea(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    // Step 1: vertically center a band
    let vertical = Layout::vertical([
        Percentage((100 - percent_y) / 2),
        Percentage(percent_y),
        Percentage((100 - percent_y) / 2),
    ]);
    let [_, band, _] = vertical.areas(area);

    // Step 2: horizontally center two blocks as a group
    let side_pct = (100 - (2 * percent_x)) / 2;
    let horizontal = Layout::horizontal([
        Percentage(side_pct),
        Percentage(percent_x),
        Percentage(percent_x),
        Percentage(side_pct),
    ]);
    let [_, _, bg, _] = horizontal.areas(band); // bg is the 3rd block
    bg
}

fn pop_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect{
        let vertical = Layout::vertical([Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
}

fn color_lines() -> Vec<ListItem<'static>> {
    vec![
        ListItem::new("0 | 00 : White"),
        ListItem::new("1 | 01 : Black"),
        ListItem::new("2 | 02 : Blue"),
        ListItem::new("3 | 03 : Green"),
        ListItem::new("4 | 04 : Red"),
        ListItem::new("5 | 05 : LightRed"),
        ListItem::new("6 | 06 : Magenta"),
        ListItem::new("7 | 07 : LightYellow"),
        ListItem::new("8 | 08 : Yellow"),
        ListItem::new("9 | 09 : LightGreen"),
        ListItem::new("10 : Cyan"),
        ListItem::new("11 : LightCyan"),
        ListItem::new("12 : LightBlue"),
        ListItem::new("13 : LightMagenta"),
        ListItem::new("14 : DarkGray"),
        ListItem::new("15 : Gray"),
        ListItem::new("99 : None"),
    ]
}

fn help_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("/quit                    ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": To quit the application", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/connect ip              ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": To connect to a server", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/disconnect server       ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": To disconnect from a server", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/Join #Channel           ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": To join a chat channel", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/part #channel           ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": To leave a chat channel", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/'command'               ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": To use a command", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("message styling Bold     ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": ctrl + 'b'", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("message styling Italic   ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": ctrl + 's'", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("message styling Underline", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": ctrl + 'u'", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("message styling Reset    ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": ctrl + 'n'", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("message styling color    ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": ctrl + 'k' : then nr_fg ',' nr_bg", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("'Up' or 'Down            ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": Cycle prompt history", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("'PgUp' or 'PgDown'       ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": Cycle channel chat history", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/list                    ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": To list channels, Esc to close window", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/alis (Libera.Chat)      ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": To list channels, Esc to close window", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/twitch_connect          ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": Join twitch, use ouath file", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/swap 'number'           ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": Swap active channel", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("/split 'number'-'number' ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": Split screen view", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("Tab                      ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": Switch active channel in split view", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("Esc                      ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(": Close Popup", Style::default()),
        ]),

    ]
}
