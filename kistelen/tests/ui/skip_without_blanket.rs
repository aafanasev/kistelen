use kistelen::Secret;

// `skip` only means something when a wider rule would otherwise mask the
// field. Without one it reads as protection that is not there.
#[derive(Secret)]
struct Account {
    #[secret(skip)]
    password: String,
}

fn main() {}
