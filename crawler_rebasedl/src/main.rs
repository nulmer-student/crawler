mod compile;
mod interface;
mod data;
mod output_parser;

use std::sync::Arc;

use crawler::run;
use crawler::interface::*;
use interface::RebaseDL;

fn main() {
    let interface: AnyInterface = Arc::new(RebaseDL {});
    run(interface);
}
