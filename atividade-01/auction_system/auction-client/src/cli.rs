
use std::error::Error;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout};

pub struct Cli{
    reader: BufReader<io::Stdin>,
    stdout: Stdout,
    buffer: String
}

impl Cli{
    pub fn new() -> Cli{
        let reader = BufReader::new(io::stdin());
        let stdout = io::stdout();
        let buffer = String::with_capacity(100);
        Cli{
            reader,
            stdout,
            buffer
        }
    }

    pub async fn read_async(&mut self, buffer: &mut String) -> Result<usize, Box<dyn Error>>{
        let written =  self.reader.read_line(buffer).await?;
        Ok(written)
    }

    pub async fn print(&mut self, str: &str){
        self.stdout.write_all(str.as_bytes()).await.unwrap();
    }
}