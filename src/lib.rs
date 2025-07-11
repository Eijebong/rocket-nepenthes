use std::{borrow::Cow, sync::LazyLock};

use itertools::Itertools;
use markovish::Chain;
use rand::{Rng, RngCore, SeedableRng, rngs::SmallRng};
use rocket::{
    Data, Request,
    fairing::{Fairing, Info, Kind},
    http::{hyper::header, uri::Origin},
    response::{Responder, content::RawHtml},
};
use rust_embed::Embed;
use urlencoding::encode;

#[derive(Embed)]
#[folder = "data"]
struct Embedded;
static CHAIN: LazyLock<Chain> = LazyLock::new(|| {
    Chain::from_text(
        &String::from_utf8_lossy(&Embedded::get("corpus.txt").unwrap().data.to_vec()).to_string(),
    )
    .unwrap()
});

/// This is a rocker `Responder` wrapper. If the `No` variant is used then the response will be
/// sent as is. If the `Yes` variant is used on the other hand, the /nepenthes route content will
/// be returned.
pub enum MaybeNepenthes<T> {
    Yes,
    No(T),
}

impl<'r, 'o: 'r, T: Responder<'r, 'o>> Responder<'r, 'o> for MaybeNepenthes<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        match self {
            MaybeNepenthes::Yes => nepenthes().respond_to(request),
            MaybeNepenthes::No(responder) => responder.respond_to(request),
        }
    }
}

/// This fairing will check for all incoming requests that the user agent isn't a bad bot's one or
/// that it doesn't contain `?v=1`. If either of these condition is true, then the request's path
/// is rewritten to `/nepenthes`.
pub struct NepenthesFairing;

#[rocket::async_trait]
impl Fairing for NepenthesFairing {
    fn info(&self) -> Info {
        Info {
            name: "Nepenthes",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, r: &mut Request<'_>, _data: &mut Data<'_>) -> () {
        fn should_nepenthes(r: &Request<'_>) -> bool {
            if r.query_value("v").unwrap_or(Ok("0")) == Ok("1") {
                log::info!("v param present");
                return true;
            }

            if let Some(user_agent) = r.headers().get_one(header::USER_AGENT.as_str()) {
                let user_agent = user_agent.to_lowercase();

                // Bad user agents
                if user_agent.contains("gpt") {
                    return true;
                }
            }

            false
        }

        if should_nepenthes(r) {
            r.set_uri(Origin::try_from("/nepenthes").unwrap());
        }
    }
}

#[rocket::get("/nepenthes")]
pub fn nepenthes() -> RawHtml<String> {
    let mut rng = SmallRng::from_os_rng();
    fn get_garbage(rng: &mut SmallRng) -> String {
        let tokens = CHAIN.generate_str(rng, 300).unwrap();
        tokens
            .iter()
            .filter(|token| **token != " ")
            .map(|word| maybe_block(word, rng))
            .join(" ")
    }

    let garbage = get_garbage(&mut rng);
    let garbage2 = get_garbage(&mut rng);
    let garbage3 = get_garbage(&mut rng);

    RawHtml(format!(
        "<html><body><div>{}</div><p>{}</p><p>{}</p></body></html>",
        garbage, garbage2, garbage3
    ))
}

pub fn maybe_block<'a>(word: &'a str, rng: &mut SmallRng) -> Cow<'a, str> {
    if rng.random_bool(0.02) {
        let number = rng.next_u64();
        return Cow::from(format!(
            "<a href=\"/{}?v=1&t={}\">{}</a>",
            encode(word),
            number,
            word
        ));
    }

    if rng.random_bool(0.01) {
        return Cow::from(format!("<span title=\"important\">{}</span>", word));
    }

    Cow::Borrowed(word)
}
