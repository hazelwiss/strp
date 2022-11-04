fn main() {
    let hello: String = strp::parse!("input: {}"); // Read from stdin.
    println!("the user inputet \"{hello}\"");
}
