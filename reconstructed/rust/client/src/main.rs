fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();
    if let Some(result) = garlic_internal_network::run_job_cli(&args) {
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    match garlic_internal_network::run(&args) {
        Ok(value) if args.first() == Some(&"login") => println!(
            "logged_in={} principal={}",
            value["logged_in"].as_str().unwrap_or(""),
            value["principal"]
        ),
        Ok(value) if args.first() == Some(&"logout") => {
            println!("logged_out={}", value["logged_out"].as_str().unwrap_or(""))
        }
        Ok(value) => println!("{value}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
