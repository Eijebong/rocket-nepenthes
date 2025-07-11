Rocket nepenthes
================

This is a set of utility to use with the rocket framework to help with
generating garbage to feed to AI data crawlers.

## Usage

### Route

For this to work, you need to mount `rocket_nepenthes::nepenthes` on `/` like this:

```rust
rocket::build()
    .mount("/", routes![rocket_nepenthes::nepenthes])
```

This is mandatory for the link maze to work and for the fairing to be able to redirect requests to it.

### Fairing

The default fairing will automatically detect two things:

1. User agents containing gpt
1. `?v=1`

If any of those conditions match, then it'll rewrite the route to `/nepenthes`.
You can install the fairing like this:

```rust
rocket::build()
    .attach(NepenthesFairing::default())
```

This behavior is configurable with the following:

```rust
fn should_nepenthes(r: &Request<'r>) -> bool {
    // For the generated links to short circuit the rest, you should keep this
    if r.query_value("v").unwrap_or(Ok("0")) == Ok("1") {
        return true;
    }

    // Do whatever here

    false
}

rocket::build()
    .attach(NepenthesFairing::new(should_nepenthes))
```

### Responder

If you have specific routes you want to protect based on application settings, you can use the `MaybeNepenthes` responder.

Example:

```rust
#[rocket::get("/?<date..>")]
fn index(date: ParsedDate) -> MaybeNepenthes<String> {
    if date < NEPENTHES_THRESHOLD {
        return MaybeNepenthes::Yes
    }

    MaybeNepenthes::No("you're ok".to_string())
}
```
