use reqwest::header::USER_AGENT;
use std::fs::read_to_string;
use std::path::Path;
use std::process;
use std::{thread, time};

fn help() {
    println!("Usage: ");
    println!("  --host");
    println!(" Provide a full fuzzing address (https://[NPF].site.com || http://site.com/[NPF])");
    println!("  --word");
    println!(" Provide a fuzzing values file located (words.txt)");
    println!("  --silent true");
    println!(" Display just 200 response.");
    println!("  --head true");
    println!(" Make HEAD request instead of GET.");
    println!("  --delay <u64>");
    println!(" Delay for some time for each request");
    println!("  --ua 'custom user agent'");
    println!(" Set custom user agent");
}

fn read_lines(filename: String) -> Vec<String> {
    let mut result = Vec::new();

    if Path::new(&filename).exists() {
        for line in read_to_string(filename).unwrap().lines() {
            result.push(line.to_string())
        }
    } else {
        println!("Cannot find a file: {}.", filename);
        help();
    }
    result
}

async fn get_contents(url: String, head: bool, ua: &String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    let client = reqwest::Client::new();
    let res: reqwest::Result<reqwest::Response>;

    if head == true {
        res = client.head(url).header(USER_AGENT, ua).send().await;
    } else {
        res = client.get(url).header(USER_AGENT, ua).send().await;
    }

    match res {
        Ok(res) => {
            result.push(res.status().to_string());
            let body = res.text().await.expect("failed to get payload");
            result.push(body);
        }
        Err(err) => {
            result.push("0 Connection Error".to_string());
            result.push(err.to_string());
        }
    };

    return result;
}

#[tokio::main]
async fn main() {
    let arguments = std::env::args();
    let arguments = arguments::parse(arguments).unwrap();

    let host = arguments.get::<String>("host").unwrap();
    let words = arguments.get::<String>("words").unwrap();
    let silent = arguments.get::<bool>("silent").unwrap_or(false);
    let head = arguments.get::<bool>("head").unwrap_or(false);
    let delay = arguments.get::<u64>("delay").unwrap_or(0);
    let ua = arguments.get::<String>("ua").unwrap_or(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/120.0"
            .to_string(),
    );

    if host.chars().count() <= 7 || words.chars().count() <= 0 {
        help();
        process::exit(1);
    }

    let pause = time::Duration::from_secs(delay);
    for line in read_lines(words) {
        if line.chars().count() > 0 {
            let url = host.replace("[NPF]", &line);
            let text = get_contents(url, head, &ua).await;

            if silent == true {
                if reqwest::StatusCode::OK.to_string() == text[0] {
                    let digest = md5::compute(text[1].as_bytes());
                    println!("{} - {} [{:x}]", line, text[0], digest);
                }
            } else {
                let digest = md5::compute(text[1].as_bytes());
                println!("{} - {} [{:x}]", line, text[0], digest);
            }
        }
        if delay > 0 {
            thread::sleep(pause);
        }
    }
}
