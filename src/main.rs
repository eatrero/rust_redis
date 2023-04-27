use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::vec::Vec;

#[derive(Debug, PartialEq)]
enum RESPType {
    Array,
    String,
    BulkString,
    Integer,
    Error,
}

fn char_to_type(c: char) -> RESPType {
    return match c {
        '+' => RESPType::String,
        '-' => RESPType::Error,
        ':' => RESPType::Integer,
        '$' => RESPType::BulkString,
        '*' => RESPType::Array,
        _ => {
            println!("error determing type");
            return RESPType::Error;
        }
    };
}

fn process_array(input: String) -> String {
    match get_cmd(input.clone()).as_str() {
        "ping" => return String::from("+PONG\r\n"),
        "echo" => return get_string_after_cmd(input.clone(), String::from("echo")),
        _ => return String::from("*0\r\n"),
    };
}

fn send_resp(mut stream: &TcpStream, output: String) {
    stream.write(String::from(output).as_bytes()).unwrap();
    stream.flush().unwrap()
}

fn parse_req(input: String) -> String {
    let resp_type_char = input.chars().nth(0).unwrap();

    let resp_type = char_to_type(resp_type_char);

    return match resp_type {
        RESPType::Array => {
            let parts = process_array(input);
            return parts;
        }
        _ => {
            println!("error in type");
            return String::from("+Error\r\n");
        }
    };
}

const MESSAGE_SIZE: usize = 512;

fn get_request(mut stream: &TcpStream) -> String {
    // Store all the bytes for our received String
    let mut received: Vec<u8> = vec![];

    let mut rx_bytes = [0u8; MESSAGE_SIZE];

    // Read from the current data in the TcpStream
    let bytes_read = match stream.read(&mut rx_bytes) {
        Ok(bytes_read) => bytes_read,
        Err(e) => {
            println!("error reading stream: {}", e);
            0
        }
    };

    received.extend_from_slice(&rx_bytes[..bytes_read]);
    return String::from_utf8(received).unwrap();
}

fn parse_stream(stream: &TcpStream) {
    loop {
        let req_string = get_request(&stream);
        if req_string == "" || req_string.len() == 0 {
            break;
        }
        let resp_string = parse_req(req_string);
        send_resp(&stream, resp_string);
    }
}

fn has_cmd(total_str: String, cmd: String) -> bool {
    let split_str = total_str.split("\r\n");

    match split_str
        .clone()
        .find(|x| x.eq_ignore_ascii_case(cmd.as_str()))
    {
        Some(_) => return true,
        None => return false,
    };
}

fn get_cmd(total_str: String) -> String {
    let split_str = total_str.split("\r\n");

    return split_str.clone().skip(2).next().unwrap().to_string();
}

fn get_string_after_cmd(total_str: String, cmd: String) -> String {
    let split_str = total_str.split("\r\n");

    let after_cmd = split_str
        .clone()
        .skip_while(|x| !x.eq_ignore_ascii_case(cmd.as_str()))
        .skip(1);

    let cnt = after_cmd.clone().fold(0, |acc, x| acc + 1) / 2;

    let remaining = after_cmd.fold(String::from(""), |acc, x| acc + "\r\n" + x);
    let y = "*".to_owned() + &cnt.to_string();
    let z = y + &remaining;

    return z;
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || match stream {
            Ok(mut _stream) => loop {
                parse_stream(&_stream)
            },
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }

    println!("ended")
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn char_to_type_returns_proper_type() {
        let type_array = char_to_type('*');
        assert_eq!(type_array, RESPType::Array);

        let type_string = char_to_type('+');
        assert_eq!(type_string, RESPType::String);

        let type_error = char_to_type('-');
        assert_eq!(type_error, RESPType::Error);

        let type_integer = char_to_type(':');
        assert_eq!(type_integer, RESPType::Integer);

        let type_bulk_string = char_to_type('$');
        assert_eq!(type_bulk_string, RESPType::BulkString);
    }

    #[test]
    fn test_get_cmd_length() {
        let test: String = String::from("3");

        assert_eq!(3, test.parse().unwrap());

        let test: String = String::from("*32");
        assert_eq!(32, test[1..].parse().unwrap());
    }

    #[test]
    fn test_get_remaining_string() {
        let test: String = String::from("*3\r\n$4\r\necho\r\n$3\r\nhey\r\n$2\r\nyo\r\n");

        let test3 = get_string_after_cmd(test, String::from("echo"));
        assert_eq!(test3, "*2\r\n$3\r\nhey\r\n$2\r\nyo\r\n");
    }

    #[test]
    fn test_has_cmd() {
        let test: String = String::from("*3\r\n$4\r\necho\r\n$3\r\nhey\r\n$2\r\nyo\r\n");

        assert_eq!(has_cmd(test.clone(), "echo".to_string()), true);
        assert_eq!(has_cmd(test.clone(), "kjhkh".to_string()), false);

        assert_eq!(get_cmd(test.clone()), "echo");
    }
}
