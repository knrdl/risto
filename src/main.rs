#![forbid(unsafe_code)]

use std::{
    fs,
    io::{prelude::*, BufReader},
    iter,
    net::{TcpListener, TcpStream},
};

type Status = u16;

type Response = (Status, Contents);

enum Contents {
    Html(String),
    Text(String),
    Json(String),
    Png(Vec<u8>),
    None,
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for connection_attempt in listener.incoming() {
        match connection_attempt {
            Ok(stream) => {
                let buf_reader = BufReader::new(&stream);
                match buf_reader.lines().next() {
                    Some(Ok(request_line)) => {
                        if let Err(e) = send_response(answer_request(request_line), stream) {
                            eprintln!("Write response failed: {}", e);
                        }
                    }
                    Some(Err(e)) => eprintln!("Read request failed: {}", e),
                    None => (),
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

#[inline]
fn send_response(response: Response, mut stream: TcpStream) -> Result<(), std::io::Error> {
    let (response_status, response_body) = response;
    let response_body_length = match response_body {
        Contents::Html(ref v) => v.len(),
        Contents::Text(ref v) => v.len(),
        Contents::Json(ref v) => v.len(),
        Contents::Png(ref v) => v.len(),
        Contents::None => 0,
    };
    let response_content_type = match response_body {
        Contents::Html(_) => Some("text/html"),
        Contents::Text(_) => Some("text/plain"),
        Contents::Json(_) => Some("application/json"),
        Contents::Png(_) => Some("image/png"),
        Contents::None => None,
    };
    stream.write_all(format!("HTTP/1.1 {response_status} {response_status}\r\n").as_bytes())?;
    if let Some(mimetype) = response_content_type {
        stream.write_all(format!("Content-Type: {mimetype}\r\n").as_bytes())?;
    }
    stream.write_all(format!("Content-Length: {response_body_length}\r\n\r\n").as_bytes())?;
    match response_body {
        Contents::Html(ref v) => stream.write_all(v.as_bytes()),
        Contents::Text(ref v) => stream.write_all(v.as_bytes()),
        Contents::Json(ref v) => stream.write_all(v.as_bytes()),
        Contents::Png(ref v) => stream.write_all(v),
        Contents::None => Ok(()),
    }?;
    Ok(())
}

#[inline]
fn urldecode(text: &str) -> Result<String, &str> {
    #[inline]
    fn append_frag<'a>(text: &mut String, frag: &mut String) -> Result<(), &'a str> {
        if !frag.is_empty() {
            let encoded: Result<Vec<u8>, _> = frag
                .chars()
                .collect::<Vec<char>>()
                .chunks(2)
                .map(|ch| u8::from_str_radix(&ch.iter().collect::<String>(), 16))
                .collect();
            if let Ok(v) = encoded {
                if let Ok(txt) = &std::str::from_utf8(&v) {
                    text.push_str(txt);
                } else {
                    return Err("malformed URI sequence");
                }
            } else {
                return Err("malformed URI sequence");
            }
            frag.clear();
        }
        Ok(())
    }

    let mut output = String::new();
    let mut encoded_ch = String::new();
    let mut iter = text.chars();
    while let Some(ch) = iter.next() {
        if ch == '%' {
            let char1 = iter.next();
            let char2 = iter.next();
            match (char1, char2) {
                (Some(v1), Some(v2)) => encoded_ch.push_str(&format!("{}{}", v1, v2)),
                _ => {
                    return Err("malformed URI sequence");
                }
            };
        } else {
            append_frag(&mut output, &mut encoded_ch)?;
            output.push(ch);
        }
    }
    append_frag(&mut output, &mut encoded_ch)?;
    Ok(output)
}

#[inline]
fn answer_request(request_line: String) -> Response {
    let response405: Response = (405, Contents::Text("Method Not Allowed".into()));

    let request_line_parts: Vec<&str> = request_line.split_whitespace().collect();
    if request_line_parts.len() != 3 || request_line_parts[2] != "HTTP/1.1" {
        return (505, Contents::Text("HTTP Version Not Supported".into()));
    }

    let request_method = request_line_parts[0];
    if !matches!(request_method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
        return response405;
    }

    let request_path_query = request_line_parts[1];
    let request_path = request_path_query.split("?").next().unwrap_or("");
    if !request_path.starts_with("/") {
        return (400, Contents::Text("Illegal path".into()));
    }
    let request_path = request_path.strip_suffix("/").unwrap_or(request_path); // remove trailing slashes

    let request_path_segments_result: Result<Vec<_>, _> = request_path
        .split("/")
        .skip(1)
        .map(|segment| urldecode(segment))
        .collect();

    match request_path_segments_result {
        Ok(request_path_segments) => {
            if request_path_segments.is_empty() || request_path_segments == ["index.html"] {
                return match request_method {
                    "GET" => (
                        200,
                        Contents::Html(include_str!("../www/index.html").into()),
                    ),
                    _ => response405,
                };
            }
            if request_path_segments == ["images", "logo-transparent.png"] {
                return match request_method {
                    "GET" => (
                        200,
                        Contents::Png(include_bytes!("../www/images/logo-transparent.png").into()),
                    ),
                    _ => response405,
                };
            }
            if request_path_segments == ["images", "logo-512x512.png"] {
                return match request_method {
                    "GET" => (
                        200,
                        Contents::Png(include_bytes!("../www/images/logo-512x512.png").into()),
                    ),
                    _ => response405,
                };
            }
            if request_path_segments == ["manifest.json"] {
                return match request_method {
                    "GET" => (
                        200,
                        Contents::Json(include_str!("../www/manifest.json").into()),
                    ),
                    _ => response405,
                };
            }
            if request_path_segments == ["favorites"] {
                return match request_method {
                    "GET" => (
                        200,
                        Contents::Json(fs::read_to_string("favorites.json").unwrap_or("{}".into())),
                    ),
                    _ => response405,
                };
            }
            if request_path_segments == ["items"] {
                return match request_method {
                    "GET" => (
                        200,
                        Contents::Text(fs::read_to_string("items.txt").unwrap_or("".into())),
                    ),
                    _ => response405,
                };
            }
            if request_path_segments.len() == 2
                && request_path_segments[0] == "items"
                && !request_path_segments[1].is_empty()
            {
                return match request_method {
                    "POST" | "DELETE" => {
                        let name = request_path_segments[1].as_str();
                        let content = fs::read_to_string("items.txt").unwrap_or("".into());
                        let items = content.lines();
                        let mut items = match request_method {
                            "POST" => items.chain(iter::once(name)).collect::<Vec<_>>(),
                            "DELETE" => items.filter(|&x| x != name).collect::<Vec<_>>(),
                            _ => items.collect::<Vec<_>>(),
                        };
                        items.sort_by_key(|x| x.to_lowercase());
                        items.dedup(); // remove duplicates
                        let content = items.join("\n");
                        if let Err(e) = fs::write("items.txt", &content) {
                            return (500, Contents::Text(format!("Write failed: {}", e)));
                        }
                        (200, Contents::Text(content))
                    }
                    _ => response405,
                };
            }
            (404, Contents::Text("Not Found".into()))
        }
        Err(e) => (400, Contents::Text(e.into())),
    }
}
