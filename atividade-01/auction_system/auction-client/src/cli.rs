use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io::{self, stdout, Write}};

use tokio::sync::mpsc::{Sender, Receiver};

use crate::models::{Bid, CliCommand, Destructured};


pub struct Cli {
    command_input: String,
    messages: Vec<String>,
    scroll_offset: usize,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            command_input: String::new(),
            messages: Vec::new(),
            scroll_offset: 0,
        }
    }
    
    pub async fn run(
        mut self,
        make_bid_tx: Sender<Bid>,
        subscribe_tx: Sender<String>,
        mut cli_print_rx: Receiver<String>
    ) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        loop {

            // Handle incoming messages from other tasks
            while let Ok(message) = cli_print_rx.try_recv() {
                self.handle_notification(message);
            }
            
            self.draw_ui()?;
            
            // Check for user input or scheduler messages
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if self.handle_key_event(key_event, &make_bid_tx, &subscribe_tx).await {
                        break;
                    }
                }
            }
        }
        
        execute!(stdout, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
    
    fn draw_ui(&self) -> io::Result<()> {
        // Debugging: Print a message to indicate the UI is being drawn
        eprintln!("Drawing UI...");
        // Clear screen and set cursor position
        execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        
        // Draw messages
        let (rows, _cols) = crossterm::terminal::size()?;
        let message_area_height = rows - 2;
        
        for (i, message) in self.messages.iter().skip(self.scroll_offset).take(message_area_height as usize).enumerate(){
            execute!(
                stdout(),
                crossterm::cursor::MoveTo(0, (message_area_height - i as u16) as u16),
                crossterm::style::Print(message),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
            )?;
        }
        
        // Draw input line
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(0, rows - 1),
            crossterm::style::Print("> "),
            crossterm::style::Print(&self.command_input),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
        )?;
        
        stdout().flush()?;
        Ok(())
    }
    
    async fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        make_bid_tx: &Sender<Bid>,
        subscribe_tx: &Sender<String>,
    ) -> bool {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => return true, // Exit
            
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                let command = self.command_input.trim().to_string();
                self.command_input.clear();
                
                if !command.is_empty() {
                    self.messages.push(format!("> {}\n", command));
                    
                    // Process command
                    match self.parse_command(command){
                        Ok(cmd)=>{
                            match cmd {
                                CliCommand::MakeBid {..} => self.send_make_bid(cmd, make_bid_tx).await,
                                CliCommand::Subscribe {..} => self.send_subscribe(cmd, subscribe_tx).await,
                            }
                        },
                        Err(e) => self.messages.push(format!("Error: {}\n", e)),
                    }
                }
            }
            
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                self.command_input.pop();
            }
            
            KeyEvent {
                code: KeyCode::Char(c),
                ..
            } => {
                self.command_input.push(c);
            }
            
            KeyEvent {
                code: KeyCode::Up,
                ..
            } => {
                self.scroll_offset = (self.scroll_offset + 1).min(self.messages.len());
            }
            
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            
            _ => {}
        }
        
        false
    }
    
    fn parse_command(
        &mut self,
        command_string: String,
    ) -> Result<CliCommand, String> {
        let command_string_lc = command_string.to_ascii_lowercase();
        let parts: Vec<&str> = command_string_lc
            .split_whitespace()
            .collect();
        
        match parts.as_slice() {
            ["subscribe", auction_id] => {
                Ok(CliCommand::Subscribe { 
                    auction_id: auction_id.to_string(), 
                })
            },
            ["bid", auction_id, value] => {
                let value = value.parse::<f64>()
                    .map_err(|_| "Invalid value: must be a number".to_string())?;
        
                Ok(CliCommand::MakeBid { 
                    auction_id: auction_id.to_string(), 
                    value, 
                })
            },
            _ => Err("Unknown command. Usage: subscribe <auction_id> | bid <auction_id> <value>".to_string()),
        }
    }
    
    fn handle_notification(&mut self, message: String) {
        self.messages.push(message);
    }

    async fn send_make_bid(&mut self, cmd: CliCommand, make_bid_tx: &Sender<Bid>){

        if let Some(destructured) = cmd.destructure() {
            match destructured {
                Destructured::MakeBid(auction_id, value) => {
                    let bid = Bid{
                        auction_id: auction_id.parse().unwrap_or(0), // assuming auction_id is a numeric string
                        client_id: 0,
                        value: value, // already f64, no need to parse
                        signature: "aaa".to_string(),
                        public_key: "aaa".to_string(),
                        valid: false

                    };
                    make_bid_tx
                    .send(bid)
                    .await
                    .unwrap();
                }
                _ => { /* Ignore other commands */}
            }
        }
        

    }

    async fn send_subscribe(&mut self, cmd: CliCommand, subscribe_tx: &Sender<String>){
        if let Some(Destructured::Subscribe(auction_id)) = cmd.destructure() {
            subscribe_tx
                .send(auction_id)
                .await
                .unwrap();
        }

    }
}