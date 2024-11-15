use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i')]
    clock_in: Option<String>,

    #[arg(short = 'o')]
    clock_out: Option<String>,

    #[arg(num_args = 0..=2)]
    times: Vec<String>,
}

fn main() {
    let cli = Args::parse();

    match (cli.clock_in, cli.clock_out, cli.times.len()) {
        (Some(time), None, 0) => println!("checked in at {}", time),
        (None, Some(time), 0) => println!("checked out at {}", time),
        (None, None, 2) => println!("recorded time {} to {}", cli.times[0], cli.times[1]),
        (None, None, 0) => println!("hello"),
        _ => println!("Invalid combination of arguments. Use -h for help."),
    }
}
