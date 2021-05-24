use ansi_term::Colour::{Blue, Green};
use async_std::task;
use html5ever::tokenizer::{
    BufferQueue, CharacterTokens, Tag, TagKind, TagToken, Token, TokenSink, TokenSinkResult,
    Tokenizer, TokenizerOpts,
};
use std::borrow::Borrow;
use std::env;
use url::{ParseError, Url};

type CrawlResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

type BoxFuture = std::pin::Pin<Box<dyn std::future::Future<Output = CrawlResult> + Send>>;

#[derive(Default, Debug)]
struct LinkQueue {
    links: Vec<(String, String)>,
    a_href: Option<String>,
    a_text: Option<String>,
}

impl TokenSink for &mut LinkQueue {
    type Handle = ();

    // <a href="link">some text</a>
    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            TagToken(
                ref
                tag
                @
                Tag {
                    kind: TagKind::StartTag,
                    ..
                },
            ) => {
                if tag.name.as_ref() == "a" {
                    for attribute in tag.attrs.iter() {
                        if attribute.name.local.as_ref() == "href" {
                            let url_str: &[u8] = attribute.value.borrow();
                            self.a_href = Some(String::from_utf8_lossy(url_str).into_owned());
                            self.a_text = Some(String::new());
                            // self.links
                            //     .push(String::from_utf8_lossy(url_str).into_owned());
                        }
                    }
                }
            }
            TagToken(
                ref
                tag
                @
                Tag {
                    kind: TagKind::EndTag,
                    ..
                },
            ) => {
                if tag.name.as_ref() == "a" {
                    self.links
                        .push((self.a_text.take().unwrap(), self.a_href.take().unwrap()));
                }
            }
            CharacterTokens(string) => {
                if let Some(a_text) = self.a_text.as_mut() {
                    a_text.push_str(&string);
                }
            }
            _ => {}
        }

        TokenSinkResult::Continue
    }
}

pub fn get_links(url: &Url, page: String) -> Vec<(String, Url)> {
    let mut domain_url = url.clone();
    domain_url.set_path("");
    domain_url.set_query(None);

    let mut queue = LinkQueue::default();
    let mut tokenizer = Tokenizer::new(&mut queue, TokenizerOpts::default());
    let mut buffer = BufferQueue::new();
    buffer.push_back(page.into());
    let _ = tokenizer.feed(&mut buffer);

    queue
        .links
        .iter()
        .map(|(text, href)| match Url::parse(href) {
            Err(ParseError::RelativeUrlWithoutBase) => {
                (text.to_owned(), domain_url.join(href).unwrap())
            }
            Err(_) => panic!("Malformed link found: {}", href),
            Ok(url) => (text.to_owned(), url),
        })
        .collect()
}

fn box_crawl(pages: Vec<(String, Url)>, current: u8, max: u8) -> BoxFuture {
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
            //println!("\t Getting: {}", url);

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
