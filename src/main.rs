use std::net::{ TcpStream };
use std::io::{ Write, BufReader, BufRead };

fn main() {
    let reader = BufReader::new(std::io::stdin());

    print!("Enter url: ");
    std::io::stdout().flush().unwrap();

    for ln in reader.lines() {
        match ln.as_ref().map(|s| s.as_str()) {
            Ok("exit") | Ok("quit") | Ok("e") | Ok("q") => return,
            Ok("Help") | Ok("help") | Ok("h") => {
                print!("Enter url including 'http://' port is optional: ");
                std::io::stdout().flush().unwrap();
                continue;
            },
            Ok("") => {},
            Ok(line) => {
                connect(line);
            },
            _ => {},
        }

        print!("Enter url: ");
        std::io::stdout().flush().unwrap();
    }
}

fn connect(line: &str) {
    if !line.starts_with("http://") {
        println!("Only http supported");
        return;
    }
    let line = &line[7..];

    let mut i = line.splitn(2, "/");
    let mut server_adr = if let Some(s) = i.next() {
        s.to_owned()
    } else {
        println!("Failed to parse server address");
        return
    };
    let uri = "/".to_owned() + i.next().or(Some("")).unwrap();

    let mut bytes = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\n\r\n",
        uri, server_adr
    ).into_bytes();

    if !server_adr.contains(":") {
        server_adr += ":80";
    }

    let mut server = if let Ok(s) = TcpStream::connect(&server_adr) {
        s
    } else {
        println!("Failed to connect to server: {}", server_adr.to_string());
        return;
    };
    let timeout = std::time::Duration::from_millis(2000);
    server.set_read_timeout(Some(timeout)).expect("Failed to set timeout");
    server.set_write_timeout(Some(timeout)).expect("Failed to set timeout");

    match server.write(bytes.as_mut()) {
        Ok(bytes_sent) if bytes_sent != bytes.len() => {
            println!("Failed to send the entire request to the server: '{}'", server_adr);
            return;
        },
        Err(err) => {
            println!("Failed to send request to server: '{}', with error: '{}'", server_adr, err.to_string());
            return;
        }
        _ => {},
    }

    let mut reader = BufReader::new(server);
    let mut lines = String::new();

    // Skip HTTP header
    while let Ok(bytes_read) = reader.read_line(&mut lines) {
        if bytes_read == 0 {
            println!("Invalid response");
            return;
        }
        if lines.ends_with("\r\n\r\n") {
            break;
        }
    }
    println!("'{}'\n", lines);
    if !lines.ends_with("\r\n\r\n") {
        println!("Failed to read entire message from server: '{}'", server_adr);
        return;
    }
    lines.clear();

    //Read actual content
    while let Ok(bytes_read) = reader.read_line(&mut lines) {
        if bytes_read == 0 {
            break;
        }
    }
    println!("{}\n", lines);
}