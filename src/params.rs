pub struct Params {
    pub project: String,
    pub text: String,
}

pub fn get_params_from_args(args: std::env::Args) -> Params {
    let mut text = String::new();
    let mut project = String::new();
    for (index, arg) in args.enumerate() {
        match index {
            0 => (),
            1 => project.push_str(&arg),
            2 => text.push_str(&arg),
            num if num > 2 => {
                text.push_str(" ");
                text.push_str(&arg);
            }
            _ => (),
        }
    }

    Params { project, text }
}
