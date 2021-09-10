use anyhow::Result;
use reqwest::blocking::{Client, Response};
use reqwest::header;
use reqwest::Method;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

pub struct Request {
    pub method: String,
    pub url: String,
    pub url_parameters: Vec<(String, String)>,
    pub headers: HashMap<String, String>,
    pub protocol: String,
    pub body: String,
}

impl Request {
    pub fn new() -> Request {
        Request {
            headers: HashMap::new(),
            url: "".to_string(),
            url_parameters: Vec::new(),
            method: "".to_string(),
            protocol: "HTTP/1.1".to_string(),
            body: "".to_string(),
        }
    }

    pub fn execute(&self, timeout: u64) -> Result<Response, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response_body = client
            .request(
                Method::from_bytes(self.method.as_bytes())?,
                self.get_url_with_parameters()?,
            )
            .headers(self.format_headers()?)
            .timeout(Duration::new(timeout, 0))
            .body(self.body.to_string())
            .send()?;

        Ok(response_body)
    }

    fn format_headers(&self) -> Result<header::HeaderMap, Box<dyn std::error::Error>> {
        let mut reqw_headers = header::HeaderMap::new();
        for v in self.headers.iter() {
            reqw_headers.append(
                header::HeaderName::from_bytes(v.0.as_bytes())?,
                header::HeaderValue::from_str(v.1)?,
            );
        }
        Ok(reqw_headers)
    }

    fn get_url_with_parameters(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url: reqwest::Url;
        if self.url_parameters.len() > 0 {
            url = reqwest::Url::parse_with_params(&self.url, self.url_parameters.iter())?;
        } else {
            url = reqwest::Url::parse(&self.url)?;
        }
        Ok(url.as_str().to_string())
    }

    fn print_request_headers(&self) -> String {
        let mut headers_string_buffer = "".to_string();
        for k in &self.headers {
            headers_string_buffer.push_str(&format!("   {}: {:?}\n", k.0, k.1));
        }
        headers_string_buffer
    }

    fn print_url_parameters(&self) -> String {
        let mut buffer = "".to_string();
        for k in &self.url_parameters {
            buffer.push_str(&format!("   {}: {}\n", k.0, k.1));
        }
        buffer
    }
}

impl fmt::Display for Request {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        write!(
            dest,
            "{} {} {}\nheaders:\n{}\nurl parameters:\n{}\nbody:\n{}",
            self.method,
            self.url,
            self.protocol,
            self.print_request_headers(),
            self.print_url_parameters(),
            self.body,
        )
    }
}

fn print_response_headers(h: &header::HeaderMap) -> String {
    let mut header_buffer = "".to_string();
    for k in h {
        header_buffer.push_str(&format!("{}: {:?}\n", k.0, k.1));
    }
    header_buffer
}

pub fn verbose_print_response(response: Response, elapsed: &Duration) -> Result<String> {
    let status = response.status();
    let headers = response.headers();
    Ok(format!(
        "{} - {:?}\n{}\n{}",
        status,
        elapsed,
        print_response_headers(headers),
        match headers.get(reqwest::header::CONTENT_TYPE) {
            None => response.text()?,
            Some(ct) => {
                match parse_content_type(&ct)? {
                    "application/json" => beautify_json(response.text()?)?,
                    _ => response.text()?,
                }
            }
        }
    ))
}

fn parse_content_type(ct: &reqwest::header::HeaderValue) -> Result<&str> {
    let type_splitted: Vec<&str> = ct.to_str()?.split(";").collect();
    Ok(type_splitted[0])
}

fn beautify_json(json_text: String) -> Result<String> {
    let parsed = json::parse(&json_text)?;
    Ok(json::stringify_pretty(parsed, 2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_headers() {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer mysupresecrettoken".to_string(),
        );
        let request = Request {
            headers: headers,
            url: "".to_string(),
            url_parameters: Vec::new(),
            method: "".to_string(),
            protocol: "HTTP/1.1".to_string(),
            body: "".to_string(),
        };
        let formatted_headers = request.format_headers().unwrap();
        assert_eq!(
            formatted_headers.get("Authorization").unwrap(),
            reqwest::header::HeaderValue::from_static("Bearer mysupresecrettoken")
        );
    }
}
