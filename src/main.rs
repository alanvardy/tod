use std::env;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod config;
mod params;
mod projects;
mod request;

fn main() {
    let params: params::Params = params::Params::new(env::args());
    let config: config::Config = config::get_or_create_config_file();

    match params.command.as_str() {
        "list" => projects::list(config),
        "add" => projects::add(config, params),
        "remove" => projects::remove(config, params),
        _ => request::Request::new(params, config).perform(),
    }
    println!(" ");
}
