use crate::{
    data::{Message, User},
    tui::ScreenStates::Main,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use local_ip_address::*;
use rand::{
    distr::{Distribution, SampleString},
    *,
};
use ratatui::{layout::*, style::Stylize, symbols::block, widgets::Paragraph};
use ratatui::{
    widgets::{Block, Borders},
    *,
};
use std::{option, path::absolute, str::FromStr};
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
#[derive(Default)]
pub enum ScreenStates {
    #[default]
    Main,
    Waiting,
    Messaging,
}

#[derive(Default)]
pub struct App {
    should_quit: bool,
    screen: ScreenStates,
    messages: Vec<Message>,
    user: User,
    chat_option: i32,
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
            chat_option: 1,
        }
    }

    fn render_main(&self, frame: &mut Frame) {
        /*
         * split the area in header, body, footer
         */
        let header = Constraint::Percentage((30));
        let footer = Constraint::Percentage((20));
        let body = Constraint::Percentage((50));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![header, body, footer])
            .split(frame.area());

        let layout_body = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage((20)),
                Constraint::Percentage((20)),
                Constraint::Percentage((20)),
                Constraint::Percentage((20)),
                Constraint::Percentage((20)),
            ])
            .split(layout[1]);

        let header_string = String::from(
            String::from("Welcome to VPidgeon. Choose your chatting option\nUsername: ")
                + self.user.name.as_str(),
        );

        frame.render_widget(
            Paragraph::new(header_string).block(Block::new().borders(Borders::ALL)),
            layout[0],
        );

        frame.render_widget(
            Paragraph::new("VPidgeon \n\n").block(Block::new().borders(Borders::ALL)),
            layout[0],
        );

        let block1 = Paragraph::new("1. Direct chat").block(Block::new().borders(Borders::ALL));
        frame.render_widget(
            if self.chat_option == 0 {
                block1.block(Block::new().borders(Borders::ALL).red())
            } else {
                block1
            },
            layout_body[2],
        );
        let block2 = Paragraph::new("2. Group chat").block(Block::new().borders(Borders::ALL));
        frame.render_widget(
            if self.chat_option == 1 {
                block2.block(Block::new().borders(Borders::ALL).red())
            } else {
                block2
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

    pub fn render_single(&self, frame: &mut Frame) {}
    pub fn render_group(&self, frame: &mut Frame) {}
    pub fn render_wainting(&self, frame: &mut Frame) {
        let join_code: String = rand::distr::Alphabetic.sample_string(&mut rand::rng(), 5);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage((10)),
                Constraint::Percentage((55)),
                Constraint::Percentage((35)),
            ])
            .split(frame.area());

        frame.render_widget(
            Paragraph::new(
                String::from("VPidgeon - Terminal Messaging app\nUsername: ")
                    + self.user.name.as_str(),
            )
            .block(Block::new().borders(Borders::ALL)),
            layout[0],
        );

        frame.render_widget(Paragraph::new("Waiting to create conection...."), layout[1]);

        frame.render_widget(
            Paragraph::new(String::from("Connection code:") + join_code.as_str()),
            layout[1],
        );
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
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
                return Ok(());
            }
            match key_code {
                KeyCode::Down => {
                    self.chat_option = (1 - self.chat_option) % 3;
                    //print!("pressed down {} ", self.chat_option);
                }
                KeyCode::Up => {
                    self.chat_option = (1 + self.chat_option) % 3;
                    //print!("pressed up {}\n", self.chat_option);
                }
                KeyCode::Enter => {
                    print!("to do the show next interface");
                    match self.chat_option {
                        0 => {
                            terminal.draw(|frame| self.render_single(frame))?;
                        }
                        1 => {
                            terminal.draw(|frame| self.render_group(frame))?;
                        }
                        2 => {
                            terminal.draw(|frame| self.render_wainting(frame))?;
                        }
                        _ => {
                            print!("")
                        }
                    }
                }
                _ => print!("nothing relevant pressed"),
            }
        }
    }
}
