pub mod request;
use anyhow::Result;
use std::time::Instant;
pub mod worker;

static HTTP_METHODS: &'static [&str] = &[
    "OPTIONS", "GET", "HEAD", "POST", "PUT", "DELETE", "TRACE", "CONNECT",
];

pub fn execute_requests(
    verbosity: u64,
    request_timeout: u64,
    reqs: Vec<request::Request>,
    reqn: isize,
) -> Result<()> {
    let request_indexes: Vec<usize> = match reqn {
        -1 => Ok(std::ops::Range {
            start: 0,
            end: reqs.len(),
        }
        .collect()),
        _ => {
            if (reqn as usize) < reqs.len() {
                Ok(vec![reqn as usize])
            } else {
                Err(anyhow::anyhow!(
                    "invalid request index: {} out of {}",
                    reqn,
                    reqs.len()
                ))
            }
        }
    }?;

    for index in request_indexes.iter() {
        execute_request(verbosity, request_timeout, &reqs[*index as usize])?;
    }
    Ok(())
}

fn execute_request(verbosity: u64, timeout: u64, req: &request::Request) -> Result<()> {
    if verbosity > 1 {
        println!("===== Request:\n{}\n===== Response:", req)
    }
    let start_instant = Instant::now();
    let response = req.execute(timeout).expect("unable to execute request");
    let elapsed = start_instant.elapsed();

    if verbosity > 0 {
        println!("{}", request::verbose_print_response(response, &elapsed)?);
    } else {
        println!("{}", response.text()?);
    }
    Ok(())
}
