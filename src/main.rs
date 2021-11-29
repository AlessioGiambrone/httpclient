extern crate clap;
use anyhow::Result;
use clap::{App, Arg};

fn main() -> Result<()> {
    let matches = App::new("httpclient")
        .version("0.1.0")
        .author("Alessio Giambrone <AlessioGiambrone@users.noreply.github.com>")
        .about("")
        .arg(
            Arg::with_name("INPUT")
                .help("Path to the .HTTP file to use")
                .required(true)
                .min_values(1)
                .index(1),
        )
        .arg(
            Arg::with_name("request number")
                .default_value("0")
                .short("n")
                .help(
                    "Selects the choosen request in the file, if more than one is present.
Numbering starts from 0; use \"a\" to execute them all",
                ),
        )
        .arg(
            Arg::with_name("timeout")
                .default_value("120")
                .short("t")
                .help("request timeout, in seconds"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity")
                .long_help(
                    "three verbosity levels are available:
(None) the output is only the unformatted response body.
       Useful when using httpclient with other tools (e.g. `jq`, `xmllint`, ...)
-v     the output shows response status, elapsed time, headers and body; 
       if `CONTENT-TYPE` is `application-json` the body will be beautified.
       Useful when reading the output is a human.
-vv    the output shows sent request method, URL, headers, url parameters, body 
       followed by all what is printed with -v
       Useful when debugging.
",
                ),
        )
        .get_matches();

    let verbosity = matches.occurrences_of("v");
    let filepaths: Vec<_> = matches.values_of("INPUT").unwrap().collect();
    let request_timeout: u64 = matches.value_of("timeout").unwrap().parse::<u64>()?;
    let selected_req_number_str = matches.value_of("request number").unwrap();
    let selected_req_number: isize = match selected_req_number_str {
        "a" => -1,
        "" => 0,
        _ => selected_req_number_str.parse::<isize>()?,
    };

    for filepath in filepaths {
        let rqsp = httpclient::worker::FileParser {};
        let reqs = rqsp.parse_from_file(&filepath)?;

        httpclient::execute_requests(verbosity, request_timeout, reqs, selected_req_number)?;
    }

    Ok(())
}
