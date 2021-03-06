pub struct Params {
    pub command: String,
    pub text: String,
}

impl Params {
    pub fn new(args: std::env::Args) -> Params {
        let mut text = String::new();
        let mut command = String::new();
        for (index, arg) in args.enumerate() {
            match index {
                0 => (),
                1 => command.push_str(&arg),
                2 => text.push_str(&arg),
                num if num > 2 => {
                    text.push(' ');
                    text.push_str(&arg);
                }
                _ => (),
            }
        }

        Params { command, text }
    }
}
