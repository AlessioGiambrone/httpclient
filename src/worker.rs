use crate::*;
use regex::Regex;
use std::env;
use std::fmt;
use std::fs;
use std::io::{Error, ErrorKind, Result as IoResult};

/// HTTPParser parses an HTTP request text into a single valid `request` struct.
///
/// # Examples
///
/// ```
/// let mut http_parser = httpclient::worker::HTTPParser::new().unwrap();
/// http_parser.parse("https://example.com/comments/1").unwrap();
/// assert_eq!(http_parser.request.url, "https://example.com/comments/1");
/// ```
///
/// ```
/// let input_text = "// RFC 2616 example
/// POST https://example.com/comments HTTP/1.1
/// content-type: application/json
///
/// {
///     \"name\": \"sample\",
///     \"time\": \"Wed, 21 Oct 2015 18:27:50 GMT\"
/// }
/// ";
/// let mut http_parser= httpclient::worker::HTTPParser::new().unwrap();
/// http_parser.parse(input_text).unwrap();
/// assert_eq!(http_parser.request.url, "https://example.com/comments");
/// assert_eq!(http_parser.request.protocol, "HTTP/1.1");
/// ```
pub struct HTTPParser {
    pub request: request::Request,
    head_done: bool,
    body_buffer: Vec<String>,
}

impl HTTPParser {
    pub fn new() -> IoResult<HTTPParser> {
        let w = HTTPParser {
            request: request::Request::new(),
            head_done: false,
            body_buffer: Vec::new(),
        };

        Ok(w)
    }

    pub fn parse(&mut self, contents: &str) -> IoResult<()> {
        for line in contents.split("\n") {
            if line.trim().len() == 0 && self.request.url == "" {
                // just an empty line before the request text starts
                continue;
            }
            if line.starts_with("//") || line.starts_with("#") {
                // this is a comment!
                continue;
            }
            if self.head_done {
                // RAW body: we're after the URL/params/headers section
                self.body_buffer.push(line.to_string());
                continue;
            }
            if HTTPParser::is_section_break(line) && self.request.url != "" {
                self.head_done = true;
                continue;
            }
            if !self.head_done {
                if self.could_be_headers_or_attr(line) {
                    self.parse_header(&line)?;
                    self.parse_url_parameter(&line)?;
                }
                if self.request.method == "" {
                    self.parse_method(line);
                }
                if self.request.url == "" {
                    self.parse_url(&line)?;
                }
            }
        }
        self.request.body = self.body_buffer.join("\n").trim().to_string();

        Ok(())
    }

    fn parse_url_parameter(&mut self, line: &str) -> IoResult<()> {
        let trimmed = line.trim_start();
        if trimmed == line {
            // this is an header
            return Ok(());
        }
        let splitted: Vec<&str> = trimmed.splitn(2, '=').collect();
        if splitted.len() < 2 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("invalid url parameter in {}", line),
            ));
        }
        if splitted[0].len() == 0 || splitted[1].len() == 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("invalid url parameter in {}", line),
            ));
        }
        self.request
            .url_parameters
            .push((splitted[0][1..].to_string(), splitted[1].to_string()));
        Ok(())
    }

    fn parse_header(&mut self, line: &str) -> IoResult<()> {
        if line.trim_start() != line {
            // this is an URL parameter
            return Ok(());
        }
        let split = line.split(":").collect::<Vec<&str>>();
        if &split.len() < &2 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("invalid header in {}", line),
            ));
        }
        let key = split[0];
        let values = &split[1..split.len()];
        let raw_value = values
            .iter()
            .fold(String::from(""), |acc, x| acc.to_string() + x);
        let value = raw_value.trim_start();
        self.request
            .headers
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn could_be_headers_or_attr(&mut self, line: &str) -> bool {
        self.request.url != "" && !HTTPParser::is_section_break(line) && !self.head_done
    }

    fn is_section_break(line: &str) -> bool {
        line.trim().len() == 0
    }

    fn parse_url(&mut self, line: &str) -> IoResult<()> {
        let split = line.split(" ").collect::<Vec<&str>>();
        if &split.len() < &1 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("URL not found in {}", line),
            ));
        }

        let mut url_candidate = split[split.len() - 1];
        // TODO
        let protocol_regexp: Regex = Regex::new(r"HTTP/(\d)(\.\d)?($|\n|\r)").unwrap();

        if protocol_regexp.is_match(url_candidate) && &split.len() > &2 {
            self.request.protocol = url_candidate.to_string();
            url_candidate = split[split.len() - 2];
        } else if protocol_regexp.is_match(url_candidate) && &split.len() <= &2 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("invalid URL: {}", line),
            ));
        }

        self.request.url = url_candidate.to_string();
        Ok(())
    }

    fn parse_method(&mut self, line: &str) {
        let split = line.split(" ");
        let method_candidate = split.collect::<Vec<&str>>()[0];
        if HTTP_METHODS.iter().any(|&i| i == method_candidate) {
            self.request.method = method_candidate.to_string();
            return;
        }
        self.request.method = "GET".to_string();
    }
}

impl fmt::Display for HTTPParser {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        write!(dest, "{}", self.request)
    }
}

pub struct FileParser {}

impl FileParser {
    pub fn parse_from_file(mut self, file_path: &str) -> Result<Vec<request::Request>> {
        let raw_contents = fs::read_to_string(file_path)?;
        let mut content_lines: Vec<String> = Vec::new();
        for line in raw_contents.split("\n") {
            content_lines.push(self.replace_env(line)?);
        }
        let content = content_lines.join("\n");

        self.parse_many(&content)
    }

    pub fn parse_many(self, file_content: &str) -> Result<Vec<request::Request>> {
        let mut requests: Vec<request::Request> = Vec::new();
        let mut raw_requests: Vec<Vec<String>> = vec![vec![]];
        for line in file_content.split("\n") {
            if line.starts_with("###") {
                raw_requests.push(Vec::new());
            }
            raw_requests.last_mut().unwrap().push(line.to_string());
        }
        for raw_request in raw_requests {
            let mut w = HTTPParser::new()?;
            w.parse(&raw_request.join("\n"))?;
            requests.push(w.request);
        }

        Ok(requests)
    }

    fn replace_env(&mut self, candidate_str: &str) -> IoResult<String> {
        // TODO refactor here
        let re = Regex::new(r"\{\{(?P<key>\w+)\}\}").unwrap();

        let mut result: IoResult<String> = Ok(candidate_str.to_string());
        for m in re.find_iter(candidate_str) {
            let key = candidate_str.get(m.start() + 2..m.end() - 2).unwrap();
            result = self.replace_single_env_var(&result?, key);
        }

        result
    }

    fn replace_single_env_var(&mut self, candidate_str: &str, key: &str) -> IoResult<String> {
        if env::var(key).is_ok() {
            return Ok(self.rpl(candidate_str, key, &env::var(key).unwrap()));
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                format!("you must provide a value for key {}", key),
            ));
        };
    }

    fn rpl(&self, candidate_str: &str, key: &str, subs: &str) -> String {
        let key_subs: String = "{{".to_string() + key + "}}";
        candidate_str.replacen(&key_subs, subs, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_header() {
        let contents = "POST https://it.wikipedia.org\nAuth: nooone\nHead: true";
        let mut hrp = HTTPParser::new().unwrap();
        &hrp.parse(contents).unwrap();
        assert_eq!(&hrp.request.method, "POST");
        assert_eq!(&hrp.request.url, "https://it.wikipedia.org");
        assert!(&hrp.request.headers.contains_key("Auth"));
        assert_eq!(
            &hrp.request.headers.get("Auth"),
            &Some(&"nooone".to_string())
        );
        assert_eq!(&hrp.request.headers.get("Head"), &Some(&"true".to_string()));
    }

    #[test]
    fn uri_post() {
        let contents = "POST https://it.wikipedia.org";
        let mut hrp = HTTPParser::new().unwrap();
        &hrp.parse(contents).unwrap();
        assert_eq!(&hrp.request.method, "POST");
        assert_eq!(&hrp.request.url, "https://it.wikipedia.org");
    }

    #[test]
    fn uri_post_2() {
        let contents = "POST https://it.wikipedia.org\n\n{\"a\":1}\n## this is a comment";
        let mut hrp = HTTPParser::new().unwrap();
        &hrp.parse(contents).unwrap();
        assert_eq!(&hrp.request.method, "POST");
        assert_eq!(&hrp.request.url, "https://it.wikipedia.org");
        assert_eq!(&hrp.request.body, "{\"a\":1}");
    }

    #[test]
    fn uri() {
        let contents = "https://it.wikipedia.org";
        let mut hrp = HTTPParser::new().unwrap();
        &hrp.parse(contents).unwrap();
        assert_eq!(&hrp.request.method, "GET");
        assert_eq!(&hrp.request.url, "https://it.wikipedia.org");
    }

    #[test]
    fn uri_blank_lines() {
        let contents = "\n  \n\nPOST https://it.wikipedia.org";
        let mut hrp = HTTPParser::new().unwrap();
        &hrp.parse(contents).unwrap();
        assert_eq!(&hrp.request.method, "POST");
        assert_eq!(&hrp.request.url, "https://it.wikipedia.org");
    }

    #[test]
    fn body() {
        let contents = "POST https://it.wikipedia.org\n#comment\n\nthis is\nthe body";
        let mut hrp = HTTPParser::new().unwrap();
        &hrp.parse(contents).unwrap();
        assert_eq!(&hrp.request.body, "this is\nthe body");
    }

    #[test]
    fn url_parameters() {
        let input_text = "  ?foo=bar";
        let mut worker = HTTPParser::new().unwrap();
        worker
            .parse_url_parameter(input_text)
            .expect("should be able to parse");
        assert_eq!(worker.request.url_parameters.len(), 1);
        assert_eq!(worker.request.url_parameters[0].0, "foo");
        assert_eq!(worker.request.url_parameters[0].1, "bar");
    }

    #[test]
    fn multi_requests() {
        let contents = "https://it.wikipedia.org\n###\nPOST https://en.wikipedia.org";
        let hrp = FileParser {};
        let result = &hrp.parse_many(contents).unwrap();
        assert_eq!(&result[0].url, "https://it.wikipedia.org");
        assert_eq!(&result[0].method, "GET");
        assert_eq!(&result[0].body, "");
        assert_eq!(&result[1].url, "https://en.wikipedia.org");
        assert_eq!(&result[1].method, "POST");
    }

    #[test]
    fn long_multi_requests() {
        let contents = "GET https://it.wikipedia.org
Authorization: Basic none

### Create
POST https://it.wikipedia.org/something
Authorization: Basic none
Content-Type: application/x-www-form-urlencoded

payload=my_payload

### Delete
DELETE https://it.wikipedia.org/something
Authorization: Basic none

### Test
GET https://it.wikipedia.org/something";
        let hrp = FileParser {};
        let result = &hrp.parse_many(contents).unwrap();
        assert_eq!(&result[0].url, "https://it.wikipedia.org");
        assert_eq!(&result[0].method, "GET");
        assert!(&result[0].headers.contains_key("Authorization"));
        assert_eq!(
            &result[0].headers.get("Authorization"),
            &Some(&"Basic none".to_string())
        );
        assert_eq!(&result[0].body, "");
        assert_eq!(&result[1].url, "https://it.wikipedia.org/something");
        assert_eq!(&result[1].method, "POST");
        assert_eq!(&result[1].body, "payload=my_payload");
        assert_eq!(&result[2].url, "https://it.wikipedia.org/something");
        assert_eq!(&result[2].method, "DELETE");
    }
}
