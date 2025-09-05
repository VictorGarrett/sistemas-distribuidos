use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io::{self, stdout, Write}};

use tokio::sync::mpsc::{Sender, Receiver};

use crate::models::{Auction, CliCommand};
struct Cli {
    command_input: String,
    messages: Vec<String>,
    scroll_offset: usize,
    auctions: Vec<Auction>
}

impl Cli {
    fn new(auctions: Option<Vec<Auction>>) -> Self {
        Self {
            command_input: String::new(),
            messages: Vec::new(),
            scroll_offset: 0,
            auctions: auctions.unwrap_or(Vec::new())
        }
    }
    
    async fn run(
        mut self,
        mut rx: Receiver<String>,
        tx: Sender<Auction>,
    ) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        loop {
            self.draw_ui()?;
            
            // Check for user input or scheduler messages
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if self.handle_key_event(key_event, &tx).await {
                        break;
                    }
                }
            }
            
            // Check for scheduler messages
            if let Ok(message) = rx.try_recv() {
                self.handle_notification(message);
            }
        }
        
        execute!(stdout, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
    
    fn draw_ui(&self) -> io::Result<()> {
        // Clear screen and set cursor position
        execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        
        // Draw messages
        let (rows, _cols) = crossterm::terminal::size()?;
        let message_area_height = rows - 2;
        
        for (i, message) in self.messages.iter().rev().skip(self.scroll_offset).take(message_area_height as usize).enumerate() {
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
        new_auction_tx: &Sender<Auction>,
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
                    self.messages.push(format!("> {}", command));
                    
                    // Process command
                    match self.parse_command(command){
                        Ok(cmd)=>{
                            self.send_new_auction(cmd, new_auction_tx).await;
                        },
                        Err(e) => self.messages.push(format!("Error: {}", e)),
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
            ["create", item, start_timestamp, end_timestamp] => {
                // Parse time and duration
                let start_timestamp: u64 = start_timestamp.parse()
                    .map_err(|_| "Invalid Timestamp format".to_string())?;
                
                let end_timestamp: u64 = end_timestamp.parse()
                    .map_err(|_| "Invalid duration".to_string())?;
                
                // Schedule the auction events
            
                Ok(
                    CliCommand::CreateAuction { 
                        item: item.to_string(), 
                        start_timestamp, 
                        end_timestamp
                    }
                )
            },
            ["list"] =>{
                Ok(CliCommand::ListAuctions)
            }
            _ => Err("Unknown command. Usage: create <time> <start_timestamp> <end_tim>".to_string()),
        }
    }
    
    fn handle_notification(&mut self, message: String) {
        self.messages.push(message);
    }

    async fn send_new_auction(&mut self, cmd: CliCommand, new_auction_tx: &Sender<Auction>){
        let new_auction = Auction::from_cmd(cmd, self.auctions.len() as u32).unwrap();
        self.auctions.push(new_auction.clone());
        
        new_auction_tx
            .send(new_auction)
            .await
            .unwrap();
    }
}