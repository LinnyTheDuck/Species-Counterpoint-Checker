# Species Counterpoint Checker

Checks two-part species counterpoint against the rules in [RULES.md](RULES.md) and says which rules a piece breaks. First species works. Second to fifth species are still to come.

The engine is Rust. A GraphQL API serves it to the checker page.

## Layout

- `crates/core`: the rule engine. Pitches, intervals, keys, the rules, and the rule descriptions shown to users (`catalogue.rs`). No web code.
- `crates/api`: the GraphQL service (axum and async-graphql).
- `RULES.md`: the rule spec. Each rule has an id that the engine, the API and the page all use, and a note of where it came from.

## Running it

You need Rust, installed with [rustup](https://rustup.rs).

```
cargo run -p counterpoint-api
```

The API listens on http://127.0.0.1:8080. Set `PORT` to use another port. Open http://127.0.0.1:8080 in a browser for the GraphiQL explorer. Queries go to `/graphql`.

Each check is logged to the terminal with the notes sent and the rules broken.

Cross-origin requests are only allowed from `http://localhost:*` and `http://127.0.0.1:*`, so the site can call the API during local development.

## Tests

```
cargo test                          # everything
cargo test -p counterpoint-core     # just the engine
cargo test species1                 # tests with "species1" in the name
```

Unit tests sit at the bottom of each source file. `crates/core/tests/assignments.rs` holds exercises that got full marks: each one must check with no errors.

## API

### Checking first species

```graphql
query {
  checkFirstSpecies(input: {
    key: "C major"
    placement: ABOVE
    cantusFirmus: ["C4", "D4", "E4", "D4", "C4"]
    counterpoint: ["G4", "A4", "G4", "B4", "C5"]
  }) {
    valid
    violations { rule severity bar beat voice message }
  }
}
```

Returns:

```json
{
  "data": {
    "checkFirstSpecies": {
      "valid": false,
      "violations": [
        {
          "rule": "parallel-perfect",
          "severity": "ERROR",
          "bar": 2,
          "beat": 1,
          "voice": null,
          "message": "Parallel 5ths: P5 to P5."
        }
      ]
    }
  }
}
```

The input fields:
- `key`: a tonic and `major` or `minor`, such as `"C major"`, `"F# minor"` or `"Bb major"`.
- `placement`: `ABOVE` or `BELOW`, meaning the counterpoint above or below the cantus firmus.
- `cantusFirmus`, `counterpoint`: one pitch per bar, at most 64. A pitch is a letter, then `#` or `b` (up to two), then an octave from 0 to 9. Middle C is `C4`.
- `ties` (optional): bars, counted from 1, whose counterpoint note is tied into the next bar. Both bars must hold the same note.

`valid` is true when there are no errors. Warnings don't make a piece invalid. `voice` is `CANTUS_FIRMUS`, `COUNTERPOINT`, or null when the problem is between the two voices.

Input the engine cannot check, such as a bad pitch or voices of different lengths, comes back as a GraphQL error.

### Listing the rules

```graphql
query {
  firstSpeciesRules { id group severity summary }
}
```

Returns every rule first species checks, in the order the page shows them.
