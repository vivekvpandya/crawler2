use url::{ParseError, Url};
use html5ever::tokenizer::{
    BufferQueue, CharacterTokens, Tag, TagKind, TagToken, Token, TokenSink, TokenSinkResult,
    Tokenizer, TokenizerOpts,
};
use std::borrow::Borrow;

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
