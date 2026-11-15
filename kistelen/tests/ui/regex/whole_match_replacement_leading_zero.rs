use kistelen::Secret;

// `00` parses as group 0, the whole match.
#[derive(Secret)]
struct Card {
    #[secret(search = r"(\d+)", replacement = "${00}")]
    number: String,
}

fn main() {}
