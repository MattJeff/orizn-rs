# orizn

Official Rust SDK for the [Orizn Visa API](https://visa.orizn.app).

Check visa requirements for **40,027 passport-destination pairs** in **15 languages**.

## Install

```toml
[dependencies]
orizn = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick start

```rust
use orizn::Orizn;

#[tokio::main]
async fn main() -> Result<(), orizn::Error> {
    // No API key needed for quick checks
    let client = Orizn::new();
    let result = client.check("FRA", "JPN").await?;
    println!("{}", result.requirement); // "visa_free"

    // Full details (set ORIZN_API_KEY env or use with_key)
    let client = Orizn::with_key("your-key");
    let visa = client.get_visa("USA", "CHN", "en").await?;
    println!("{:?}", visa.documents_required);
    Ok(())
}
```

## Feedback

Building a travel agent or visa tool? We'd love to hear what you're building.

→ **api@orizn.app** — Feature requests, partnerships, and questions welcome.

## Links

- [Get free API key](https://visa.orizn.app)
- [API Docs](https://visa.orizn.app/visa-api/dashboard/docs)
- [docs.rs](https://docs.rs/orizn)
