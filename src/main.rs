pub mod ctrlc;
pub mod http;

use http::{Request, RequestMethod, Response, ResponseBody};

use std::{fs, iter, net::TcpListener};

fn main() {
    ctrlc::init();

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for connection_attempt in listener.incoming() {
        match connection_attempt {
            Ok(stream) => {
                let response = match http::Request::parse(&stream) {
                    Ok(request) => answer_request(request),
                    Err(response) => response,
                };
                if let Err(e) = response.send(stream) {
                    eprintln!("Write response failed: {}", e);
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

#[inline]
fn answer_request(request: Request) -> Response {
    let response405 = Response(405, ResponseBody::Text("Method Not Allowed".into()));
    let Request { pathparts, method } = request;

    match pathparts
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["index.html"] => match method {
            RequestMethod::Get => match fs::read_to_string("www/index.html") {
                Ok(body) => Response(200, ResponseBody::Html(body.into())),
                Err(e) => Response(500, ResponseBody::Text(e.to_string().into())),
            },
            _ => response405,
        },
        ["manifest.json"] => match method {
            RequestMethod::Get => match fs::read_to_string("www/manifest.json") {
                Ok(body) => Response(200, ResponseBody::Json(body.into())),
                Err(e) => Response(500, ResponseBody::Text(e.to_string().into())),
            },
            _ => response405,
        },
        ["images", "logo-transparent.png"] | ["images", "logo-512x512.png"] => match method {
            RequestMethod::Get => match fs::read(format!("www/{}", pathparts.join("/"))) {
                Ok(body) => Response(200, ResponseBody::Png(body)),
                Err(e) => Response(500, ResponseBody::Text(e.to_string().into())),
            },
            _ => response405,
        },
        ["favorites.json"] => match method {
            RequestMethod::Get => Response(
                200,
                ResponseBody::Json(
                    fs::read_to_string("data/favorites.json")
                        .unwrap_or_else(|_| "{}".into())
                        .into(),
                ),
            ),
            _ => response405,
        },
        ["items"] => match method {
            RequestMethod::Get => Response(
                200,
                ResponseBody::Text(
                    fs::read_to_string("data/items.txt")
                        .unwrap_or_else(|_| "".into())
                        .into(),
                ),
            ),
            _ => response405,
        },
        ["items", name] if !name.is_empty() => match method {
            RequestMethod::Post | RequestMethod::Delete => {
                let content = fs::read_to_string("data/items.txt").unwrap_or_else(|_| "".into());
                let items = content.lines();
                let mut items = match method {
                    RequestMethod::Post => items.chain(iter::once(*name)).collect::<Vec<_>>(),
                    RequestMethod::Delete => items.filter(|x| x != name).collect::<Vec<_>>(),
                    _ => items.collect::<Vec<_>>(),
                };
                items.sort_unstable_by_key(|x| x.to_lowercase());
                items.dedup(); // remove duplicates
                let content = items.join("\n");
                if let Err(e) = fs::write("data/items.txt", &content) {
                    return Response(
                        500,
                        ResponseBody::Text(format!("Write failed: {}", e).into()),
                    );
                }
                Response(200, ResponseBody::Text(content.into()))
            }
            _ => response405,
        },
        _ => Response(404, ResponseBody::Text("Not Found".into())),
    }
}
