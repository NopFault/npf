use regex::Regex;
use reqwest::header::USER_AGENT;
use std::fs::read_to_string;
use std::path::Path;
use std::process;
use std::{thread, time};

fn help() {
    println!("Usage: ");
    println!("  --host");
    println!(" Provide a full fuzzing address (https://[NPF].site.com || http://site.com/[NPF])\n");
    println!("  --word");
    println!(" Provide a fuzzing values file located (words.txt)\n");
    println!("  --hide 404,403");
    println!(" Hide 404 and 403 responses.\n");
    println!("  --head true");
    println!(" Make HEAD request instead of GET.\n");
    println!("  --delay <u64>");
    println!(" Delay for some time for each request\n");
    println!(
        "  --ua 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/120.0'"
    );
    println!(" Set custom user agent\n");
    println!("  --randomua true");
    println!(" I have a pool of mixed random user agents");
    println!("  --titles true");
    println!(" Show page titles");
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

fn something_in_code(code: String) -> bool {
    return code.split("<?").count() > 1
        || code.split("api_key").count() > 1
        || code.split("api-key").count() > 1
        || code.split("eval(").count() > 1;
}

fn get_title(code: &String) -> String {
    let re = Regex::new(r"<title>(.*?)<\/title>").unwrap();

    for capture in re.captures_iter(code.as_str()) {
        if let Some(title) = capture.get(1) {
            return title.as_str().to_string();
        }
    }
    return "".to_string();
}

fn get_random_ua() -> String {
    let ua_list: Vec<&str> = vec![
        "Mozilla/5.0 (Linux; Android 13; SM-S908B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Mobile Safari/537.36",
        "Mozilla/5.0 (Linux; Android 13; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Mobile Safari/537.36",
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Mobile Safari/537.36",
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Mobile Safari/537.36,gzip(gfe)",
        "Mozilla/5.0 (Linux; Android 11; moto g power (2021)) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Mobile Safari/537.36",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 12_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/12.0 Mobile/15E148 Safari/604.1",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 11_0 like Mac OS X) AppleWebKit/604.1.38 (KHTML, like Gecko) Version/11.0 Mobile/15A372 Safari/604.1",
        "Mozilla/5.0 (iPhone9,4; U; CPU iPhone OS 10_0_1 like Mac OS X) AppleWebKit/602.1.50 (KHTML, like Gecko) Version/10.0 Mobile/14A403 Safari/602.1",
        "Mozilla/5.0 (Windows Phone 10.0; Android 6.0.1; Microsoft; RM-1152) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/52.0.2743.116 Mobile Safari/537.36 Edge/15.15254",
        "Mozilla/5.0 (Linux; Android 9; AFTWMST22 Build/PS7233; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/88.0.4324.152 Mobile Safari/537.36",
        "Mozilla/5.0 (Windows NT 6.1; WOW64; Trident/7.0; MAAU; rv:11.0) like Gecko",
        "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/42.0.2311.152 Safari/537.36",
        "Mozilla/4.0 (compatible; MSIE 8.0; Windows NT 6.1; Trident/4.0; SLCC2; .NET CLR 2.0.50727; .NET CLR 3.5.30729; .NET CLR 3.0.30729; Media Center PC 6.0; .NET4.0C; .NET4.0E)",
        "Mozilla/5.0 (Windows NT 6.1; WOW64; rv:35.0) Gecko/20100101 Firefox/35.0",
        "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/43.0.2357.132 Safari/537.36",
        "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/42.0.2311.90 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_9_2) AppleWebKit/537.74.9 (KHTML, like Gecko) Version/7.0.2 Safari/537.74.9",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_8_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/44.0.2403.157 Safari/537.36",
        "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/44.0.2403.155 Safari/537.36",
        "Mozilla/5.0 (iPad; CPU OS 7_0_2 like Mac OS X) AppleWebKit/537.51.1 (KHTML, like Gecko) Version/7.0 Mobile/11A501 Safari/9537.53",
        "Mozilla/5.0 (Windows NT 6.3; WOW64; Trident/7.0; Touch; MAARJS; rv:11.0) like Gecko",
    ];

    let index = (rand::random::<f32>() * ua_list.len() as f32).floor() as usize;

    return ua_list[index].to_string();
}

#[tokio::main]
async fn main() {
    let arguments = std::env::args();
    let arguments = arguments::parse(arguments).unwrap();

    let silent = arguments.get::<String>("hide").unwrap_or(String::new());
    let words = arguments.get::<String>("words").unwrap_or(String::new());
    let host = arguments.get::<String>("host").unwrap_or(String::new());
    let head = arguments.get::<bool>("head").unwrap_or(false);
    let delay = arguments.get::<u64>("delay").unwrap_or(0);
    let mut ua = arguments.get::<String>("ua").unwrap_or(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/120.0"
            .to_string(),
    );
    let ua_pool = arguments.get::<bool>("randomua").unwrap_or(false);
    let show_titles = arguments.get::<bool>("titles").unwrap_or(false);

    if host.chars().count() <= 7 || words.chars().count() <= 0 {
        help();
        process::exit(1);
    }

    let pause = time::Duration::from_secs(delay);
    for line in read_lines(words) {
        if line.chars().count() > 0 {
            let url = host.replace("[NPF]", &line);

            if ua_pool == true {
                ua = get_random_ua();
            }
            let text = get_contents(url, head, &ua).await;

            if silent.trim().chars().count() > 2 {
                let silencer: Vec<String> = silent
                    .clone()
                    .split(",")
                    .map(|code| {
                        reqwest::StatusCode::from_u16(code.parse::<u16>().unwrap_or(0))
                            .unwrap()
                            .to_string()
                    })
                    .collect();

                if silencer.len() > 0 {
                    if !silencer.contains(&text[0]) {
                        let digest = md5::compute(text[1].as_bytes());
                        if show_titles {
                            println!("[{}]:", get_title(&text[1]));
                        }
                        println!(
                            "{} - {}, [{:x}], Length: {}",
                            line,
                            text[0],
                            digest,
                            text[1].chars().count(),
                        );
                        if show_titles {
                            println!("");
                        }
                    }
                }
            } else {
                let digest = md5::compute(text[1].as_bytes());

                if show_titles {
                    println!("[{}]:", get_title(&text[1]));
                }
                println!(
                    "{} - {}, [{:x}], Length: {}",
                    line,
                    text[0],
                    digest,
                    text[1].chars().count()
                );
                if show_titles {
                    println!("");
                }
            }

            let code: String = text[1].to_string();
            if something_in_code(code) {
                println!("Find something interesting in source code\n\n")
            }
        }
        if delay > 0 {
            thread::sleep(pause);
        }
    }
}
