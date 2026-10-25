use crate::{
    data::{Message, User},
    client::join_server,
    server::create_server,
    server::lead_join,
    tui::ScreenStates::Main,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use local_ip_address::*;
use num::*;
use rand::{
    distr::{Distribution, SampleString},
    *,
};
use ratatui::{layout::*, style::Stylize, symbols::block, widgets::Paragraph};
use ratatui::{
    widgets::{Block, Borders},
    *,
};
use std::{
    option,
    path::absolute,
    str::FromStr,
    time::{Duration, Instant},
};
use whoami::*;
use x25519_dalek::*;
/*
       _______________________________________________
       |Username:_____        |           ip:_______
       |----------------------------------------------
       |
       |1. direct connection
       |2. groupe
       |                                     version
       -----------------------------------------------

       _______________________________________________
       |
       |
       |
       |
       |Message block
       |message block
       |---------------------------------------------
       | write messsage.
       -----------------------------------------------


       _______________________________________________
       |
       |
       |
       |                Waiting
       |                ?_ time | code:
       |
       |
       -----------------------------------------------

*/
#[derive(Default, PartialEq)]
pub enum ScreenStates {
    #[default]
    Main,
    Waiting,
    Messaging,
    Join,
}

#[derive(Default)]
pub struct App {
    should_quit: bool,
    screen: ScreenStates,
    messages: Vec<Message>,
    user: User,
    chat_option: i32,
    input: String,
    join_code: String,
    waiting_since: Option<Instant>,
    vm_status: String,
    join_ip: String,
    join_password: String,
    join_field: i32,
    join_status: String,
    join_result: Option<std::sync::mpsc::Receiver<std::result::Result<String, String>>>,
}

impl App {
    pub fn get_pc_name() -> String {
        let username = whoami::username();
        let name = username.unwrap();
        name
    }

    pub fn get_ip() {
        //to do
        return;
    }

    pub fn inst() -> Self {
        Self {
            should_quit: false,
            screen: Main,
            messages: Vec::new(),
            user: User::default(),
            chat_option: 0,
            input: String::new(),
            join_code: String::new(),
            waiting_since: None,
            vm_status: String::new(),
            join_ip: String::new(),
            join_password: String::new(),
            join_field: 0,
            join_status: String::new(),
            join_result: None,
        }
    }

    /*
     * border around the whole page, returns the area inside it
     */
    fn render_border(&self, frame: &mut Frame) -> Rect {
        frame.render_widget(
            Block::new().borders(Borders::ALL).title(" VPidgeon "),
            frame.area(),
        );

        let layout_border = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![Constraint::Percentage((100))])
            .split(frame.area());

        layout_border[0]
    }

    fn render_main(&self, frame: &mut Frame) {
        let area = self.render_border(frame);

        /*
         * split the area in header, body, footer
         */
        let header = Constraint::Length((4));
        let footer = Constraint::Length((3));
        let body = Constraint::Min((0));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![header, body, footer])
            .split(area);

        /*
         * split the header in username on the left, ip on the right
         */
        let layout_header = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(vec![Constraint::Min((0)), Constraint::Length((20))])
            .split(layout[0]);

        let layout_body = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length((1)),
                Constraint::Length((3)),
                Constraint::Length((3)),
                Constraint::Length((3)),
                Constraint::Min((0)),
            ])
            .split(layout[1]);

        let header_string = String::from(
            String::from("Welcome to VPidgeon. Choose your chatting option\nUsername: ")
                + self.user.name.as_str(),
        );

        frame.render_widget(Block::new().borders(Borders::ALL), layout[0]);

        frame.render_widget(Paragraph::new(header_string), layout_header[0]);

        frame.render_widget(
            Paragraph::new(String::from("ip: ") + self.user.private_ip.to_string().as_str())
                .alignment(Alignment::Right),
            layout_header[1],
        );

        let block1 = Paragraph::new("1. Direct chat").block(Block::new().borders(Borders::ALL));
        frame.render_widget(
            if self.chat_option == 0 {
                block1.block(Block::new().borders(Borders::ALL).red())
            } else {
                block1
            },
            layout_body[1],
        );
        let block2 = Paragraph::new("2. Group chat").block(Block::new().borders(Borders::ALL));
        frame.render_widget(
            if self.chat_option == 1 {
                block2.block(Block::new().borders(Borders::ALL).red())
            } else {
                block2
            },
            layout_body[2],
        );

        let block3 = Paragraph::new("3. Join").block(Block::new().borders(Borders::ALL));
        frame.render_widget(
            if self.chat_option == 2 {
                block3.block(Block::new().borders(Borders::ALL).red())
            } else {
                block3
            },
            layout_body[3],
        );

        frame.render_widget(
            Paragraph::new("Version 0.0.1")
                .block(Block::new().borders(Borders::ALL))
                .alignment(Alignment::Right),
            layout[2],
        );
    }

    /*
     * copy of the dummy .env with the generated code, it is uploaded to the vm
     */
    fn write_vm_env(&self) -> std::result::Result<(), String> {
        let template = match std::fs::read_to_string("vm_env.template") {
            Ok(text) => text,
            Err(err) => {
                return Err(
                    String::from("Couldnt read vm_env.template: ") + err.to_string().as_str()
                );
            }
        };
        let vm_env = template.replace("CHANGE_ME", self.join_code.as_str());
        match std::fs::create_dir_all("vm") {
            Ok(_) => {}
            Err(err) => {
                return Err(String::from("Couldnt create the vm folder: ") + err.to_string().as_str());
            }
        }
        match std::fs::write("vm/.env", vm_env) {
            Ok(_) => Ok(()),
            Err(err) => Err(String::from("Couldnt write vm/.env: ") + err.to_string().as_str()),
        }
    }

    pub fn render_single(&self, frame: &mut Frame) {}
    pub fn render_group(&self, frame: &mut Frame) {}

    pub fn render_wainting(&self, frame: &mut Frame) {
        let area = self.render_border(frame);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length((4)), Constraint::Min((0))])
            .split(area);

        /*
         * center the waiting text in the middle of the body
         */
        let layout_body = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min((0)),
                Constraint::Length((3)),
                Constraint::Min((0)),
            ])
            .split(layout[1]);

        frame.render_widget(
            Paragraph::new(
                String::from("VPidgeon - Terminal Messaging app\nUsername: ")
                    + self.user.name.as_str(),
            )
            .block(Block::new().borders(Borders::ALL)),
            layout[0],
        );

        let seconds = match self.waiting_since {
            Some(waiting_since) => waiting_since.elapsed().as_secs(),
            _ => 0,
        };
        let time_string = format!("{:02}:{:02}", seconds / 60, seconds % 60);

        frame.render_widget(
            Paragraph::new(
                String::from("Waiting to create conection....\n")
                    + time_string.as_str()
                    + " | Connection code: "
                    + self.join_code.as_str()
                    + "\n"
                    + self.vm_status.as_str(),
            )
            .alignment(Alignment::Center),
            layout_body[1],
        );
    }

    pub fn render_join(&self, frame: &mut Frame) {
        let area = self.render_border(frame);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length((4)),
                Constraint::Length((3)),
                Constraint::Length((3)),
                Constraint::Length((3)),
                Constraint::Min((0)),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new(
                String::from("Join a chat. Write the server ip and the password\nUsername: ")
                    + self.user.name.as_str(),
            )
            .block(Block::new().borders(Borders::ALL)),
            layout[0],
        );

        let ip_block = Paragraph::new(self.join_ip.as_str())
            .block(Block::new().borders(Borders::ALL).title(" Server ip "));
        frame.render_widget(
            if self.join_field == 0 {
                ip_block.block(Block::new().borders(Borders::ALL).title(" Server ip ").red())
            } else {
                ip_block
            },
            layout[1],
        );

        let password_block = Paragraph::new(self.join_password.as_str())
            .block(Block::new().borders(Borders::ALL).title(" Password "));
        frame.render_widget(
            if self.join_field == 1 {
                password_block.block(Block::new().borders(Borders::ALL).title(" Password ").red())
            } else {
                password_block
            },
            layout[2],
        );

        frame.render_widget(
            Paragraph::new(
                String::from(self.join_status.as_str())
                    + "\nTab: switch field | Enter: join | Esc: back",
            ),
            layout[3],
        );

        if self.join_field == 0 {
            frame.set_cursor_position((
                layout[1].x + 1 + self.join_ip.chars().count() as u16,
                layout[1].y + 1,
            ));
        } else {
            frame.set_cursor_position((
                layout[2].x + 1 + self.join_password.chars().count() as u16,
                layout[2].y + 1,
            ));
        }
    }

    pub fn render_messaging(&self, frame: &mut Frame) {
        let area = self.render_border(frame);

        /*
         * split the area in messages, input
         */
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min((0)), Constraint::Length((3))])
            .split(area);

        /*
         * show only the last messages that fit in the block
         */
        let visible = layout[0].height.saturating_sub(2) as usize;
        let skip = self.messages.len().saturating_sub(visible);

        let mut messages_string = String::new();
        for message in self.messages.iter().skip(skip) {
            messages_string.push_str(message.sender.name.as_str());
            messages_string.push_str(": ");
            messages_string.push_str(message.content.as_str());
            messages_string.push('\n');
        }

        frame.render_widget(
            Paragraph::new(messages_string)
                .block(Block::new().borders(Borders::ALL).title(" Messages ")),
            layout[0],
        );

        frame.render_widget(
            Paragraph::new(self.input.as_str())
                .block(Block::new().borders(Borders::ALL).title(" Write message ")),
            layout[1],
        );

        /*
         * put the cursor at the end of the text in the input block
         */
        frame.set_cursor_position((
            layout[1].x + 1 + self.input.chars().count() as u16,
            layout[1].y + 1,
        ));
    }

    pub fn run_main(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.render_main(frame))?;
            let key_event = event::read().unwrap().as_key_press_event();

            let key_code: KeyCode;
            match key_event {
                Some(key_event) => {
                    //print!("{:?}", key_event);
                    key_code = key_event.code;
                }
                _ => {
                    key_code = KeyCode::Null;
                    //print!("do nothing, no key recorded");
                }
            }

            //stop the app
            if key_code == KeyCode::Esc {
                self.should_quit = true;
                return Ok(());
            }
            match key_code {
                KeyCode::Down => {
                    self.chat_option = (self.chat_option + 1) % 3;
                    //print!("pressed down {} ", self.chat_option);
                }
                KeyCode::Up => {
                    //println!("before up {}\n", self.chat_option);
                    self.chat_option = (self.chat_option - 1 + 3) % 3;
                    //println!("pressed up {}\n", self.chat_option);
                }
                KeyCode::Enter => {
                    self.screen = ScreenStates::Waiting;
                    match self.chat_option {
                        0 => {
                            terminal.clear();
                            self.screen = ScreenStates::Waiting;
                            return Ok(());
                            //terminal.draw(|frame| self.render_single(frame))?;
                        }
                        1 => {
                            terminal.clear();
                            self.screen = ScreenStates::Waiting;
                            return Ok(());
                            //terminal.draw(|frame| self.render_group(frame))?;
                        }
                        2 => {
                            terminal.clear();
                            self.screen = ScreenStates::Join;
                            return Ok(());
                            //terminal.draw(|frame| self.render_wainting(frame))?;
                        }
                        _ => {
                            print!("")
                        }
                    }
                }
                _ => {}
            }
        }
    }
    pub fn run_waiting(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        self.join_code = rand::distr::Alphabetic.sample_string(&mut rand::rng(), 5);
        self.waiting_since = Some(Instant::now());
        //draw once before connecting, because create_server blocks until it ends
        self.vm_status = String::from("Starting the server on the VM...");
        terminal.draw(|frame| self.render_wainting(frame))?;
        //option 2 is Join, the joiner has no vm to start
        if self.chat_option == 2 {
            self.vm_status = String::new();
        } else {
            let started = match self.write_vm_env() {
                Ok(_) => create_server(),
                Err(message) => Err(message),
            };
            match started {
                Ok(message) => {
                    self.vm_status = message + ", waiting for the other peers...";
                    let (sender, receiver) = std::sync::mpsc::channel();
                    let code = self.join_code.clone();
                    std::thread::spawn(move || {
                        let result = lead_join(code.as_str());
                        let _ = sender.send(result);
                    });
                    self.join_result = Some(receiver);
                }
                Err(message) => {
                    self.vm_status = message;
                }
            }
        }
        loop {
            terminal.draw(|frame| self.render_wainting(frame))?;
            match self.poll_join() {
                Some(message) => {
                    self.vm_status = message;
                }
                None => {}
            }
            let key_code: KeyCode;
            //wait at most half a second for a key, so the timer keeps moving
            if event::poll(Duration::from_millis(500))? {
                let key_event = event::read().unwrap().as_key_press_event();
                match key_event {
                    Some(key_event) => {
                        //print!("RECORD EVENT HERE {:?}\n", key_event);
                        key_code = key_event.code;
                    }
                    _ => {
                        key_code = KeyCode::Null;
                        //print!("do nothing, no key recorded");
                    }
                }
            } else {
                key_code = KeyCode::Null;
            }
            match key_code {
                KeyCode::Esc => {
                    self.join_result = None;
                    self.screen = ScreenStates::Main;
                    return Ok(());
                }
                //no connection yet, Enter opens the messaging page to test it
                KeyCode::Enter => {
                    self.screen = ScreenStates::Messaging;
                    return Ok(());
                }
                _ => {
                    print!("");
                }
            }
        }
    }
    fn poll_join(&mut self) -> Option<String> {
        let mut answer = None;
        match &self.join_result {
            Some(receiver) => match receiver.try_recv() {
                Ok(result) => {
                    answer = Some(result);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    answer = Some(Err(String::from("The connection thread stopped")));
                }
                Err(_) => {}
            },
            None => {}
        }
        match answer {
            Some(result) => {
                self.join_result = None;
                match result {
                    Ok(message) => Some(message),
                    Err(message) => Some(message),
                }
            }
            None => None,
        }
    }
    pub fn run_join(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.render_join(frame))?;

            match self.poll_join() {
                Some(message) => {
                    self.join_status = message;
                }
                None => {}
            }

            let key_code: KeyCode;
            if event::poll(Duration::from_millis(500))? {
                let key_event = event::read().unwrap().as_key_press_event();
                match key_event {
                    Some(key_event) => {
                        key_code = key_event.code;
                    }
                    _ => {
                        key_code = KeyCode::Null;
                    }
                }
            } else {
                key_code = KeyCode::Null;
            }

            let joining = self.join_result.is_some();
            match key_code {
                KeyCode::Esc => {
                    self.join_result = None;
                    self.screen = ScreenStates::Main;
                    return Ok(());
                }
                KeyCode::Tab | KeyCode::Up | KeyCode::Down => {
                    self.join_field = (self.join_field + 1) % 2;
                }
                KeyCode::Char(character) => {
                    if !joining {
                        if self.join_field == 0 {
                            self.join_ip.push(character);
                        } else {
                            self.join_password.push(character);
                        }
                    }
                }
                KeyCode::Backspace => {
                    if !joining {
                        if self.join_field == 0 {
                            self.join_ip.pop();
                        } else {
                            self.join_password.pop();
                        }
                    }
                }
                KeyCode::Enter => {
                    if !joining {
                        if self.join_ip.is_empty() || self.join_password.is_empty() {
                            self.join_status = String::from("Fill the ip and the password");
                        } else {
                            let (sender, receiver) = std::sync::mpsc::channel();
                            let ip = self.join_ip.clone();
                            let password = self.join_password.clone();
                            std::thread::spawn(move || {
                                let result = join_server(ip.as_str(), password.as_str());
                                let _ = sender.send(result);
                            });
                            self.join_result = Some(receiver);
                            self.join_status =
                                String::from("Connected, waiting for the other peers...");
                        }
                    }
                }
                _ => {}
            }
        }
    }
    pub fn run_messaging(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.render_messaging(frame))?;
            let key_code: KeyCode;
            let key_event = event::read().unwrap().as_key_press_event();
            match key_event {
                Some(key_event) => {
                    key_code = key_event.code;
                }
                _ => {
                    key_code = KeyCode::Null;
                }
            }
            match key_code {
                KeyCode::Esc => {
                    self.screen = ScreenStates::Main;
                    return Ok(());
                }
                KeyCode::Char(character) => {
                    self.input.push(character);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    if !self.input.is_empty() {
                        self.messages.push(Message {
                            sender: self.user.clone(),
                            reply_to: None,
                            content: self.input.clone(),
                        });
                        self.input.clear();
                    }
                }
                _ => {
                    print!("");
                }
            }
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            if self.should_quit {
                break;
            }
            match self.screen {
                ScreenStates::Main => self.run_main(terminal)?,
                ScreenStates::Waiting => self.run_waiting(terminal)?,
                ScreenStates::Messaging => self.run_messaging(terminal)?,
                ScreenStates::Join => self.run_join(terminal)?,
            }
        }
        Ok(())
    }
}
