#[derive(Debug, Eq, PartialEq)]
pub struct Params {
    /// The first argument
    pub command: String,
    /// The remaining arguments with spaces between them
    pub text: String,
}

impl Params {
    pub fn new(args: Vec<String>) -> Params {
        let mut iterator = args.iter();
        let command = iterator.next().unwrap().to_owned();
        let text = iterator
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
            .join(" ");

        Params { command, text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_can_create_params() {
        let args = vec![
            String::from("one"),
            String::from("two"),
            String::from("three"),
            String::from("four"),
        ];

        let params = Params {
            command: String::from("one"),
            text: String::from("two three four"),
        };
        assert_eq!(Params::new(args), params)
    }
}
