+++
draft = false
tags = ["rust", "oss", "deep-dive", "sqlx"]
title = "SQLx Compile Time Woes"
date = "2023-03-13T00:00:00-00:00"
description = "A curious case of climbing compile-times"
+++

# Someone on the internet...

> `sqlx` is really nice, but you definitely take a hit to compile times
>
> \- Random people on the internet

It's something that really resonated with me after heavily using `sqlx` at my old
job.
Even with a Ryzen 3700, `cargo check` times climbed from 5 to 10 to **20 seconds**, and
`cargo sqlx prepare`ing off a remote database was a good excuse to take a coffee
break. There's got to be something we can do...

<!--more-->

Before we dive into all that I'd like to have a quick aside to cover some `sqlx`
basics. Feel free to skip if you're already familiar with `sqlx`'s macros and
offline builds.

_Disclaimer: This whole post was done on `sqlx` v0.5-v0.6. Some parts are
already inaccurate with v0.7, and I'm sure that trend will continue_

## Primer: `sqlx` macros 101

One of `sqlx`'s main selling points is that it can perform compile-time query
checking against an actual database. That means that if you have some code like
so

```rust
use sqlx::{Result, SqliteConnection};

struct User {
    id: i64,
    name: String,
}

async fn get_user_by_id(db: &mut SqliteConnection, id: i64) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT id, name FROM User WHERE id = ?",
        id,
    )
    .fetch_one(db)
    .await
}
```

and compile with the `DATABASE_URL` env var set to your database URL then it will connect to the
database at compile time to verify that both the query is valid, and that the Rust
types match the database's returned types. If your query is invalid, or your
database types don't match their Rust counterparts then you'll end up with a
compile-time error.

```diff
11c11
<         "SELECT id, name FROM User WHERE id = ?",
---
>         "SELECT id, name FROM User WHERE id = ? AND NOT deleted",
```

{{< raw >}}
<pre tabindex="0"><code><b><span class="term-green">$</span></b> <span class="term-red">DATABASE_URL</span>=<span class="term-yellow">'sqlite:blog.db'</span> <span class="term-green">cargo</span> check
<b><span class="term-green">    Checking</span></b> sqlx_blog v0.1.0 (/.../sqlx_blog)
<b><span class="term-red">error</span>: error returned from database: (code: 1) no such column: deleted</b>
  <b><span class="term-blue">--> </span></b>src/lib.rs:9:5
   <b><span class="term-blue">|</span></b>
<b><span class="term-blue">9  | </span><span class="term-red">/ </span></b>    sqlx::query_as!(
<b><span class="term-blue">10 | </span><span class="term-red">| </span></b>        User,
<b><span class="term-blue">11 | </span><span class="term-red">| </span></b>        "SELECT id, name FROM User WHERE id = ? AND NOT deleted",
<b><span class="term-blue">12 | </span><span class="term-red">| </span></b>        id,
<b><span class="term-blue">13 | </span><span class="term-red">| </span></b>    )
   <b><span class="term-blue">| </span><span class="term-red">|_____^</span></b>
   <b><span class="term-blue">|</span></b>
   <b><span class="term-blue">= </span>note</b>: this error originates in the macro ...

<b><span class="term-red">error</span>:</b> could not compile `sqlx_blog` due to previous error
</code></pre>
{{< /raw >}}

Whoops. Forgot to add that column :)

Of course, requiring a database connection every time you build can be a hassle
(e.g. in CI), so `sqlx` also provides the option
to do offline builds. You just add the `"offline"` feature to `sqlx`, and now you
can prepare an `sqlx-data.json` file that describes all of your queries using
`cargo-sqlx` from
[`sqlx-cli`](https://crates.io/crates/sqlx-cli "Command-line utility for SQLx, the Rust SQL toolkit").

{{< raw >}}
<pre tabindex="0"><code><b><span class="term-green">$</span></b> <span class="term-red">DATABASE_URL</span>=<span class="term-yellow">'sqlite:blog.db'</span> <span class="term-green">cargo</span> sqlx prepare
   <b><span class="term-green">Compiling</span></b> sqlx_blog v0.1.0 (/.../sqlx_blog)
    <b><span class="term-green">Finished</span></b> dev [unoptimized + debuginfo] target(s) in 0.10s
query data written to `sqlx-data.json` in the current directory; please ...

<span class="term-green"><b>$</b> bat</span> --plain <u>sqlx-data.json</u>
{
  <span class="term-cyan">"db"</span>: <span class="term-yellow">"SQLite"</span>,
  <span class="term-cyan"><i>"{query_hash}"</i></span>: {
    <span class="term-cyan">"describe"</span>: {
      <span class="term-blue">...</span> <span class="term-gray">// A lot of information on the query</span>
    },
    <span class="term-cyan">"query"</span>: <span class="term-yellow">"SELECT id, name FROM User WHERE id = ?"</span>
  }
  <span class="term-blue">...</span> <span class="term-gray">// More entries for all other queries</span>
}
</code></pre>
{{< /raw >}}

Now your project can build using the information in `sqlx-data.json` instead of
needing to connect to a live database.

Connecting to databases at compile time is the kind of proc-macro (ab)use that people
usually bring up when talking about `sqlx`. It's equally beautiful and horrifying
:smile:.

## Peering through the looking glass

You can probably guess from this blog post... ya know... _existing and all_ that `sqlx` ended up being
the main culprit, but I really had no clue when I started this journey. All I
knew was that my old job's 60k+ sloc mono-crate code base was slowly spiraling into a very unhealthy dev-feedback
loop, and I was wanting **out**.

I was rooting through my usual toolbox of
[`cargo check --timings`](https://doc.rust-lang.org/stable/cargo/reference/timings.html "Reporting build timings"),
[`cargo-llvm-lines`](https://github.com/dtolnay/cargo-llvm-lines "Count lines of LLVM IR per generic function"),
and
[`summarize`](https://github.com/rust-lang/measureme/blob/master/summarize/README.md "A tool to produce a human readable summary of measureme profiling data")
to get a better idea of what was blowing up the `cargo check` times.
`cargo check --timings` showed that, unsurprisingly, it was the massive
mono-crate taking up all the time. `cargo-llvm-lines` pointed to a tossup
between large `sqlx` macros, `serde`'s
{{< hl_inline rust >}}#[derive(Deserialize, Serialize)]{{< /hl_inline >}}s, and
some large functions. Last, but _certainly not least_, `summarize` had something very
interesting to note (edited for narrow screens).

{{< raw >}}
<pre tabindex="0"><code><span class="term-green"><b>$</b> cargo</span> +nightly rustc -- -Z self-profile

<span class="term-green"><b>$</b> summarize</span> summarize <u><i>{redacted}</i>-<i>{pid}</i>.mm_profdata</u>
+--------------------------+-----------+-----------------+----------+
| Item                     | Self time | % of total time | Time     |
+--------------------------+-----------+-----------------+----------+
| expand_crate             | 14.75s    | 67.516          | 14.83s   |
+--------------------------+-----------+-----------------+----------+
| monomorphization_coll... | 1.11s     | 5.074           | 2.55s    |
+--------------------------+-----------+-----------------+----------+
| hir_lowering             | 419.93ms  | 1.923           | 419.93ms |
+--------------------------+-----------+-----------------+----------+
<span class="term-gray"><i>... many many elided rows</i></span>
</code></pre>
{{< /raw >}}

**67.5% of the time was taken up by `expand_crate`!** What even is `expand_crate`?
Well lucky for all of us living in the future this appears to now get reported
in the much more appropriate item: `expand_proc_macro`, which makes things pretty obvious (probably
thanks to
[this PR](https://github.com/rust-lang/rust/pull/95739 "self-profiler: record spans for proc-macro expansions")).
That's okay though, a quick `rg` on the compiler source at the time suggested
the same thing.

Well that seems to point a _reeeally_ big spotlight on `sqlx`, but what was
it doing that was taking up so much time? And then my eyes fell on that ~500 KiB
`sqlx-data.json` file...

<!-- TODO: use a shortcode for this -->
{{< raw >}}
<blockquote id="inner-mono">
{{< /raw >}}
_500 KiB isn't that big_

_It could be getting handled really poorly?_

_The JSON is pretty printed. Maybe if we compact it..._
{{< raw >}}
</blockquote>
{{< /raw >}}

{{< raw >}}<pre tabindex="0"><code>{{<
in_pre_hyperfine
    warmup=1 prepare="touch src/lib.rs" cmd="cargo check"
    mean=19.273 stddev=" 0.202" user=18.977 sys=0.542
    min=18.839 max=19.524
>}}

<span class="term-green"><b>$</b> mv</span> <u>sqlx-data.json</u> sqlx-data.json.pretty

<span class="term-green"><b>$</b> cat</span> <u>sqlx-data.json.pretty</u> | <span class="term-green">jq</span> -c > sqlx-data.json

{{<
in_pre_hyperfine
    warmup=1 prepare="touch src/lib.rs" cmd="cargo check"
    mean=14.965 stddev=" 0.294" user=14.685 sys=0.531
    min=14.449 max=15.368
>}}</code></pre>
{{< /raw >}}

_(Note: All timings are run on a laptop with an i5-1135G7 because I wiped my
desktop after it was having GPU issues :c)_

Several. Seconds. Faster.

A -22% change for something so simple is huge, but now that leaves the question of _why_ the JSON parsing is so slow. Even if `sqlx` was doing something really
bad like re-reading the whole file for each macro then there are still only a couple
hundred queries. **497 KiB * 203 queries** comes out to **98.5 MiB** which should be
nothing for an optimized JSON par- :facepalm:. Wait... this is a debug build... it won't be an optimized JSON parser.

# Down the rabbit hole

Now that we found the culprit we can try to fix things. `cargo` makes
it easy enough to change how we build dependencies. Why not set it to do an
optimized build for all proc-macro related bits? That should speed things up a
lot.

Consulting
[the docs](https://doc.rust-lang.org/cargo/reference/profiles.html#build-dependencies "Cargo Reference: build dependencies")
we see that we can do an optimized build for all proc-macros and their dependencies by adding

```toml
[profile.dev.build-override]
opt-level = 3
```

to our `Cargo.toml` file.

We're back to our pretty-printed `sqlx-data.json` now. How are we looking?

{{<
hyperfine
    warmup=1 prepare="touch src/lib.rs" cmd="cargo check"
    mean=" 7.021" stddev=" 0.077" user=6.696 sys=0.539
    min=" 6.887" max=" 7.176"
>}}

**12 seconds faster?** One word: _**dopamine**._

On top of that one of my coworkers pointed out that we get most of the benefit
still when
doing an optimized build of just `sqlx-macros` a la

```toml
[profile.dev.package.sqlx-macros]
opt-level = 3
```

## Pulchritudinous parsing

{{< raw >}}
<blockquote id="inner-mono">
{{< /raw >}}
_Bless you_

_Oh, I didn't sneeze. It actually means beautiful_

_Huh. Pretty ugly looking word for beautiful_
{{< raw >}}
</blockquote>
{{< /raw >}}

So we're done, right? Ship it, and all that? **No.** We may have put a nice band-aid
on things, but I'm not going to call it good here. Let's fix-up `sqlx`, so
that everyone can benefit from faster builds. Poking around the source for a bit
we find {{< hl_inline rust >}}DynQueryData::from_data_file(){{< /hl_inline >}} (edited for brevity).

```rust
// from sqlx-macros/src/query/data.rs

#[derive(serde::Deserialize)]
pub struct DynQueryData {
    #[serde(skip)]
    pub db_name: String,
    pub query: String,
    pub describe: serde_json::Value,
    #[serde(skip)]
    pub hash: String,
}

impl DynQueryData {
    /// Find and deserialize the data table for this query from a shared
    /// `sqlx-data.json` file. The expected structure is a JSON map keyed by
    /// the SHA-256 hash of queries in hex.
    pub fn from_data_file(path: &Path, query: &str) -> crate::Result<Self> {
        let this = serde_json::Deserializer::from_reader(BufReader::new(
            File::open(path).map_err(...)?
        ))
        .deserialize_map(DataFileVisitor {
            query,
            hash: hash_string(query),
        })?;

        // ...

        Ok(this)
    }
}

// lazily deserializes only the `QueryData` for the query we're looking for
struct DataFileVisitor<'a> {
    query: &'a str,
    hash: String,
}

impl<'de> Visitor<'de> for DataFileVisitor<'_> {
    type Value = DynQueryData;

    fn expecting(...) -> fmt::Result { ... }

    fn visit_map<A>(
        self,
        mut map: A
    ) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
    where
        A: MapAccess<'de>,
    {
        let mut db_name: Option<String> = None;

        let query_data = loop {
            // -- 8< -- Get db name and query info then break -- 8< --
        };

        // Serde expects us to consume the whole map; fortunately they've got a
        // convenient type to let us do just that
        while let Some(_) = map.next_entry::<IgnoredAny, IgnoredAny>()? {}

        Ok(query_data)
    }
}
```

It's a decent chunk of code, but we can see that they do, in fact, deserialize the
whole file for each query. They're smart about it and tell `serde` to ignore most
of the values,
but `serde_json` still has to parse the full thing to ensure it's valid JSON.

That leaves an easy fix though. The `sqlx-data.json` file shouldn't change while
we're compiling, so we can just deserialize the whole thing once upfront and
then pass out the data for each query as needed. Something like

```rust
static OFFLINE_DATA_CACHE: Lazy<Mutex<BTreeMap<PathBuf, OfflineData>>> =
    Lazy::default();

#[derive(serde::Deserialize)]
struct BaseQuery {
    query: String,
    describe: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct OfflineData {
    db: String,
    #[serde(flatten)]
    hash_to_query: BTreeMap<String, BaseQuery>,
}

impl OfflineData {
    fn get_query_from_hash(&self, hash: &str) -> Option<DynQueryData> {
        self.hash_to_query.get(hash).map(|base_query| DynQueryData {
            db_name: self.db.clone(),
            query: base_query.query.to_owned(),
            describe: base_query.describe.to_owned(),
            hash: hash.to_owned(),
        })
    }
}

#[derive(serde::Deserialize)]
pub struct DynQueryData { ... }

impl DynQueryData {
    pub fn from_data_file(path: &Path, query: &str) -> crate::Result<Self> {
        let query_data = {
            let mut cache = OFFLINE_DATA_CACHE
                .lock()
                .unwrap_or_else(/* reset the cache */);

            if !cache.contains_key(path) {
                let offline_data_contents = fs::read_to_string(path)
                    .map_err(...)?;
                let offline_data: OfflineData =
                    serde_json::from_str(&offline_data_contents)?;
                let _ = cache.insert(path.to_owned(), offline_data);
            }

            let offline_data = cache
                .get(path)
                .expect("Missing data should have just been added");

            let query_data = offline_data
                .get_query_from_hash(&hash_string(query);
                .ok_or_else(...)?;

            if query != query_data.query {
                return Err(/* hash collision error */);
            }

            query_data
        };

        // ...

        Ok(query_data)
    }
}
```

As you can see we deserialize all the `sqlx-data.json` data into an
{{< hl_inline rust >}}OFFLINE_DATA_CACHE{{< /hl_inline >}} which stores it in a
{{< hl_inline rust >}}BTreeMap<PathBuf, OfflineData>{{< /hl_inline >}}
 (A {{< hl_inline rust >}}BTreeMap{{< /hl_inline >}} is needed because there can actually be multiple `sqlx-data.json`
files in use, so the path maps to its deserialized data). From there we can just
build and return {{< hl_inline rust >}}DynQueryData{{< /hl_inline >}}s from {{< hl_inline rust >}}OfflineData{{< /hl_inline >}} on the fly. Not too bad, and we get to scrap
all the custom deserializer logic as a bonus.

_PR: [launchbadge/sqlx#1684](https://github.com/launchbadge/sqlx/pull/1684
"refactor: Keep parsed sqlx-data.json in a cache instead of reparsing")_


How's the time looking now?

_(Note: Still keeping the `build-override` from before)_

{{<
hyperfine
    warmup=1 prepare="touch src/lib.rs" cmd="cargo check"
    mean=" 5.614" stddev=" 0.064" user=5.349 sys=0.501
    min=" 5.489" max=" 5.739"
>}}

Over a full second shaved off from the ~7 seconds before!

## And you get a cache, and you get a cache

That covers the check times mentioned in the opening quote. What else was there?
Something about coffee?

> `cargo sqlx prepare`ing off a remote database was a good excuse to take a
> coffee break.

Oh yeah, preparing off remote databases! (Also I don't really drink coffee, but
I do have a dog to walk :) )

Diving back into the code yields us this snippet for preparing off a remote
database.

```rust
// from sqlx-macros/src/query/mod.rs

fn expand_from_db(
    input: QueryMacroInput,
    db_url: &str
) -> crate::Result<TokenStream> {
    let db_url = Url::parse(db_url)?;
    match db_url.scheme() {
        #[cfg(feature = "postgres")]
        "postgres" | "postgresql" => {
            let data = block_on(async {
                let mut conn = sqlx_core::postgres::PgConnection::connect(
                    db_url.as_str()
                ).await?;
                QueryData::from_db(&mut conn, &input.sql).await
            })?;

            expand_with_data(input, data, false)
        },

        #[cfg(not(feature = "postgres"))]
        "postgres" | "postgresql" => Err(
            "database URL has the scheme of a PostgreSQL database but the \
            `postgres` feature is not enabled".into()
        ),

        // -- 8< -- Same thing for other dbs. Soooo many `cfg`s -- 8< --

        item => Err(format!("Missing expansion needed for: {:?}", item).into())
    }
}
```

Very similar to the last section, but this time we're creating a fresh database connection for
each query macro instead of parsing a file. This is not cheap for remote databases! Quoting
[`sqlx`'s docs](https://docs.rs/sqlx/0.6.2/sqlx/pool/struct.Pool.html#1-overhead-of-opening-a-connection
"1. Overhead of Opening a Connection")

> **1. Overhead of Opening a Connection**
>
> Opening a database connection is not exactly a cheap operation.
> 
> For SQLite, it means numerous requests to the filesystem and memory
> allocations, while for server-based databases it involves performing DNS
> resolution, opening a new TCP connection and allocating buffers.
> 
> Each connection involves a nontrivial allocation of resources for the database
> server, usually including spawning a new thread or process specifically to
> handle the connection, both for concurrency and isolation of faults.
> 
> Additionally, database connections typically involve a complex handshake
> including authentication, negotiation regarding connection parameters (default
> character sets, timezones, locales, supported features) and upgrades to
> encrypted tunnels.

Couldn't have put it better myself! The fix is largely the same as last time.
Instead of opening a new connection for each query we cache a single connection
that gets reused. I'll spare you the code since it's the
same idea, so let's jump straight to the numbers instead (Taking them from the PR description
since it'd be a pain to reproduce now).

_PR: [launchbadge/sqlx#1782](https://github.com/launchbadge/sqlx/pull/1782
"Reuse a cached DB connection instead of always recreating for sqlx-macros")
which caused an
[issue with SQLite](https://github.com/launchbadge/sqlx/issues/1929
'0.6.0: query macro fails on sqlite with "error returned from database: database is locked"'),
so it was
[excluded from the caching logic](https://github.com/launchbadge/sqlx/pull/1930
"Don't cache sqlite connections for macros")_

**\# of queries:** 332

**`sqlx-data.json` size:** 705 KiB

| Setup | `main` (f858138) | This PR | % Change |
| :---: | :---: | :---: | :---:
| MariaDB remote | 61.421 s ±  1.985 s | 12.694 s ±  0.153 s | -79% |
| MariaDB local | 5.291 s ±  0.086 s | 5.050 s ±  0.064 s |  -5% |

That's almost a -80% change when preparing off a remote MariaDB instance! On top
of that the
original ~1 minute is actually after two non-`sqlx` changes to my old work's
mono-crate that drastically reduced the time. It used to be ~8 minutes before
we:

1. Removed an unused `ormx`-based helper crate that seemed to add **_a lot_**
more queries
2. Split the main mono-crate into several crates in a workspace

Well... the second one helped a lot
{{< raw >}}<sub>{{< /raw >}}
if you ignore the full rebuild...
{{< raw >}}</sub>{{< /raw >}}

That whole can of worms deserves a blog post of it's own though (spoilers: All
the work in `sqlx` has already been done)

# Conclusion

Everything is sprawling, so in short the changes were

1. Caching the parsed `sqlx-data.json` file when building the macros
   (`cargo check/build/run`) in offline mode
2. Reusing the same database connections when using a remote database. This
   impacts `cargo check/build/run` when not using offline mode and `cargo sqlx
   prepare` when you are

Now I'm sure something that is on at least some of your all's minds is, "when
will all of this be released?!" The answer is that it already has been... for
like 6 months! It turns out that I'm _really_ slow at writing blog posts.
Hopefully later ones will go by much faster now that I have a lot of
non-recurring engineering work done.

Big thanks to:

- LaunchBadge for open-sourcing `sqlx`, so that I can submit my hot trash for
  them to maintain :relieved:
- Rust for giving me enough compile-time guarantees that I don't want to cry
  when hacking on a new codebase
- Rust tooling for being **insanely** good at times
- My previous coworker for always finding things that I manage to miss
