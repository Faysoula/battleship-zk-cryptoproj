use crate::network_protocol::GameMessage;
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

pub struct NetworkConnection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl NetworkConnection {
    /// Host: Create a server and wait for connection
    pub fn host(port: u16) -> Result<Self> {
        println!("Starting server on port {}...", port);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
            .context("Failed to bind to port")?;
        
        println!("   Waiting for opponent to connect...");
        println!("   Share this info with your opponent:");
        println!("   - Your IP address (use 'ip addr' or 'ipconfig')");
        println!("   - Port: {}", port);
        
        let (stream, addr) = listener.accept()?;
        println!("✓ Opponent connected from: {}", addr);
        
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Self { stream, reader })
    }

    /// Client: Connect to a host
    pub fn connect(host: &str, port: u16) -> Result<Self> {
        println!("🌐 Connecting to {}:{}...", host, port);
        let stream = TcpStream::connect(format!("{}:{}", host, port))
            .context("Failed to connect to host")?;
        
        println!("✓ Connected to opponent!");
        
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Self { stream, reader })
    }

    /// Send a message
    pub fn send(&mut self, message: &GameMessage) -> Result<()> {
        let json = serde_json::to_string(message)?;
        writeln!(self.stream, "{}", json)?;
        self.stream.flush()?;
        Ok(())
    }

    /// Receive a message (blocking)
    pub fn receive(&mut self) -> Result<GameMessage> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        let message = serde_json::from_str(&line)?;
        Ok(message)
    }
}