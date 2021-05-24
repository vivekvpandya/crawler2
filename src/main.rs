use std::env;
use url::{ Url};
use async_std::task;

use crate::crawl::CrawlResult;
use crate::crawl::box_crawl;

mod crawl;
mod parse;

fn main() -> CrawlResult {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        task::block_on(async {
            box_crawl(vec![(args[1].clone(), Url::parse(&args[1]).unwrap())], 1, 2).await
        })
    } else {
        println!("Please provide a URL.");
        Ok(())
    }
}
