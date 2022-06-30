
fn main_worker(input: &String, output: &String, resume: bool) {
    println!("{}, {}, {}", input, output, resume)
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let args_c: Vec<String> = args.collect();

    let input = args_c.get(1);
    if input.is_none() {
        println!("No input");
        return
    }

    let output = args_c.get(2);
    if output.is_none() {
        println!("No output");
        return
    }

    let resume = !args.find(|p| p.eq("--resume")).is_none();
    main_worker(input.unwrap(), output.unwrap(), resume);
}
