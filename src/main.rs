use std::io::{self, BufRead};
// use std::fs::File;
// use std::io::prelude::*;
use std::process::exit;

mod score;
mod console;
mod render;
mod ansi;

fn run<'a>(stdin_lines: Vec<String>) -> Result<&'a str, &'a str> {
    let console = console::Console::new()?;
    console.configure();

    let query = &['f', 'o', 'o', 'b', 'a', 'r'];

    let mut scores: Vec<score::Score> = stdin_lines
        .iter()
        .filter_map(|line| score::calculate_score(line, query))
        .collect();
    scores.sort_unstable_by(|a, b| a.points.cmp(&b.points));

    let renderer = render::Renderer::new(&scores, &console, query);

    println!("{:?}", console);
    renderer.render();

    // loop {}
    console.restore();
    Ok("foo")
}

fn main() {
    let stdin = io::stdin();
    let mut stdin_lines: Vec<String> =
        stdin.lock().lines().filter_map(|line| line.ok()).collect();

    stdin_lines.sort_unstable();
    stdin_lines.dedup();
    // let mut f = File::create("output.txt").unwrap();
    // for line in &lines {
    //     let _r = f.write_all(line.line.as_bytes());
    //     let _r = f.write_all(b"\n");
    // }

    // println!(
    //     "total lines {}, total matches: {}",
    //     stdin_lines.len(),
    //     lines.len()
    // );
    // println!("{:?}", lines);

    match run(stdin_lines) {
        // Ok(l) => println!("{}", l),
        Ok(_) => (),
        Err(_) => exit(1),
    };
}
