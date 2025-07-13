// app.rs
use crate::{tui, event, config, stream::StreamManager, stream::ServerId, stream::NetEvent};
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;
use ratatui::text::Span;
use crossterm::terminal::{self};
use tokio::sync::mpsc;
use std::io::Result;
use textwrap::{wrap, Options};
use regex::Regex;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

#[derive(Default)]
pub struct ChannelData {
    pub chat_list: Vec<(String, String)>,
    pub user_list: Vec<String>,
    pub chat_pos: usize,
    pub notification: bool,
}

type ChannelName = String;

#[derive(Default)]
pub struct ServerData {
    pub channels: BTreeMap<ChannelName, ChannelData>,
    pub nick: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Popup {
    #[default]
    None,
    Color,
    List,
    Help,
    User,
    Channel,
}

#[derive(Default)]
pub struct App {
    pub server_list: BTreeMap<ServerId, ServerData>,
    exit: bool,
    pub w: u16,
    pub h: u16,
    pub stream_mgr: StreamManager,
    pub prompt: String,
    pub active_server: String,
    pub active_channel: String,
    pub active_nick: String,
    pub real: String,
    pub spark_data: Vec<u64>,
    pub active_tab: usize,
    pub popup: Popup,
    pub color_state_fg: ListState,
    pub color_state_bg: ListState,
    pub channel_state: ListState,
    pub list_response: Vec<String>,
    pub character_index: usize,
    pub prompt_list: Vec<String>,
    pub prompt_pos: usize,
    pub list_pos: usize,
    pub menu_pos: usize,
    pub split: (bool, String, String, String, String),
    pub style_bg: (u8, u8, u8),
    pub style_fg: (u8, u8, u8),
    pub style_notif: (u8, u8, u8),
    pub style_highlight: (u8, u8, u8),
    pub style_txt: (u8, u8, u8),
    pub input_mode: Vec<Span<'static>>,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        
        let (input_tx, mut input_rx) = tokio::sync::mpsc::unbounded_channel();
        let (net_tx, mut net_rx) = mpsc::unbounded_channel::<(ServerId, NetEvent)>();

        self.active_nick = whoami::username();
        self.real = whoami::realname();
    
        self.input_mode.push(Span::from("N"));
        self.style_bg = (47, 50, 54);
        self.style_fg = (255, 238, 140);
        self.style_notif = (140, 255, 238);
        self.style_highlight = (238, 140, 255);
        self.style_txt = (255, 255, 255);

        let system_server = self.server_list
            .entry("System".to_string())
            .or_default();
        system_server.nick = self.active_nick.clone();
        system_server.channels
            .insert("Status".to_string(), ChannelData {
            chat_list: vec![
                (String::from("System"),"    __           _           ___ _           _   ".to_string()),
                (String::from("System"),"   /__\\_   _ ___| |_ _   _  / __\\ |__   __ _| |_ ".to_string()),
                (String::from("System"),"  / \\// | | / __| __| | | |/ /  | '_ \\ / _` | __|".to_string()), 
                (String::from("System")," / _  \\ |_| \\__ \\ |_| |_| / /___| | | | (_| | |_ ".to_string()), 
                (String::from("System")," \\/ \\_/\\__,_|___/\\__|\\__, \\____/|_| |_|\\__,_|\\__|".to_string()), 
                (String::from("System"),"                     |___/                       ".to_string())],
            user_list: Vec::new(),
            chat_pos: 0,
            notification: false,
        });
        system_server.nick = self.active_nick.clone();

        self.active_server = "System".to_string();
        self.active_channel = "Status".to_string();
        self.spark_data = [0,1,2,3,4,5,6,7,8,9,9,9,9,8,7,6,5,4,3,2,1,0].to_vec();
        let (nw, nh) = terminal::size().unwrap();
        self.w = nw;
        self.h = nh;
        self.active_tab = usize::MAX;

        config::read_theme(self);
        config::read_config(self);
        config::read_autojoin(self, &net_tx).await;

        // Spawn input handler
        tokio::spawn(event::input_event_loop(input_tx));

        // Main loop
        while !self.exit {
            let _ = terminal.draw(|frame| tui::draw(frame, self));

            tokio::select! {
                Some(cmd) = input_rx.recv() => {
                    event::handle_input(self, cmd, &net_tx).await;
                }
                Some((sid, netmsg)) = net_rx.recv() => {
                    self.handle_net_msg(sid, netmsg);
                }
                else => {
                    break;
                }
            }

        }
        Ok(())
    }

    pub fn handle_net_msg(&mut self, server_id: ServerId, event: NetEvent) {
        match event {
            NetEvent::Line(line) => {
                let bytes = line.clone().into_bytes();
                self.spark_data = bytes.iter().map(|&b| b as u64).collect();

                let mut value = line.split_whitespace();
                let prefix = value.next();
                let arg = value.next();
                let command_pos = line.find(arg.unwrap_or(""));
                let command = &line[command_pos.unwrap_or(0) as usize + arg.unwrap_or("").len()..];

                match prefix {
                    Some("PING") => {
                        let pong_line = line.to_string().replace("PING", "PONG");
                        self.stream_mgr.send_line(server_id.clone(), pong_line);
                    },
                    _ => {}
                }
                match arg {
                    Some("001") => {
                        //Welcome, parse autojoin channels
                        config::autojoin_channel(self, server_id);
                    }
                    Some("NOTICE") => {
                        if prefix.unwrap_or("").to_uppercase().starts_with(":ALIS!") {
                            let start_byte = line.find("#");
                            let result = &line[start_byte.unwrap_or(0)..];
                            self.list_pos = 0;
                            self.popup = Popup::List;
                            if result.to_uppercase().contains("RETURNING MAXIMUM OF") {
                                //Do not push
                            } else if result.to_uppercase().contains("MAXIMUM CHANNEL OUTPUT REACHED"){
                                //Do not push
                            } else {
                                self.list_response.push(result.to_string());
                            }
                        } else {
                            self.chat_bounds(line.to_string(), server_id.clone(), "Status".to_string(), arg.unwrap_or("").to_string());
                        }
                    }
                    Some("366") => {
                        //Hide Incomming Message
                    }
                    Some("322") => {
                        //HANDLE LIST COMMAND
                        self.list_pos = 0;
                        self.popup = Popup::List;
                        let start_byte = line.find('#');
                        let result = &line[start_byte.unwrap_or(0)..];
                        self.list_response.push(result.to_string());
                    }
                    Some("331") => {
                        //HANDLE no topic
                        let start_byte = command.find('#');
                        let end_byte = command.find(':');
                        self.chat_bounds(command[end_byte.unwrap_or(0)+1..].to_string(), server_id.clone(), command[start_byte.unwrap_or(0)..end_byte.unwrap_or(0)-1].to_owned(), arg.unwrap_or("").to_string());
                    }
                    Some("332") => {
                        //HANDLE topic
                        let start_byte = command.find('#');
                        let end_byte = command.find(':');
                        self.chat_bounds(command[end_byte.unwrap_or(0)+1..].to_string(), server_id.clone(), command[start_byte.unwrap_or(0)..end_byte.unwrap_or(0)-1].to_owned(), arg.unwrap_or("").to_string());
                    }
                    Some("433") => {
                        //HANDLE NickName in use
                        if let Some(server) = self.server_list.get_mut(&server_id.clone()) {
                            server.nick = server.nick.clone() + "_";
                            self.active_nick = server.nick.clone();
                            self.stream_mgr.send_line(server_id.clone(), "NICK ".to_owned() + &server.nick);
                        }
                        //self.nick = self.nick.clone() + "_";
                        //self.stream_mgr.send_line(server_id.clone(), "NICK ".to_owned() + &self.nick);
                        self.chat_bounds(prefix.unwrap_or("").to_string() + " " + &command[1..], server_id.clone(), "Status".to_string(), arg.unwrap_or("").to_string());
                    }
                    Some("QUIT") => {
                        //HANDLE QUIT
                        let end_bytes = line.find('!');
                        let result = &line[1..end_bytes.unwrap_or(0)];
                        let target = result.trim_start_matches(|c| c == '@' || c == '+');
                        
                        if let Some(server) = self.server_list.get_mut(&server_id.clone()) {
                            for (_channel_name, channel_data) in server.channels.iter_mut() {
                                channel_data.user_list.retain(|user| {
                                    let stripped_user = user.trim_start_matches(|c| c == '@' || c == '+');
                                    stripped_user != target
                                });

                            }
                        }
                        self.chat_bounds(prefix.unwrap_or("").to_string() + " " + &command[1..], server_id.clone(), "Status".to_string(), arg.unwrap_or("").to_string());
                    }
                    Some("PART") => {
                        //HANDLE PART
                        let end_bytes = line.find('!');
                        let result = &line[1..end_bytes.unwrap_or(1)];
                        //let start_chan = line.find('#');
                        let mut line_split = command.split_whitespace();
                        let chan = line_split.next();

                        if let Some(server) = self.server_list.get_mut(&server_id.clone()) {
                            if let Some(channel) = server.channels.get_mut(chan.unwrap_or("")) {
                                channel.user_list.retain(|u| {
                                let stripped = u.trim_start_matches(|c| c == '@' || c == '+');
                                stripped != result
                            });

                            }
                        }
                        self.chat_bounds(prefix.unwrap_or("").to_string() + " " + &command[1..], server_id.clone(), "Status".to_string(), arg.unwrap_or("").to_string());
                    }
                    Some("NICK") => {
                        //HANDLE NICK COMMAND
                        let end_byte = &line.find('!');
                        let start_byte = &line.find("NICK :");
                        let user_old = &line[1..end_byte.unwrap_or(0)];
                        let new_user = &line[(start_byte.unwrap_or(0) + 6)..];

                        if let Some(server) = self.server_list.get_mut(&server_id.clone()) {
                            if user_old == server.nick {
                                server.nick = new_user.to_string();
                                self.active_nick = new_user.to_string();
                                self.chat_bounds("You're now known as ".to_owned() + &self.active_nick, server_id.clone(), self.active_channel.clone(), arg.unwrap_or("").to_string());
                            }
                        }
                        if let Some(server) = self.server_list.get_mut(&server_id.clone()){
                            for (_channel_name, channel_data) in server.channels.iter_mut() {
                                for user in &mut channel_data.user_list {
                                    let stripped = user.trim_start_matches(|c| c == '@' || c == '+');
                                    if stripped == user_old {
                                        *user = new_user.to_string();
                                    }
                                }
                            }
                        }
                        self.chat_bounds(prefix.unwrap_or("").to_string() + " " + &command[1..], server_id.clone(), "Status".to_string(), arg.unwrap_or("").to_string());
                    }
                    Some("JOIN") => {
                        let end_bytes = line.find('!');
                        let result = &line[1..end_bytes.unwrap_or(1)];
                        let re = Regex::new(r"(#[^\s:]+)").unwrap();
                        let chan_re = re.captures(&line).unwrap();


                        if let Some(server) = self.server_list.get_mut(&server_id.clone()) {
                            if result == server.nick {
                                let server_channels = &mut self.server_list.entry(server_id.clone()).or_default().channels;
                                server_channels.entry(chan_re[1].to_owned()).or_insert(self::ChannelData { chat_list: vec![("System".to_string(),"Joining Channel".to_string())], user_list: vec![], chat_pos: 0, notification: false, });
                                let (on, left_server, left, right_server, right) = self.split.clone();
                                if on == true {
                                    if self.active_channel == left {
                                        self.split = (true, server_id.clone(), chan_re[1].to_string(), right_server, right);
                                        self.active_channel = chan_re[1].to_owned();
                                        self.active_server = server_id.clone();

                                    } else {
                                        self.split = (true, left_server, left, server_id.clone(), chan_re[1].to_string());
                                        self.active_channel = chan_re[1].to_owned();
                                        self.active_server = server_id.clone();
                                    }
                                } else {
                                    self.active_channel = chan_re[1].to_owned();
                                    self.active_server = server_id.clone();
                                }
                            }
                        }
                        if let Some(server) = self.server_list.get_mut(&server_id.clone()) {
                            if let Some(channel) = server.channels.get_mut(&chan_re[1].to_owned()) {
                                if result != server.nick {
                                    if !channel.user_list.contains(&result.to_string()) {
                                        channel.user_list.push(result.to_string());
                                    }
                                }
                            }
                        }
                        self.chat_bounds(prefix.unwrap_or("").to_string() + " " + &command[1..], server_id.clone(), "Status".to_string(), arg.unwrap_or("").to_string());
                    }
                    Some("353") => {
                        let start_bytes = command.find(':');
                        let result = &command[start_bytes.unwrap_or(0) + 1..];
                        let list_result = result.split_whitespace();
                        let re = Regex::new(r"(#[^\s:]+)").unwrap();
                        let chan_re = re.captures(&command).unwrap();
                        if let Some(server) = self.server_list.get_mut(&server_id.clone()) {
                            if let Some(channel) = server.channels.get_mut(&chan_re[1].to_owned()) {
                                for i in list_result {
                                    if !channel.user_list.contains(&i.to_string()) {
                                        channel.user_list.push(i.to_string());
                                    }
                                }
                            }
                        }
                    }
                    Some("PRIVMSG") => {
                        let end_bytes = line.find('!');
                        let res_nick = &line[1..end_bytes.unwrap_or(0)];
                        let end_mess = line.find(&(" :"));
                        let msg_start = line.find(&("PRIVMSG "));
                        let msg_chan = &line[msg_start.unwrap_or(0) + 8..end_mess.unwrap_or(0)];
                        let msg = line[(end_mess.unwrap_or(0) + 2)..].to_string();
                        if msg_chan.starts_with('#') {
                            self.chat_bounds(msg.clone(), server_id.clone(), msg_chan.to_string(), res_nick.to_string());
                        } else {
                            let msg = msg_chan.to_string() + "-> " + &line[(end_mess.unwrap_or(0) + 2)..];
                            self.chat_bounds(msg.clone(), server_id.clone(), res_nick.to_string(), res_nick.to_string());
                            if self.active_channel != res_nick {
                                self.chat_bounds(msg.clone(), server_id.clone(), self.active_channel.clone(), res_nick.to_string());
                            }
                        }
                    }
                    _ => {
                        if !line.is_empty() && line.starts_with(':') {
                            let command_trim: String = command[1..].chars().filter(|c| !c.is_control()).collect();
                            self.chat_bounds(command_trim, server_id.clone(), "Status".to_string(), arg.unwrap_or("").to_string());
                        }
                    }
                } 
            } 
            NetEvent::Error(e)   => {
                self.chat_bounds(e.to_string(), self.active_server.clone(), self.active_channel.clone(), "Error".to_string());
            }
        }
    }

    pub fn quit(&mut self) {
        self.stream_mgr.disconnect_all();
        self.exit = true;
    }

    pub fn chat_bounds(&mut self, data: String, server_id: String, channel_id: String, nick: String) {

        //Limit length of list
        if let Some(server) = self.server_list.get_mut(&server_id) {
            if let Some(channel) = server.channels.get_mut(&channel_id) {
                if channel.chat_list.len() > 1000 {
                channel.chat_list = channel.chat_list.split_off(500);
                }
            }
        }

        
        if let Some(server) = self.server_list.get_mut(&server_id) {
            match server.channels.entry(channel_id.clone()) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().chat_list.push((nick, data.clone()));
                    let (on, _left_server, left_chan, _right_server, right_chan) = self.split.clone();
                    //if self.active_server == server_id && self.active_channel == channel_id {
                        if let Some(server) = self.server_list.get_mut(&server_id) {
                            if let Some(channel) = server.channels.get_mut(&channel_id) {
                                 
                                let wrap_width;
                                //chat window horizontal "linewrap"
                                //let (on, _, _, _, _) = self.split;
                                if on == true {
                                    wrap_width = (self.w as usize / 2) - 6 - 12;
                                    if channel_id != left_chan {
                                        if channel_id != right_chan {
                                            channel.notification = true;
                                        }
                                    }
                                } else {
                                    wrap_width = self.w as usize-4 - 12;
                                    if self.active_channel != channel_id {
                                        channel.notification = true;
                                    }
                                }
                                let wrap_options = Options::new(wrap_width).break_words(false);
                                let wrapped_line = wrap(&data, wrap_options);
                                for _line in wrapped_line {
                                    if channel.chat_pos > 0 {
                                        channel.chat_pos = channel.chat_pos.saturating_add(1);
                                    }
                                }
                            };
                        };
                    //}
                }

                Entry::Vacant(entry) => {
                    entry.insert(self::ChannelData {
                        chat_list: vec![(nick, data)],
                        user_list: vec![],
                        chat_pos: 0,
                        notification: false,
                    });
                }
            }
        }
    } 
}
