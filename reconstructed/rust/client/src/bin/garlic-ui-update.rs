fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match garlic_internal_network::run_ui_update(
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
    ) {
        Ok(value) => println!("{value}"),
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}
