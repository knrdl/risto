use std::{borrow::Cow, io, io::prelude::*, net::TcpStream, str::FromStr};

type PathParts = Vec<String>;

pub enum RequestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl FromStr for RequestMethod {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "GET" => Ok(RequestMethod::Get),
            "POST" => Ok(RequestMethod::Post),
            "PUT" => Ok(RequestMethod::Put),
            "PATCH" => Ok(RequestMethod::Patch),
            "DELETE" => Ok(RequestMethod::Delete),
            _ => Err(()),
        }
    }
}

pub struct Request {
    pub pathparts: PathParts,
    pub method: RequestMethod,
}

impl Request {
    pub fn parse(stream: &TcpStream) -> Result<Request, Response> {
        let buf_reader = io::BufReader::new(stream);

        if let Some(Ok(request_line)) = buf_reader.lines().next() {
            let mut parts = request_line.split_whitespace();

            let method = RequestMethod::from_str(parts.next().unwrap_or(""))
                .map_err(|_| Response(405, ResponseBody::Text("Method Not Allowed".into())))?;
            let path_query = parts.next().unwrap_or("");
            let version = parts.next().unwrap_or("");

            if parts.next().is_some() {
                return Err(Response(
                    400,
                    ResponseBody::Text("Malformed request line".into()),
                ));
            }

            if version != "HTTP/1.1" {
                return Err(Response(
                    505,
                    ResponseBody::Text("HTTP Version Not Supported".into()),
                ));
            }

            let path = path_query
                .split('?')
                .next()
                .unwrap_or("")
                .trim_end_matches('/');
            if !path.starts_with("/") && !path.is_empty() {
                return Err(Response(400, ResponseBody::Text("Illegal path".into())));
            }

            let pathparts = path
                .split("/")
                .skip(1)
                .map(urldecode)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Response(400, ResponseBody::Text(e.into())))?;

            Ok(Request { pathparts, method })
        } else {
            Err(Response(400, ResponseBody::Text("Invalid request".into())))
        }
    }
}

type StatusCode = u16;

pub struct Response(pub StatusCode, pub ResponseBody);

impl Response {
    pub fn send(&self, mut stream: TcpStream) -> io::Result<()> {
        let Response(status, body) = self;

        let headers = format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n{}\r\n",
            status,
            status,
            body.content_length(),
            body.content_type()
                .map_or("".to_string(), |ct| format!("Content-Type: {}\r\n", ct))
        );

        stream.write_all(headers.as_bytes())?;
        body.write_to(stream)?;
        Ok(())
    }
}

pub enum ResponseBody {
    Html(Cow<'static, str>),
    Text(Cow<'static, str>),
    Json(Cow<'static, str>),
    Png(Vec<u8>),
    None,
}

impl ResponseBody {
    fn content_type(&self) -> Option<&'static str> {
        match self {
            ResponseBody::Html(_) => Some("text/html"),
            ResponseBody::Text(_) => Some("text/plain"),
            ResponseBody::Json(_) => Some("application/json"),
            ResponseBody::Png(_) => Some("image/png"),
            ResponseBody::None => None,
        }
    }

    fn content_length(&self) -> usize {
        match self {
            ResponseBody::Html(v) | ResponseBody::Text(v) | ResponseBody::Json(v) => v.len(),
            ResponseBody::Png(v) => v.len(),
            ResponseBody::None => 0,
        }
    }

    fn write_to(&self, mut stream: TcpStream) -> io::Result<()> {
        match self {
            ResponseBody::Html(v) | ResponseBody::Text(v) | ResponseBody::Json(v) => {
                stream.write_all(v.as_bytes())
            }
            ResponseBody::Png(v) => stream.write_all(v),
            ResponseBody::None => Ok(()),
        }
    }
}

fn urldecode(text: &str) -> Result<String, &'static str> {
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
