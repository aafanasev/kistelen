use kistelen::Secret;

#[derive(Secret)]
struct Person {
    #[secret(search = r"(?P<name>\w+) \w+", replacement = "${nme} ***")]
    full_name: String,
}

fn main() {}
