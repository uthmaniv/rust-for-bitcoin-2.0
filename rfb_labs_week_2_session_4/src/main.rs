//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "Dune".to_string(),
        "Frank Herbert".to_string(),
        MediaKind::Book { pages: 320 },
    ))?;
    library.add_item(Item::new(
        2,
        "Project Hail Mary".to_string(),
        "Andy Weir".to_string(),
        MediaKind::Audiobook { minutes: 540 },
    ))?;
    library.add_item(Item::new(
        3,
        "The Rust Programming Language".to_string(),
        "Steve Klabnik".to_string(),
        MediaKind::Ebook { size_kb: 1_200 },
    ))?;

    library.register_member(Member::new(100, "Ada".to_string()))?;

    // An on-time loan: a book borrowed day 5, returned day 20 (within 21).
    library.checkout(1, 100, 5)?;
    let on_time_fee = library.return_item(1, 20)?;
    println!("on-time return owed {on_time_fee} cents");

    // A late loan: an audiobook held 30 days against a 14-day limit.
    library.checkout(2, 100, 5)?;
    let late_fee = library.return_item(2, 35)?;
    println!("late return owed {late_fee} cents");

    // A handled error, printed instead of crashing the program.
    match library.checkout(999, 100, 5) {
        Ok(()) => println!("that should not have succeeded"),
        Err(error) => println!("handled error: {error}"),
    }

    Ok(())
}
