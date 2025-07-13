// event.rs
use crate::app::App;
use crate::app::Popup;
use crate::config;
use crate::app::ServerData;
//use std::fs;
use crate::app::ChannelData;
//use crate::tui;
use ratatui::text::Span;
use crate::stream::{ServerId, NetEvent};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;
use regex::Regex;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use textwrap::{wrap, Options};
use crate::cursor;

pub enum AppEvent {
    InputEdit(char),
    InputSend,
    InputBackspace,
    Resize(u16, u16),
    Popup(KeyCode),
    KeyLeft,
    KeyRight,
    PromptHistory(KeyCode),
    InputDelete,
    InputEscape,
    ListHistory(KeyCode),
    SplitSwap,
    StyleSwitch(char),
}

static CTRL_KEYS: &[char] = &['s', 'b', 'k', 'u', 'n'];

pub async fn input_event_loop(sender: UnboundedSender<AppEvent>) {
    loop {
        if event::poll(std::time::Duration::from_millis(66)).unwrap() {
            match event::read().unwrap() {
                Event::Resize(nw, nh ) => {
                    if sender.send(AppEvent::Resize(nw, nh)).is_err() { break; }
                }
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) && CTRL_KEYS.contains(&c) {
                                if sender.send(AppEvent::StyleSwitch(c)).is_err() { break; }
                            } else {
                                if sender.send(AppEvent::InputEdit(c)).is_err() { break; }
                            }
                        }
                        KeyCode::Left => {
                           if sender.send(AppEvent::KeyLeft).is_err() { break; }
                        }
                        KeyCode::Right => {
                           if sender.send(AppEvent::KeyRight).is_err() { break; }
                        }
                        KeyCode::Backspace => {
                            if sender.send(AppEvent::InputBackspace).is_err() { break; }
                        }
                        KeyCode::Delete => {
                            if sender.send(AppEvent::InputDelete).is_err() { break; }
                        }
                        KeyCode::Enter => {
                            if sender.send(AppEvent::InputSend).is_err() { break; }
                        }
                        KeyCode::F(_) => {
                            if sender.send(AppEvent::Popup(key.code)).is_err() { break; }
                        }
                        KeyCode::Up => {
                            if sender.send(AppEvent::PromptHistory(key.code)).is_err() { break; }
                        }
                        KeyCode::Down => {
                            if sender.send(AppEvent::PromptHistory(key.code)).is_err() { break; }
                        }
                        KeyCode::Esc => {
                            if sender.send(AppEvent::InputEscape).is_err() { break; }
                        }
                        KeyCode::PageUp => {
                            if sender.send(AppEvent::ListHistory(key.code)).is_err() { break; }
                        }
                        KeyCode::PageDown => {
                            if sender.send(AppEvent::ListHistory(key.code)).is_err() { break; }
                        }
                        KeyCode::Tab => {
                            if sender.send(AppEvent::SplitSwap).is_err() { break; }
                        }
                        _ => {}
                    }
                }
            _ => {}
            }
        } else {
            if sender.is_closed() {
                break;
            }
        }
    }
}

// For main loop to use:
pub async fn handle_input(app: &mut App, ev: AppEvent, net_tx: &tokio::sync::mpsc::UnboundedSender<(ServerId, NetEvent)>) {
    match ev {
        AppEvent::InputEdit(c) => { 
            //app.prompt.push(c); 
            //tui::enter_char(c, app);
            cursor::enter_char(app, c);
        }
        AppEvent::KeyLeft => {
            //tui::move_cursor_left(app);
            let map = cursor::build_prompt_cursor_map(&app.prompt);
            cursor::move_cursor_left(app, &map);
        }
        AppEvent::KeyRight => {
            //tui::move_cursor_right(app);
            let map = cursor::build_prompt_cursor_map(&app.prompt);
            cursor::move_cursor_right(app, &map);
        }
        AppEvent::StyleSwitch(key) => {
            match key {
                's' => {
                    app.prompt.push('\u{1D}');
                    if app.input_mode.contains(&Span::from("N")) {
                        if let Some(idx) = app.input_mode.iter().position(|s| *s == Span::from("N")) {
                            app.input_mode.remove(idx);
                        }
                        //app.input_mode.remove(Span::from("N "));
                    }
                    if app.input_mode.contains(&Span::from("I")) {
                        if let Some(idx) = app.input_mode.iter().position(|s| *s == Span::from("I")) {
                            app.input_mode.remove(idx);
                        }
                        app.input_mode.push(Span::from("N"));
                        //app.input_mode.remove("I ");
                    } else {
                        app.input_mode.push(Span::from("I"));
                    }
                    //tui::move_cursor_right(app);
                    let map = cursor::build_prompt_cursor_map(&app.prompt);
                    cursor::move_cursor_right(app, &map);
                }
                'b' => {
                    app.prompt.push('\u{2}');
                    if app.input_mode.contains(&Span::from("N")) {
                        if let Some(idx) = app.input_mode.iter().position(|s| *s == Span::from("N")) {
                            app.input_mode.remove(idx);
                        }
                        //app.input_mode.remove(Span::from("N "));
                    }
                    if app.input_mode.contains(&Span::from("B")) {
                        if let Some(idx) = app.input_mode.iter().position(|s| *s == Span::from("B")) {
                            app.input_mode.remove(idx);
                        }
                        //app.input_mode.remove("I ");
                        app.input_mode.push(Span::from("N"));
                    } else {
                        app.input_mode.push(Span::from("B"));
                    }
                    //tui::move_cursor_right(app);
                    let map = cursor::build_prompt_cursor_map(&app.prompt);
                    cursor::move_cursor_right(app, &map);
                }
                'k' => {
                    app.prompt.push('\u{3}');
                    if app.input_mode.contains(&Span::from("N")) {
                        if let Some(idx) = app.input_mode.iter().position(|s| *s == Span::from("N")) {
                            app.input_mode.remove(idx);
                        }
                        //app.input_mode.remove(Span::from("N "));
                    }
                    if app.input_mode.contains(&Span::from("C")) {
                        //if let Some(idx) = app.input_mode.iter().position(|s| *s == Span::from("I")) {
                            //app.input_mode.remove(idx);
                        //}
                        //app.input_mode.remove("I ");
                    } else {
                        app.input_mode.push(Span::from("C"));
                    }
                    //tui::move_cursor_right(app);
                    let map = cursor::build_prompt_cursor_map(&app.prompt);
                    cursor::move_cursor_right(app, &map);
                    app.popup = Popup::Color;
                }
                'u' => {
                    app.prompt.push('\u{1F}');
                    if app.input_mode.contains(&Span::from("N")) {
                        if let Some(idx) = app.input_mode.iter().position(|s| *s == Span::from("N")) {
                            app.input_mode.remove(idx);
                        }
                        //app.input_mode.remove(Span::from("N "));
                    }
                    if app.input_mode.contains(&Span::from("U")) {
                        if let Some(idx) = app.input_mode.iter().position(|s| *s == Span::from("U")) {
                            app.input_mode.remove(idx);
                        }
                        app.input_mode.push(Span::from("N"));
                        //app.input_mode.remove("I ");
                    } else {
                        app.input_mode.push(Span::from("U"));
                    }
                    //tui::move_cursor_right(app);
                    let map = cursor::build_prompt_cursor_map(&app.prompt);
                    cursor::move_cursor_right(app, &map);
                }
                'n' => {
                    app.prompt.push('\u{F}');
                    app.input_mode.clear();
                    app.input_mode.push(Span::from("N"));
                    //tui::move_cursor_right(app);
                    let map = cursor::build_prompt_cursor_map(&app.prompt);
                    cursor::move_cursor_right(app, &map);
                    app.popup = Popup::None;
                }
                _ => {}
            }
        }
        AppEvent::SplitSwap => {
            let (toggle, server_left, left, server_right, right) = &app.split;
            if *toggle == true {
                if app.active_server == *server_left && app.active_channel == *left {
                    //app.chat_pos = 0;
                    app.active_server = server_right.to_string() ;
                    app.active_channel = right.to_string();
                    if let Some(server) = app.server_list.get(&server_right.clone()) {
                        app.active_nick = server.nick.clone();
                    }
                } else {
                   // app.chat_pos = 0;
                    app.active_server = server_left.to_string();
                    app.active_channel = left.to_string();
                    if let Some(server) = app.server_list.get(&server_left.clone()) {
                        app.active_nick = server.nick.clone();
                    }
                }
            }
        }
        AppEvent::PromptHistory(key) => {
            match key {
                KeyCode::Up => {
                    //Handle Up
                    if app.prompt_pos < 1 && app.prompt_list.len() < 1 {
                        //Stop Scroll
                    } else if app.prompt_pos < 1 && app.prompt_list.len() >= 1 {
                        app.prompt = app.prompt_list[0].clone();
                    } else {
                        app.prompt = app.prompt_list[app.prompt_pos - 1].clone();
                        app.prompt_pos = app.prompt_pos - 1;
                        let map = cursor::build_prompt_cursor_map(&app.prompt);
                        app.character_index = map.visible_to_raw.len();
                        //let cursor_moved_right = app.character_index.saturating_add(app.prompt.len());
                        //app.character_index = tui::clamp_cursor(cursor_moved_right, app);
                    }
                }
                KeyCode::Down => {
                    //Handle Down
                    if app.prompt_pos == app.prompt_list.len() {
                        //
                    } else if app.prompt_list.len() == app.prompt_pos +1 {
                        app.prompt.clear();
                        //tui::reset_cursor(app);
                        cursor::reset_cursor(app);
                        app.prompt_pos = app.prompt_pos + 1;
                    } else {
                        app.prompt = app.prompt_list[app.prompt_pos + 1].clone();
                        app.prompt_pos = app.prompt_pos + 1;
                        //let cursor_moved_right = app.character_index.saturating_add(app.prompt.len());
                        //app.character_index = tui::clamp_cursor(cursor_moved_right, app);
                        let map = cursor::build_prompt_cursor_map(&app.prompt);
                        app.character_index = map.visible_to_raw.len();
                    }
                }
                _ => {}
            }
        }
        AppEvent::ListHistory(key) => {
            match key {
                KeyCode::PageUp => {
                    //Handle PageUp
                    if app.popup == Popup::List {
                        app.list_pos = app.list_pos.saturating_sub(1);
                    } else if app.popup == Popup::User {
                        app.menu_pos = app.menu_pos.saturating_sub(1);
                    } else {
                        if let Some(server) = app.server_list.get_mut(&app.active_server) {
                            if let Some(channel) = server.channels.get_mut(&app.active_channel) {
                                
                                let raw_lines: Vec<&str> = channel.chat_list.iter().map(|(_, line)| line.as_str()).collect(); 
                                let wrap_width;
                                //chat window horizontal "linewrap"
                                let (on, _, _, _, _) = app.split;
                                if on == true {
                                    wrap_width = (app.w as usize / 2) - 6 - 12;
                                } else {
                                    wrap_width = app.w as usize-4 - 12;
                                }

                                let wrap_options = Options::new(wrap_width).break_words(false);
                                let wrapped_lines: Vec<String> = raw_lines.iter()
                                  .flat_map(|line| wrap(line, &wrap_options))
                                    .map(|cow| cow.into_owned())
                                    .collect();
                                
                                if channel.chat_pos == wrapped_lines.len().saturating_sub(app.h as usize - 6) {
                                    //Strop Scroll
                                } else {
                                    channel.chat_pos = channel.chat_pos.saturating_add(1);
                                }
                            };
                        };
                    };
                }
                KeyCode::PageDown => {
                    //Handle PageDown
                    if app.popup == Popup::List {
                        if app.list_pos >= app.list_response.len().saturating_sub(((app.h as usize * 70) / 100) - 1) {
                            //Stop Scrolling
                        } else {
                            app.list_pos = app.list_pos.saturating_add(1);
                        }
                    } else if app.popup == Popup::User {
                        //let mut user_length: usize = 0;
                        if let Some(server) = app.server_list.get(&app.active_server) {
                            if let Some(channel) = server.channels.get(&app.active_channel) {
                                let user_length = channel.user_list.len();

                                if app.menu_pos >= user_length.saturating_sub(((app.h as usize * 70) / 100) - 1) {
                                    //Stop Scrolloing
                                } else {
                                  app.menu_pos = app.menu_pos.saturating_add(1);
                                }
                            };
                        };
                    } else {
                        if let Some(server) = app.server_list.get_mut(&app.active_server) {
                            if let Some(channel) = server.channels.get_mut(&app.active_channel) {
                                channel.chat_pos = channel.chat_pos.saturating_sub(1);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        AppEvent::InputBackspace => { 
            //tui::delete_char(app);
            cursor::delete_char(app);
        }
        AppEvent::InputDelete => {
            if app.character_index == app.prompt.len() {
                } else if app.prompt.len() > 0 {
                    app.prompt.remove(app.character_index);
                }
        }
        AppEvent::InputEscape => {
            app.popup = Popup::None;
            app.list_response.clear();
            /*if app.list_popup == true {
                app.list_popup = false;
                app.list_response.clear();
            }*/
        }
        AppEvent::Resize(nw, nh) => { 
            app.w = nw;
            app.h = nh;
        }
        AppEvent::Popup(key) => {
            match key {
                KeyCode::F(1) => {
                    if app.popup == Popup::Help {
                        app.popup = Popup::None;
                        app.active_tab = usize::MAX;
                    } else {
                        app.popup = Popup::Help;
                        app.active_tab = 0;
                    }
                },
                KeyCode::F(2) => {
                    if app.popup == Popup::User {
                        app.popup = Popup::None;
                        app.active_tab = usize::MAX;
                    } else {
                        app.popup = Popup::User;
                        app.active_tab = 1;
                        app.menu_pos = 0;
                    }
                },
                KeyCode::F(3) => {
                    if app.popup == Popup::Channel {
                        app.popup = Popup::None;
                        app.active_tab = usize::MAX;
                    } else {
                        app.popup = Popup::Channel;
                        app.active_tab = 2;
                    }
                },
                _ => {},
            }

        }
        AppEvent::InputSend => {
            let line = app.prompt.clone();
            app.prompt_list.push(line.clone());
            app.prompt_pos = app.prompt_list.len();
            if !line.is_empty() {
                match &line {
                    s if s.to_uppercase().starts_with("/TWITCH_CONNECT") => {
                        //Handle Tiwtch Connection
                        let tw_serv = "irc.chat.twitch.tv";
                        let tw_port = ":6667";
                        //match fs::read_to_string("oauth") {
                        //    Ok(n) => {
                        let (tw_nick, oauth) = config::read_twitch();
                        if oauth == "Error" {
                            app.chat_bounds(tw_nick.to_owned(), app.active_server.clone(), app.active_channel.clone(), "ERROR".to_string());
                        } else {
                                //let mut oauth_iter = n.lines();
                                //let tw_nick = oauth_iter.next().unwrap_or(&app.active_nick.to_string()).to_string();
                                //let oauth = oauth_iter.next().unwrap_or("");
                                
                            if app.stream_mgr.connect(tw_serv.to_string().clone(), tw_serv.to_owned() + tw_port, net_tx.clone(), tw_nick.clone(), app.real.clone(), oauth.to_string()).await {
                                match app.server_list.entry(tw_serv.to_string().clone()) {
                                    Entry::Occupied(o) => o.into_mut(),
                                    Entry::Vacant(v) => {
                                    // Create a new HashMap with the "Status" channel already inserted
                                        let mut channels = BTreeMap::new();
                                        channels.insert("Status".to_string(), ChannelData::default());
                                        v.insert(ServerData {
                                            channels,
                                            nick: tw_nick.clone(),
                                        })
                                    }
                                };
                                app.active_server = tw_serv.to_string().clone();
                                app.active_channel = "Status".to_string();
                                app.active_nick = tw_nick.to_string();
                                if let Some(server) =  app.server_list.get_mut(&mut app.active_server) {
                                    if let Some(channel) = server.channels.get_mut(&app.active_channel) {
                                        channel.chat_list.push(("System".to_string(), String::from(format!("<connecting to {}>", tw_serv.to_owned() + tw_port))));
                                    }
                                }
                            }
                        }
                                //}
                                //Err(_) => {
                                //    app.chat_bounds("Error found!".to_owned(), app.active_server.clone(), app.active_channel.clone(), "ERROR".to_string());
                                //}
                            //}
                    }
                    s if s.to_uppercase().starts_with("/CONNECT") => {
                        let port = ":6667".to_string();
                        let mut server_id: String = "".to_string();
                        let mut addr:String = "".to_string();
                        let re = Regex::new(r"^/connect\s+([^\s:]+)(?::(\d+))?$").unwrap();
                        if re.is_match(&line) {
                            let conn = re.captures(&line).unwrap();
                            let serv_group = conn.get(1).map_or("", |m| m.as_str());
                            let port_group = conn.get(2).map_or("", |m| m.as_str());
                            server_id = serv_group.to_string();
                            if port_group.is_empty() {
                                addr = serv_group.to_string() + &port;
                            } else {
                                addr = serv_group.to_string() + ":" + &port_group;
                            }
                        }
 
                        if app.stream_mgr.connect(server_id.clone(), addr.to_string(), net_tx.clone(), app.active_nick.clone(), app.real.clone(), "".to_string()).await {
                            match app.server_list.entry(server_id.clone()) {
                                Entry::Occupied(o) => o.into_mut(),
                                Entry::Vacant(v) => {
                                // Create a new HashMap with the "Status" channel already inserted
                                    let mut channels = BTreeMap::new();
                                    channels.insert("Status".to_string(), ChannelData::default());
                                    v.insert(ServerData {
                                        channels,
                                        nick: app.active_nick.clone(),
                                    })
                                }
                            };
                            app.active_server = server_id.clone();
                            app.active_channel = "Status".to_string();
                            if let Some(server) =  app.server_list.get_mut(&mut app.active_server) {
                                if let Some(channel) = server.channels.get_mut(&app.active_channel) {
                                    channel.chat_list.push(("System".to_string(),String::from(format!("<connecting to {}>", addr))));
                                }
                            }
                        } 
                    }
                    s if s.to_uppercase().starts_with("/QUIT") => {
                        app.quit();
                    }
                    s if s.to_uppercase().starts_with("/DISCONNECT ") => {
                        let server_id = &line[12..];
                        if app.server_list.contains_key(server_id) {
                            app.stream_mgr.disconnect(server_id);
                            app.server_list.remove(server_id);
                            //app.chat_bounds(server_id.to_string(), "System".to_string(), "Status".to_string(), "test".to_string());
                            let (on, left_server, left_chan, right_server, right_chan) = app.split.clone();
                            if on {
                                if server_id == left_server {
                                    if app.active_server == left_server {
                                        app.split = (true, "System".to_string(), "Status".to_string(), right_server.clone(), right_chan.clone());
                                        app.active_server = "System".to_string();
                                        app.active_channel = "Status".to_string();
                                    } else {
                                        app.split = (true, "System".to_string(), "Status".to_string(), right_server.clone(), right_chan.clone());
                                    }
                                } else if server_id == right_server {
                                    if app.active_server == right_server {
                                        app.split = (true, left_server, left_chan, "System".to_string(), "Status".to_string());
                                        app.active_server = "System".to_string();
                                        app.active_channel = "Status".to_string();
                                    } else  {
                                        app.split = (true, left_server, left_chan, "System".to_string(), "Status".to_string());
                                    }
                                }
                            } else {
                                if app.active_server == server_id {
                                    app.active_server = "System".to_string();
                                    app.active_channel = "Status".to_string();
                                }
                            }
                        }                        
                    }
                    s if s.to_uppercase().starts_with("/JOIN #") => {
                        let prompt_command = line[1..line.len()].to_owned();
                        if app.active_server!= "System" {
                            app.stream_mgr.send_line(app.active_server.clone(), prompt_command);
                        } else {
                            app.chat_bounds("Error Not Connected to a server".to_owned(), "System".to_string(), "Status".to_string(), "ERROR".to_string());
                        }
                    }
                    s if s.to_uppercase().starts_with("/PART") => {
                        let prompt_write = line[1..].to_string();
                        if line.contains('#') {
                            app.stream_mgr.send_line(app.active_server.clone(), prompt_write);
                        }
                        if let Some(server) = app.server_list.get_mut(&app.active_server) {
                            if server.channels.contains_key(&line[6..]) {
                                server.channels.remove(&line[6..]);
                                
                                let (on, left_server, left_chan, right_server, right_chan) = app.split.clone();
                                if on {
                                    if app.active_channel == &line[6..] {
                                        app.split = (true, "System".to_string(), "Status".to_string(), right_server, right_chan);
                                        app.active_server = "System".to_string();
                                        app.active_channel = "Status".to_string();
                                    } else if app.active_channel == right_chan {
                                        app.split = (true, left_server, left_chan, "System".to_string(), "Status".to_string());
                                        app.active_server = "System".to_string();
                                        app.active_channel = "Status".to_string();
                                    }
                                } else {
                                    if app.active_channel == &line[6..] {
                                        app.active_server = "System".to_string();
                                        app.active_channel = "Status".to_string();
                                }
                            }
                                //app.active_server = "System".to_string();
                                //app.active_channel = "Status".to_string();
                            } else {
                                app.chat_bounds("Channel Not Joined".to_string(), app.active_server.clone(), app.active_channel.clone(), "Error".to_string())
                            }
                        }
                    }
                    s if s.to_uppercase().starts_with("/NICK") => {
                        if app.active_server != "System" {
                            let prompt_command = line[1..line.len()].to_owned();
                            app.stream_mgr.send_line(app.active_server.clone(), prompt_command);
                        } else {
                            app.active_nick = line[6..].to_string();
                            app.chat_bounds("You're now known as ".to_owned() + &app.active_nick, "System".to_string(), "Status".to_string(), "NICK".to_string())
                        }
                    }
                    s if s.to_uppercase().starts_with("/MSG") => {
                        if app.active_server != "System" {
                            let mut input = line.split_whitespace();
                            let _ = input.next();
                            let nick = input.next();
                            let msg_pos = line.find(nick.unwrap_or(""));
                            let mut msg = line[msg_pos.unwrap_or(0) as usize + nick.unwrap_or("").len()..].to_string();

                            while let Some(start) = msg.find("\\u{") {
                                if let Some(end) = msg[start..].find('}') {
                                    let end = start + end + 1;
                                    let unicode_escape = &msg[start..end];
                                    let hex_value = &unicode_escape[3..unicode_escape.len() - 1];

                                    if let Ok(code_point) = u32::from_str_radix(hex_value, 16) {
                                        if let Some(character) = char::from_u32(code_point) {
                                            msg = format!(
                                                "{}{}{}",
                                                &msg[..start],
                                                character,
                                                &msg[end..]
                                            );
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }

                            let prompt_write = "PRIVMSG ".to_owned() + &nick.unwrap_or("") + " :" + &msg;
                            app.stream_mgr.send_line(app.active_server.clone(), prompt_write);
                            app.chat_bounds(msg.clone(), app.active_server.clone(), app.active_channel.clone(), app.active_nick.clone());
                            app.chat_bounds(msg.clone(), app.active_server.clone(), nick.unwrap_or("").to_string(), app.active_nick.clone());
                        } else {
                            app.chat_bounds("Error Not Connected, or wrong server".to_owned(), "System".to_string(), "Status".to_string(), "Error".to_string())
                        }

                    }
                    s if s.to_uppercase().starts_with("/SWAP") => {
                        let swap_nr: &str = &s[6..];
                        if let Ok(nr) = swap_nr.parse::<usize>() {
                            let mut channel_lines: (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());

                            for (outer_key, inner_map) in &app.server_list {
                                //channel_lines.push(Line::from(outer_key.to_owned()));
                                for (inner_key, _data) in &inner_map.channels {
                                    channel_lines.1.push(format!("{}", inner_key));
                                    channel_lines.0.push(format!("{}", outer_key));
                                }

                            }
                            if nr < channel_lines.0.len() {
                                app.split = (false, String::new(),String::new(),String::new(),String::new());
                                app.active_server = channel_lines.0[nr].clone();
                                app.active_channel = channel_lines.1[nr].clone();
                                if let Some(server) = app.server_list.get_mut(&channel_lines.0[nr].clone()) {
                                    app.active_nick = server.nick.clone();
                                    if let Some(channel) = server.channels.get_mut(&channel_lines.1[nr].clone()) {
                                        channel.chat_pos = 0;
                                        channel.notification = false;
                                    }
                                }
                            }
                        }
                    }
                    s if s.to_uppercase().starts_with("/SPLIT") => {
                        let center_byte = &s.find('-');
                        let nr_left: &str = &s[7..center_byte.unwrap_or(7)];
                        let nr_right: &str = &s[center_byte.unwrap_or(0)+1..];
 
                        if let (Ok(left), Ok(right)) = (nr_left.parse::<usize>(), nr_right.parse::<usize>()) {
                            let mut channel_lines: (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());

                            for (outer_key, inner_map) in &app.server_list {
                                for (inner_key, _data) in &inner_map.channels {
                                    channel_lines.1.push(format!("{}",inner_key));
                                    channel_lines.0.push(format!("{}",outer_key));
                                }
                            }
                            if left < channel_lines.0.len() && right < channel_lines.0.len() {
                                app.split = (true, channel_lines.0[left].clone(), channel_lines.1[left].clone(), channel_lines.0[right].clone(), channel_lines.1[right].clone());
                                app.active_server = channel_lines.0[left].clone();
                                app.active_channel = channel_lines.1[left].clone();
                                if let Some(server) = app.server_list.get_mut(&channel_lines.0[left].clone()) {
                                    app.active_nick = server.nick.clone();
                                    if let Some(channel) = server.channels.get_mut(&channel_lines.1[left].clone()) {
                                        channel.chat_pos = 0;
                                        channel.notification = false;
                                    }
                                }
                                if let Some(server) = app.server_list.get_mut(&channel_lines.0[right].clone()) {
                                    if let Some(channel) = server.channels.get_mut(&channel_lines.1[right].clone()) {
                                        channel.chat_pos = 0;
                                        channel.notification = false;
                                    }
                                }

                            }
                        }
                    }
                    s if s.to_uppercase().starts_with("/") => {
                        let prompt_command = line[1..line.len()].to_owned();
                        app.stream_mgr.send_line(app.active_server.clone(), prompt_command);
                    }
                    _ => {
                        // process line, send, etc
                        if app.active_server != "System" {
                            let mut result = String::from(line);
                            while let Some(start) = result.find("\\u{") {
                                if let Some(end) = result[start..].find('}') {
                                    let end = start + end + 1;
                                    let unicode_escape = &result[start..end];
                                    let hex_value = &unicode_escape[3..unicode_escape.len() - 1];

                                    if let Ok(code_point) = u32::from_str_radix(hex_value, 16) {
                                        if let Some(character) = char::from_u32(code_point) {
                                            result = format!(
                                                "{}{}{}",
                                                &result[..start],
                                                character,
                                                &result[end..]
                                            );
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                            let prompt_write = "PRIVMSG ".to_owned() + &app.active_channel + " :" + &result;
                            app.stream_mgr.send_line(app.active_server.clone(), prompt_write);
                            app.chat_bounds(result.clone(), app.active_server.clone(), app.active_channel.clone(), app.active_nick.clone())
                        } else {
                            app.chat_bounds("Error currently not connected to a server or in a channel".to_owned(), "System".to_owned(), "Status".to_owned(), "Error".to_string());
                        }
                    }
                }
            }
            cursor::reset_cursor(app);
            app.prompt.clear();
            app.input_mode.clear();
            app.input_mode.push(Span::from("N"));
            app.popup = Popup::None;
        }
    }
}

