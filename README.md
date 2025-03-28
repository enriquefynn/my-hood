# My Hood 🏘️

## This is an open source software for managing associations, clubs and other places that require coordination between multiple parties

### Installing dependencies

- Install Rust, follow the steps at <https://rustup.rs/>

- Install Postgres
- Migrate database with

    ```bash
    sqlx database create
    sqlx migrate run
    ```

- Move the `.env.example` file to `.env` and replace or add fields.
- Run `cargo run -- run` to run the project.

### Create a user

First you need a super user, create one with

```bash
cargo run -- create-user superuser@hood.com password
```

Get an authentication token with:

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"email": "superuser@hood.com", "password": "password"}' \
  http://127.0.0.1:8000/auth
```

## Examples of queries

### Add association

mutation add_association {
  createAssociation(association: {
    name: "foo",
    neighborhood: "Foo",
    country: "BR",
    state: "BA",
    address: "Foobar street",
  }) {
    id,
    name
  }
}

## Tests

cargo test -- --test-threads=1
