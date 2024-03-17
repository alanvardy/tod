use colored::*;

pub fn green_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).green().to_string()
}

pub fn red_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).red().to_string()
}

pub fn cyan_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).bright_cyan().to_string()
}

pub fn purple_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).purple().to_string()
}

pub fn blue_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).blue().to_string()
}

pub fn yellow_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).yellow().to_string()
}

pub fn debug_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).bright_blue().on_yellow().to_string()
}

pub fn normal_string(str: &str) -> String {
    String::from(str).normal().to_string()
}
