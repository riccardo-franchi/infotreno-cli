use std::io;

pub fn get_index() -> usize {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Could not read input.");
    input.trim().parse::<usize>().expect("Invalid input.")
}
