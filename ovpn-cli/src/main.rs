use std::{env, io};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use common::command::Cli;
use clap::Parser;
use common::paths::SOCKET_PATH;

fn main() -> io::Result<()> {
    _ = Cli::parse();

    let args: Vec<String> = env::args().skip(1).collect();
    let command = format!("{}\n", args.join(" "));

    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write_all(command.as_bytes())?;

    // Create a buffer reader to handle multiple lines
    let reader = BufReader::new(stream);

    // Variables to store the response components
    let mut response_length: usize = 0;
    let mut response_status = String::new();
    let mut response_message = String::new();
    let mut bytes_read: usize = 0;

    // Read the response line by line
    for line in reader.lines() {
        let line = line?;

        // Ignore event messages starting with '!'
        if line.starts_with('!') {
            continue;
        }

        // Check if this is the command response
        if response_length == 0 {
            // Parse the first line which should be in the format: <length>:<status>:<message>
            let parts: Vec<&str> = line.splitn(3, ':').collect();
            if parts.len() == 3 {
                response_length = parts[0].parse().unwrap_or(0);
                response_status = parts[1].to_string();
                response_message.push_str(parts[2]);
                bytes_read = response_status.len() + 1 + response_message.len(); // Include the ':' and the message
            }
        } else {
            // Accumulate the rest of the message
            response_message.push('\n');
            response_message.push_str(&line);
            bytes_read += line.len() + 1; // +1 for the newline character
        }

        // Stop reading if we've read the complete response
        if bytes_read >= response_length {
            break;
        }
    }

    // Output the parsed response
    if response_status == "err" {
        return Err(io::Error::new(io::ErrorKind::Other, response_message));
    }

    println!("{}", response_message);

    Ok(())
}
