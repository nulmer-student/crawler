mod compile;
mod data;
mod interface;
mod intern;
mod loops;
mod pattern;

use std::sync::Arc;

use crawler::run;
use crawler::interface::*;
use interface::FindVectorSI;

fn main() {
    // Run the crawler with the SI interface
    let interface: AnyInterface = Arc::new(FindVectorSI {});
    run(interface);
}
