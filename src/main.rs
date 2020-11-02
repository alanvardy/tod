use std::env;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod config;
mod request;
mod params;


fn main() {
    let params: params::Params = params::get_params_from_args(env::args());
    let config: config::Config = config::get_or_create_token_file();
    let (url, body) = request::build_request(params, config);

    request::do_request(&url, body);
}
