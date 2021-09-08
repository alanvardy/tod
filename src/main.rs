use std::env;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod config;
mod params;
mod projects;
mod request;

fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let params: params::Params = params::Params::new(args);
    let config: config::Config = config::get_or_create();

    match params.command.as_str() {
        "--list" | "-l" => projects::list(config),
        "--add" | "-a" => projects::add(config, params).save(),
        "--remove" | "-r" => projects::remove(config, params).save(),
        _ => request::Request::new(params, config).perform(),
    };
}
