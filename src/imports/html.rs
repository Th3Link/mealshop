//! A bounded token stream, not a browser or DOM: only text and inline JSON-LD are collected.
use html5ever::{
    tendril::StrTendril,
    tokenizer::{
        BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, states::RawKind,
    },
};
use std::cell::RefCell;
#[derive(Default)]
pub(super) struct Data {
    pub(super) language: Option<String>,
    pub(super) scripts: Vec<String>,
    pub(super) text: String,
    script: Option<(bool, String)>,
    ignored: Vec<String>,
    raw_ignored: Option<String>,
    pub(super) overflow: bool,
}
struct Sink {
    data: RefCell<Data>,
    text: bool,
}
impl TokenSink for Sink {
    type Handle = ();
    fn process_token(&self, token: Token, _: u64) -> TokenSinkResult<()> {
        let mut data = self.data.borrow_mut();
        match token {
            Token::TagToken(tag) => {
                let name = tag.name.as_ref();
                if tag.kind == TagKind::EndTag {
                    if name == "script" {
                        if let Some((true, raw)) = data.script.take() {
                            if data.scripts.len() < 32 {
                                data.scripts.push(raw);
                            } else {
                                data.overflow = true;
                            }
                        }
                        data.script = None;
                    }
                    if data.raw_ignored.as_deref() == Some(name) {
                        data.raw_ignored = None;
                    }
                    if data.ignored.last().is_some_and(|n| n == name) {
                        data.ignored.pop();
                    }
                } else {
                    if name == "html" && data.language.is_none() {
                        data.language = tag
                            .attrs
                            .iter()
                            .find(|a| a.name.local.as_ref() == "lang")
                            .map(|a| a.value.to_string());
                    }
                    if name == "script" {
                        let wanted = !self.text
                            && data.ignored.is_empty()
                            && tag.attrs.iter().any(|a| {
                                a.name.local.as_ref() == "type"
                                    && a.value.split(';').next().is_some_and(|t| {
                                        t.trim().eq_ignore_ascii_case("application/ld+json")
                                    })
                            });
                        data.script = Some((wanted, String::new()));
                        return TokenSinkResult::RawData(RawKind::ScriptData);
                    }
                    if matches!(name, "style" | "xmp" | "iframe" | "noembed" | "noframes") {
                        data.raw_ignored = Some(name.to_owned());
                        return TokenSinkResult::RawData(RawKind::Rawtext);
                    }
                    if matches!(name, "template" | "noscript") {
                        if data.ignored.len() < 32 {
                            data.ignored.push(name.to_owned());
                        } else {
                            data.overflow = true;
                        }
                    }
                    if matches!(name, "title" | "textarea") {
                        return TokenSinkResult::RawData(RawKind::Rcdata);
                    }
                }
                if self.text && data.ignored.is_empty() && data.raw_ignored.is_none() {
                    data.text.push(' ');
                }
            }
            Token::CharacterTokens(text) => {
                if let Some((wanted, raw)) = &mut data.script {
                    if *wanted {
                        raw.push_str(&text);
                    }
                } else if self.text && data.ignored.is_empty() && data.raw_ignored.is_none() {
                    data.text.push_str(&text);
                }
            }
            Token::NullCharacterToken
                if self.text
                    && data.script.is_none()
                    && data.ignored.is_empty()
                    && data.raw_ignored.is_none() =>
            {
                data.text.push('\u{fffd}');
            }
            _ => {}
        }
        TokenSinkResult::Continue
    }
}
pub(super) fn scan(raw: &str, text: bool) -> Data {
    let input = BufferQueue::default();
    input.push_back(StrTendril::from_slice(raw));
    let tokenizer = Tokenizer::new(
        Sink {
            data: RefCell::new(Data::default()),
            text,
        },
        html5ever::tokenizer::TokenizerOpts::default(),
    );
    let _ = tokenizer.feed(&input);
    tokenizer.end();
    tokenizer.sink.data.into_inner()
}
