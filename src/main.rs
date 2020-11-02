use std::env;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod config;
mod params;
mod projects;
mod request;

fn main() {
    let params: params::Params = params::get_params_from_args(env::args());
    let config: config::Config = config::get_or_create_token_file();

    match params.command.as_str() {
        "list" => projects::list(config),
        "add" => projects::add(params, config),
        _ => {
            let (url, body) = request::build_request(params, config);

            request::do_request(&url, body);
        }
    }
}
