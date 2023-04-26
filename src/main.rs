use redis_starter_rust::ThreadPool;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
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
    let mut split_string = input.split("\r\n");

    let str_cmd = split_string.next().unwrap();
    match str_cmd {
        "*2" => return String::from("*0\r\n"),
        "*1" => return String::from("+PONG\r\n"),
        _ => return String::from("+PONG\r\n"),
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
        RESPType::Array => process_array(input),
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

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        pool.execute(|| match stream {
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
}
