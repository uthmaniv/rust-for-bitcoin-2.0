# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits,
ownership, borrowing, collections, and `Result`-based error handling. No
Bitcoin and no external crates — just Rust.

The crate is intentionally incomplete. Search for `TODO` and implement each
part; do not change the public type names or function signatures.

## Recommended workflow

1. Read [ASSIGNMENT.md](ASSIGNMENT.md).
2. Complete Part 2 in `error.rs`, then Part 3 in `library.rs`.
3. Remove `#[ignore]` from the relevant test and run it.
4. Complete the traits in Part 4 and the two operations in Parts 5–6.
5. Run the ownership experiments and record the errors.
6. Build the demo in `main.rs`.
7. Add the remaining required tests yourself.

```bash
cargo test
cargo test -- --ignored
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo test` checks the starter project. Ignored tests intentionally exercise
unfinished code; enable them progressively rather than leaving them ignored in
the submission.

## The two ownership experiments

### Experiment A — read the title after `add_item`

The offending line is commented out; running it produced an `E0382`.

```rust
let dune = Item::new(
    1,
    "Dune".to_string(),
    "Frank Herbert".to_string(),
    MediaKind::Book { pages: 320 },
);
library.add_item(dune)?;
// println!("{}", dune.title); // E0382: borrow of moved value
```

```
error[E0382]: borrow of moved value: `dune`
  --> src/main.rs:14:20
   |
 7 |     let dune = Item::new(
   |         ---- move occurs because `dune` has type `Item`, which does not
   |              implement the `Copy` trait
...
13 |     library.add_item(dune)?;
   |                      ---- value moved here
14 |     println!("{}", dune.title);
   |                    ^^^^^^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
```

**What caused it:** `add_item` takes the `Item` **by value**, so the call moves
`dune` into the library's `items` vector. `Item` is deliberately not `Copy` (it
owns two `String`s), so the original binding is invalid afterwards; reading
`dune.title` after the move borrows storage that no longer holds a value. This
is the ownership transfer the method signature was advertising: the library,
not the caller, owns the title now.

### Experiment B — print a held reference after `checkout`

The offending line is commented out; running it produced an `E0502`.

```rust
let held = library.find_item(1).unwrap();
library.checkout(1, 100, 5)?;
// println!("{held}"); // E0502: cannot borrow `library` as mutable
```

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as
              immutable
  --> src/main.rs:14:5
   |
13 |     let held = library.find_item(1).unwrap();
   |                ------- immutable borrow occurs here
14 |     library.checkout(1, 100, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
15 |     println!("{held}");
   |                ---- immutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
```

**What caused it:** `find_item` borrows `library` immutably and hands back a
`&Item` whose lifetime is tied to that borrow. `checkout` needs `&mut library`
to update the item and the member, and Rust will not allow a mutable borrow
while an immutate borrow is still alive — `held` keeps it alive until the final
`println!`. This is exactly why `checkout` and `return_item` are written with
index lookups and short, sequential mutable borrows instead of trying to hold
`&mut Item` and `&mut Member` simultaneously.

## Written answers

Answer in your own words. Add both ownership compiler errors from Part 7 as
fenced text blocks, then explain what caused each.

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?**
   An item is in exactly one of three states: `Available`, `OnLoan`, `Lost`.
   A `bool` can only distinguish two, so you would need a `bool loaned` plus
   `Option<u32> member_id` and `Option<u32> day_borrowed`, which can express
   impossible combinations (loaned but no member, or a member id with no day,
   or a lost item still marked loaned). The enum makes the states mutually
   exclusive by construction and forces every `match` to handle all three, so a
   `Lost` item can never quietly hold borrower data.

2. **What does `match` force you to do when a fourth `MediaKind` is added later?**
   Update every exhaustive `match` over `MediaKind` — here `loan_days`,
   `daily_late_fee_cents`, and `Display` — because the compiler refuses to build
   until the new variant is covered. The burden is exactly the point: "did I
   think about the new kind everywhere?" becomes a compile-time guarantee.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title afterwards?**
   The `Item`. Passing ownership lets the string move into the struct with no
   copying, and the caller can no longer touch it. Taking `&str` would either
   pin the item to the caller's data (a borrow that must outlive the item) or
   force us to copy into our own allocation.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `&mut self` because pushing into `items` mutates the library. `item` by
   value because the library must become the sole owner so that only its methods
   can change the item's `status`. A reference would leave the item's fate
   ambiguous between caller and library.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller passed
   in? Was that a good design choice, and what is the alternative?**
   The item was moved into the function and is dropped on the error path, so the
   caller can no longer use it. That is a reasonable default — if the id turned
   out to be a duplicate, the caller rarely still wants to keep the rejected
   item — and it keeps the signature simple. The alternative is handing the item
   back on failure, e.g. `Result<(), (LibraryError, Item)>`, so the caller could
   repair and resubmit it. I kept the dropping behaviour for simplicity and
   documented it in the error.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**
   Returning an owned `Item` would require either copying it (impossible: it is
   not `Clone`) or moving it out of the library, which would punch a hole in the
   catalogue and let the item and the member lists disagree. A `&Item` observes
   the shelf without taking anything off it.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**
   The returned `Vec<&Item>` is tied to the borrow of `self`: every reference is
   valid only while that borrow lives. Concretely, the compiler can prove the
   caller may not mutate or drop the `Library` while it still holds those
   references, which is exactly what makes the borrow-versus-clone claim in the
   test (`std::ptr::eq`) sound.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?**
   Both would be mutable borrows from the same root (`self`), and Rust permits
   only one active mutable borrow at a time. I find the item and member by
   `position(...)` to get their indexes, validate against those immutable reads,
   then mutate first `self.items[i]` and then `self.members[j]` as short
   sequential borrows, never holding two mutable borrows into `self` together.

9. **Why are `Library`'s fields private?**
   Because the library maintains an invariant: an item's `LoanStatus` and its
   member's borrowed-id list must always agree. If callers could push directly,
   a checkout could mark an item loaned without registering it, and a return
   could clear one side and not the other. Privacy funnels every change through
   the methods that keep the invariant.

10. **What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?**
    Without it, each implementer of `LoanTerms` would copy the formula
    (`overdue days × daily fee`). As a default trait method it is written once
    and shared by `MediaKind` and `Item`, so the policy changes in one place.
    A free function would still remove the duplication but would break the
    `late_fee_cents(days_held)` method-call syntax and would not be inherited
    automatically by any future implementer of the trait.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.**
    Validation failures — unknown ids, empty titles, the borrow limit — are
    expected, recoverable conditions, so they belong in the return type where
    the caller decides what to do. A defensible panic is reserved for internal
    invariants that must hold: indexing `self.items[item_index]` in `checkout`/
    `return_item` would panic if that index were out of bounds, but it cannot
    be, because we computed it from `position()` moments earlier.

12. **Which derive did you deliberately leave off a type, and why?**
    `Clone` on `Item` and `Member`. Those types own `String`s and a loan status;
    letting callers clone whole records would encourage copying instead of
    borrowing and blur who owns what. `MediaKind` and `LoanStatus` are small,
    flat enums of integers, so they are `Copy`/`Clone` — duplicating them carries
    no ownership meaning.

## Design notes

The core design constraint was keeping an item's `status` and its member's
`borrowed_item_ids` in agreement. I made `checkout` and `return_item` the only
mutation points, and each performs both writes in the same call — first validate
against the immutable catalogue, then shift the item's status and push/remove the
id from the member's list. Because there is no public way to mutate one side
without the other, the two views cannot drift.

To avoid holding two mutable borrows into `Library` at once, both methods locate
their records by index (`items.iter().position(...)`, `members.iter().position(...)`)
and then mutate through short, sequential `self.items[i]` / `self.members[j]`
borrows. `find_item`/`find_member` are re-used only for read-only lookups.

I attempted the optional Part 9: `filter_items<F>(&self, predicate: F)` where
`F: Fn(&Item) -> bool`. `items_by_author` and `available_items` are now expressed
in terms of it, so the filtering logic lives once. The `late_fee_cents` formula
lives once too, as a default `LoanTerms` method, so an ebook keeps its "never
late" behaviour for free by reporting a zero daily fee.

The `Lost` state has no public entry point yet (a future feature), so the
lost-item error paths are covered by unit tests inside `src/library.rs`, where
the test module can flag an item lost through the private `items` field; every
other behaviour is tested from the public API in `tests/library.rs`.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.

```
on-time return owed 0 cents
late return owed 400 cents
handled error: no item with id 999 is in the catalogue
```
