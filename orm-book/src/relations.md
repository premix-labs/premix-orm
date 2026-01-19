# Relations

Premix supports `has_many` and `belongs_to` relations. There are two layers:

1. Lazy relation methods generated from struct-level attributes.
2. Eager loading via `include()` on fields marked with `#[premix(ignore)]`.

![Relations Flow](../../assets/premix-orm-relations-flow.svg)

## 1. Lazy Relations (Struct Attributes)

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
#[has_many(Post)]
struct User {
    id: i32,
    name: String,
}

#[derive(Model)]
#[belongs_to(User)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}
```

This generates lazy methods:

- `user.posts_lazy(&pool).await?`
- `post.user(&pool).await?`

Lazy loading is simple and explicit, but it performs one query per parent row.

## 2. Eager Loading (Field Attributes)

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,

    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,
}

#[derive(Model)]
#[belongs_to(User)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}
```

Then call:

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,

    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,
}

#[derive(Model)]
#[belongs_to(User)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, Post>(&pool).await?;
let users = User::find_in_pool(&pool)
    .include("posts")
    .all()
    .await?;
# Ok(())
# }
```

Premix batches related rows using a `WHERE IN (...)` query and attaches the
results to the `posts` field.

## Avoiding N+1

**Lazy** (N+1 queries):

```rust,no_run
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
# let users: Vec<User> = Vec::new();
for user in users {
    let _posts = user.posts_lazy(&pool).await?;
}
# Ok(())
# }
```

**Eager** (2 queries total):

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,

    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,
}

#[derive(Model)]
#[belongs_to(User)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, Post>(&pool).await?;
let users = User::find_in_pool(&pool)
    .include("posts")
    .all()
    .await?;
# Ok(())
# }
```

## Conventions and Requirements

- `belongs_to(User)` expects a `user_id` field on the child model.
- Eager loading requires a `#[premix(ignore)]` field to store the relation.
- The relation name passed to `include("...")` must match the field name.

If you need complex joins or custom projections, use raw SQL and map results
manually.
