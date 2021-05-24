use ansi_term::Colour::{Blue, Green};
use async_std::task;
use crate::parse::get_links;
use url::{ Url};

pub type CrawlResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

pub type BoxFuture = std::pin::Pin<Box<dyn std::future::Future<Output = CrawlResult> + Send>>;

pub fn box_crawl(pages: Vec<(String, Url)>, current: u8, max: u8) -> BoxFuture {
    Box::pin(crawl(pages, current, max))
}

async fn crawl(pages: Vec<(String, Url)>, current: u8, max: u8) -> CrawlResult {
    println!("Current Depth: {}, Max Depth: {}", current, max);

    if current > max {
        println!("Reached Max Depth");
        return Ok(());
    }

    let mut tasks = vec![];

    for p in pages {
        println!("Crawling: {}", p.1);
        let task = task::spawn(async move {

            let mut res = surf::get(&p.1).await?;
            let body = res.body_string().await?;

            let links = get_links(&p.1, body);

            for (t, l) in &links {
                println!(
                    "\t Following link on {} with text {} to ==> {}",
                    Blue.bold().underline().paint(p.1.as_str()),
                    Green.bold().paint(t),
                    Blue.bold().underline().paint(l.as_str())
                );
            }
            box_crawl(links, current + 1, max).await
        });
        tasks.push(task);
    }

    for task in tasks.into_iter() {
        task.await?;
    }

    Ok(())
}
