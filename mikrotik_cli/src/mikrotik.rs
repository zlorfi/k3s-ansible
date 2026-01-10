use anyhow::{anyhow, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tracing::{debug, info};

/// MikroTik RouterOS API Client
///
/// Implements the RouterOS binary API protocol for communicating with MikroTik devices.
pub struct MikroTikClient {
    host: String,
    port: u16,
    user: String,
    password: String,
    timeout: u64,
    stream: Option<TcpStream>,
}

impl MikroTikClient {
    /// Create a new MikroTik client and establish connection
    pub async fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        timeout: u64,
    ) -> Result<Self> {
        let mut client = MikroTikClient {
            host: host.to_string(),
            port,
            user: user.to_string(),
            password: password.to_string(),
            timeout,
            stream: None,
        };

        // Connect and authenticate
        client.connect().await?;
        client.authenticate().await?;

        Ok(client)
    }

    /// Connect to the MikroTik device via TCP
    async fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        debug!("Connecting to {}", addr);

        let stream = TcpStream::connect(&addr)
            .map_err(|e| anyhow!("Failed to connect to {}: {}", addr, e))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(self.timeout)))
            .map_err(|e| anyhow!("Failed to set read timeout: {}", e))?;

        stream
            .set_write_timeout(Some(Duration::from_secs(self.timeout)))
            .map_err(|e| anyhow!("Failed to set write timeout: {}", e))?;

        self.stream = Some(stream);
        info!("Connected to {} successfully", self.host);
        Ok(())
    }

    /// Authenticate with the MikroTik device
    async fn authenticate(&mut self) -> Result<()> {
        debug!("Authenticating as user: {}", self.user);

        // Send login request
        let login_cmd = vec![
            "/login".to_string(),
            format!("=name={}", self.user),
            format!("=password={}", self.password),
        ];

        self.send_command(&login_cmd).await?;
        let response = self.read_response().await?;

        if response.contains("!done") {
            info!("Authentication successful");
            Ok(())
        } else {
            Err(anyhow!("Authentication failed: {}", response))
        }
    }

    /// Send a command to the MikroTik device
    async fn send_command(&mut self, command: &[String]) -> Result<()> {
        if self.stream.is_none() {
            return Err(anyhow!("Not connected to MikroTik device"));
        }

        let stream = self.stream.as_mut().unwrap();
        let mut data = Vec::new();

        // Encode command using RouterOS protocol
        for part in command {
            let len = part.len();
            let len_bytes = Self::encode_length(len);
            data.extend_from_slice(&len_bytes);
            data.extend_from_slice(part.as_bytes());
        }

        // Send null terminator
        data.push(0);

        debug!("Sending command: {:?}", command);
        stream
            .write_all(&data)
            .map_err(|e| anyhow!("Failed to send command: {}", e))?;

        Ok(())
    }

    /// Read response from MikroTik device
    async fn read_response(&mut self) -> Result<String> {
        if self.stream.is_none() {
            return Err(anyhow!("Not connected to MikroTik device"));
        }

        let stream = self.stream.as_mut().unwrap();
        let mut response = String::new();
        let mut buffer = [0u8; 4096];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    if response.contains("!done") || response.contains("!trap") {
                        break;
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        break;
                    }
                    return Err(anyhow!("Failed to read response: {}", e));
                }
            }
        }

        debug!("Response: {}", response);
        Ok(response)
    }

    /// Encode length using RouterOS length encoding
    ///
    /// RouterOS uses a variable-length encoding for strings:
    /// - 0-0x7F: 1 byte with value
    /// - 0x80-0x3FFF: 2 bytes with high bit set on first byte
    /// - 0x4000-0x1FFFFF: 3 bytes with 0xC0 on first byte
    /// - 0x200000-0xFFFFFFF: 4 bytes with 0xE0 on first byte
    fn encode_length(len: usize) -> Vec<u8> {
        if len < 0x80 {
            vec![len as u8]
        } else if len < 0x4000 {
            vec![((len >> 8) | 0x80) as u8, (len & 0xFF) as u8]
        } else if len < 0x200000 {
            vec![
                ((len >> 16) | 0xC0) as u8,
                ((len >> 8) & 0xFF) as u8,
                (len & 0xFF) as u8,
            ]
        } else {
            vec![
                ((len >> 24) | 0xE0) as u8,
                ((len >> 16) & 0xFF) as u8,
                ((len >> 8) & 0xFF) as u8,
                (len & 0xFF) as u8,
            ]
        }
    }

    /// Run a script by name
    pub async fn run_script(&mut self, script_name: &str) -> Result<()> {
        info!("Running script: {}", script_name);

        let command = vec![
            "/system/script/run".to_string(),
            format!("=source={}", script_name),
        ];

        self.send_command(&command).await?;
        let response = self.read_response().await?;

        if response.contains("!done") {
            Ok(())
        } else if response.contains("!trap") {
            Err(anyhow!("Script execution error: {}", response))
        } else {
            Err(anyhow!("Unexpected response: {}", response))
        }
    }

    /// Get PoE status
    pub async fn get_poe_status(&mut self) -> Result<String> {
        info!("Querying PoE interfaces");

        let command = vec!["/interface/ethernet/poe/print".to_string()];

        self.send_command(&command).await?;
        let response = self.read_response().await?;

        if !response.contains("!done") {
            if response.contains("!trap") {
                return Err(anyhow!("Failed to query PoE status: {}", response));
            }
            return Ok(format!("Status query returned: {}", response));
        }

        let mut status_output = String::new();
        status_output.push_str("\n");
        status_output.push_str(&"=".repeat(70));
        status_output.push('\n');
        status_output.push_str("PoE Interface Status\n");
        status_output.push_str(&"=".repeat(70));
        status_output.push('\n');

        // Parse the RouterOS API response - split by !re to get individual records
        let records: Vec<&str> = response.split("!re").collect();
        let mut interfaces: Vec<(String, String)> = Vec::new();

        for record in records.iter().skip(1) {
            if record.contains("!done") {
                break;
            }

            // Extract key=value pairs from this record
            let fields: Vec<&str> = record.split('=').collect();
            let mut name = String::new();
            let mut poe_out = String::new();

            // Process fields: [0] is .id or empty, [1] is key, [2] is value, [3] is key, [4] is value, etc.
            let mut i = 0;
            while i < fields.len() {
                let field = fields[i].trim();

                // Skip .id field and empty fields
                if field.is_empty() || field.starts_with('.') {
                    i += 1;
                    continue;
                }

                // Check if this is a key and next index has a value
                if i + 1 < fields.len() {
                    let key = field;
                    let value = fields[i + 1].trim();

                    match key {
                        "name" => {
                            // Extract just the name part (stop at newline or next special char)
                            name = value
                                .split(|c: char| c == '\n' || c == '!' || c == '\r')
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string();
                        }
                        "poe-out" => {
                            // Extract just the poe-out value
                            poe_out = value
                                .split(|c: char| c == '\n' || c == '!' || c == '\r')
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string();
                        }
                        _ => {}
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }

            if !name.is_empty() {
                if poe_out.is_empty() {
                    poe_out = "off".to_string();
                }
                interfaces.push((name, poe_out));
            }
        }

        if interfaces.is_empty() {
            status_output.push_str("No PoE interfaces found\n");
        } else {
            status_output.push_str(&format!("Total Interfaces: {}\n", interfaces.len()));
            status_output.push_str(&"-".repeat(70));
            status_output.push('\n');
            status_output.push_str(&format!("{:<20} {:<20}\n", "Interface", "PoE Status"));
            status_output.push_str(&"-".repeat(70));
            status_output.push('\n');

            for (name, poe_status) in interfaces {
                let display_status = match poe_status.to_lowercase().as_str() {
                    "off" => "OFF",
                    "forced-on" => "FORCED ON",
                    "auto-on" => "AUTO ON",
                    "backup" => "BACKUP",
                    _ => &poe_status.to_uppercase(),
                };
                status_output.push_str(&format!("{:<20} {:<20}\n", name, display_status));
            }
        }

        status_output.push_str(&"-".repeat(70));
        status_output.push('\n');
        Ok(status_output)
    }
}

impl Drop for MikroTikClient {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
            debug!("Closed connection to {}", self.host);
        }
    }
}
