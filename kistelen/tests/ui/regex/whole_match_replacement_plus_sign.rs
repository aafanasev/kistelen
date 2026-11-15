use kistelen::Secret;

// `+0` parses as group 0, the whole match.
#[derive(Secret)]
struct Card {
    #[secret(search = r"(\d+)", replacement = "${+0}")]
    number: String,
}

fn main() {}
