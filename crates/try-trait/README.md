# Exploring Traits and Trait Syntax pm

## `async`: desugaring required for robust traits

See: [async in public traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html#async-fn-in-public-traits)  
TLDR:
- **BAD**: impl+async (sugar): `async fn xxx -> impl Future<Output = y> + Send` (**BAD**)
  - === `fn xxx -> impl Future< Output = impl Future<Output = y>> + Send` (?)
- *GOOD*: drop sugar, just impl: `fn xxx -> impl Future<Output = y> + Send` (*GOOD*)
  - === `async fn xxx -> y  | Future<Output = y> ∈ Send`

It's natural to write
```rust
trait SomeTrait {
     ///   fn xxx -> impl Future<Output = y>
     async fn xxx -> y
}
```
, *but* in *many* cases we want to bound the *output* to be `Send`.

The twist is that `async` desugars to an output `impl ...` statement.
So we need to strip the async (de-sugar) and just write the impl.

```rust
trait SomeTrait {
     ///   async fn xxx -> y | Future<Output = y> ∈ Send
     fn xxx -> impl Future<Output = y> + Send
}
