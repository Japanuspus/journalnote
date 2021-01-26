use journalnote;

fn main() {
    let message = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    journalnote::enter_message(&message);
}

