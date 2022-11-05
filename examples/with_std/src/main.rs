fn main() {
    let parse: u32 = strp::parse!("{}");
    println!("{parse}");

    let try_parse: Result<u32, _> = strp::try_parse!("{}");
    println!("{try_parse:?}");

    let scan: (u32, u32) = strp::scan!("{}, {}");
    println!("{scan:?}");

    let try_scan: Result<(u32, u32), _> = strp::try_scan!("{}, {}");
    println!("{try_scan:?}");

    let (first, second): (String, String) = strp::scan!("first: {} second: {}");
    println!("first: {first}, second: {second}");
}
