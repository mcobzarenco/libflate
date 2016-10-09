extern crate clap;
extern crate libflate;

use std::io;
use std::io::Read;
use std::io::Write;
use std::fs;
use std::process;
use clap::App;
use clap::Arg;
use clap::SubCommand;
use libflate::gzip;

fn main() {
    let matches = App::new("deflate")
        .arg(Arg::with_name("INPUT")
            .short("i")
            .long("input")
            .value_name("FILE")
            .takes_value(true)
            .default_value("-"))
        .arg(Arg::with_name("OUTPUT")
            .short("o")
            .long("output")
            .value_name("FILE")
            .takes_value(true)
            .default_value("-"))
        .arg(Arg::with_name("VERBOSE").short("v").long("verbose"))
        .subcommand(SubCommand::with_name("copy"))
        .subcommand(SubCommand::with_name("bit-read"))
        .subcommand(SubCommand::with_name("byte-read").arg(Arg::with_name("UNIT")
            .short("u")
            .long("unit")
            .takes_value(true)
            .default_value("1")))
        .subcommand(SubCommand::with_name("gzip-decode"))
        .subcommand(SubCommand::with_name("gzip-encode"))
        .get_matches();

    let input_filename = matches.value_of("INPUT").unwrap();
    let input: Box<io::Read> = if input_filename == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open(input_filename)
            .expect(&format!("Can't open file: {}", input_filename)))
    };
    let mut input = io::BufReader::new(input);

    let output_filename = matches.value_of("OUTPUT").unwrap();
    let output: Box<io::Write> = if output_filename == "-" {
        Box::new(io::stdout())
    } else if output_filename == "/dev/null" {
        Box::new(io::sink())
    } else {
        Box::new(fs::File::create(output_filename)
            .expect(&format!("Can't create file: {}", output_filename)))
    };
    let mut output = io::BufWriter::new(output);

    let verbose = matches.is_present("VERBOSE");
    if let Some(_matches) = matches.subcommand_matches("copy") {
        io::copy(&mut input, &mut output).expect("Coyping failed");
    } else if let Some(_matches) = matches.subcommand_matches("bit-read") {
        let mut reader = libflate::bit::BitReader::new(input);
        let mut count = 0;
        while let Ok(_) = reader.read_bit() {
            count += 1;
        }
        println!("COUNT: {}", count);
    } else if let Some(matches) = matches.subcommand_matches("byte-read") {
        let unit = matches.value_of("UNIT").and_then(|x| x.parse::<usize>().ok()).unwrap();
        let mut buf = vec![0; unit];
        let mut reader = input;
        let mut count = 0;
        while let Ok(size) = reader.read(&mut buf) {
            if size == 0 {
                break;
            }
            count += size;
        }
        println!("COUNT: {}", count);
    } else if let Some(_matches) = matches.subcommand_matches("gzip-decode") {
        let mut decoder = gzip::Decoder::new(input);
        if verbose {
            let _ = writeln!(&mut io::stderr(),
                             "HEADER: {:?}",
                             decoder.header().expect("Read GZIP header
                             failed"));
        }
        io::copy(&mut decoder, &mut output).expect("Decoding GZIP stream failed");
        if verbose {
            let (_, _, trailer) = decoder.finish().unwrap();
            let _ = writeln!(&mut io::stderr(), "TRAILER: {:?}", trailer);
        }
    } else if let Some(_matches) = matches.subcommand_matches("gzip-encode") {
        let mut encoder = gzip::Encoder::new(output);
        io::copy(&mut input, &mut encoder).expect("Encoding GZIP stream failed");
    } else {
        println!("{}", matches.usage());
        process::exit(1);
    }
}
