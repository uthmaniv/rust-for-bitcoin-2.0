use rfb_labs_week_2_session_4::{
    Item, Library, LibraryError, LoanStatus, LoanTerms, MediaKind, Member,
};

fn library_with_items() -> Library {
    let mut library = Library::new();

    for (id, title, author, kind) in [
        (1, "Dune", "Frank Herbert", MediaKind::Book { pages: 320 }),
        (
            2,
            "Children of Dune",
            "Frank Herbert",
            MediaKind::Book { pages: 180 },
        ),
        (
            3,
            "Project Hail Mary",
            "Andy Weir",
            MediaKind::Audiobook { minutes: 540 },
        ),
        (
            4,
            "The Rust Programming Language",
            "Steve Klabnik",
            MediaKind::Ebook { size_kb: 1_200 },
        ),
    ] {
        library
            .add_item(Item::new(id, title.into(), author.into(), kind))
            .unwrap();
    }

    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    library
}

// These tests are ignored so the starter repository builds before students
// implement the TODOs. Remove `#[ignore]` from one test at a time while working.

#[test]
fn checkout_updates_both_the_item_and_the_member() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.find_item(1).unwrap().status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 5,
        }
    );
    assert_eq!(library.find_member(100).unwrap().borrowed_item_ids, vec![1]);
}

#[test]
fn an_item_cannot_be_lent_to_a_second_member() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.checkout(1, 100, 6),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        })
    );
}

#[test]
fn a_member_cannot_exceed_the_borrow_limit() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 0).unwrap();
    library.checkout(3, 100, 0).unwrap();

    assert_eq!(
        library.checkout(4, 100, 0),
        Err(LibraryError::BorrowLimitReached {
            member_id: 100,
            limit: 3,
        })
    );
}

#[test]
fn returning_a_book_late_charges_a_daily_fee() {
    let mut library = library_with_items();

    // A book may be kept 21 days. Held for 30, so 9 days are overdue.
    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 40), Ok(9 * 25));
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
    assert!(
        library
            .find_member(100)
            .unwrap()
            .borrowed_item_ids
            .is_empty()
    );
}

#[test]
fn returning_on_time_owes_nothing() {
    let mut library = library_with_items();

    // A book may be kept 21 days; held for 10 exactly.
    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 20), Ok(0));
}

#[test]
fn returning_an_ebook_late_still_owes_nothing() {
    let mut library = library_with_items();

    // An ebook may be kept 7 days and is never late regardless of how long it
    // is held, because its daily fee is zero.
    library.checkout(4, 100, 10).unwrap();

    assert_eq!(library.return_item(4, 60), Ok(0));
}

#[test]
fn searching_by_author_borrows_rather_than_clones() {
    let library = library_with_items();

    let found = library.items_by_author("Frank Herbert");

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].title, "Dune");
    // `found` holds references into `library`, so these are the same item.
    assert!(std::ptr::eq(found[0], library.find_item(1).unwrap()));
}

#[test]
fn author_search_includes_items_currently_on_loan() {
    let mut library = library_with_items();

    // Check out the first Frank Herbert book so search should return one
    // available and one on-loan entry for the same author.
    library.checkout(1, 100, 5).unwrap();

    let found = library.items_by_author("Frank Herbert");
    assert_eq!(found.len(), 2);
    assert_eq!(
        found[0].status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 5,
        }
    );
    assert_eq!(found[1].status, LoanStatus::Available);
}

#[test]
fn adding_an_item_requires_a_title() {
    let mut library = Library::new();

    let blank = Item::new(
        1,
        "".to_string(),
        "No One".to_string(),
        MediaKind::Book { pages: 1 },
    );

    assert_eq!(library.add_item(blank), Err(LibraryError::EmptyTitle));
    assert!(library.find_item(1).is_none());
}

#[test]
fn adding_an_item_with_a_duplicate_id_is_rejected() {
    let mut library = library_with_items();

    let duplicate = Item::new(
        1,
        "A Doppelganger".to_string(),
        "Someone Else".to_string(),
        MediaKind::Ebook { size_kb: 10 },
    );

    assert_eq!(
        library.add_item(duplicate),
        Err(LibraryError::DuplicateItemId { id: 1 })
    );
}

#[test]
fn registering_a_duplicate_member_is_rejected() {
    let mut library = library_with_items();

    let duplicate = Member::new(100, "Second Ada".to_string());

    assert_eq!(
        library.register_member(duplicate),
        Err(LibraryError::DuplicateMemberId { id: 100 })
    );
}

#[test]
fn checkout_of_an_unknown_item_is_reported_before_anything_else() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(999, 100, 0),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
    assert_eq!(
        library.checkout(999, 999, 0),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn checkout_of_an_unknown_member_is_rejected() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(1, 999, 0),
        Err(LibraryError::MemberNotFound { id: 999 })
    );
}

#[test]
fn returning_an_item_not_on_loan_is_rejected() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(1, 5),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );
}

#[test]
fn returning_late_only_charges_days_beyond_the_agreed_length() {
    let mut library = library_with_items();

    // Book loan is 21 days. Held 30 days, but the "loan" started on day 20.
    library.checkout(1, 100, 20).unwrap();

    assert_eq!(library.return_item(1, 50), Ok((50 - 20 - 21) * 25));
}

#[test]
fn returning_before_the_borrow_day_is_an_error_not_an_underflow() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    assert_eq!(
        library.return_item(1, 3),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 10,
            day_returned: 3,
        })
    );
}

#[test]
fn available_items_lists_only_the_shelf() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    let available_ids: Vec<u32> = library
        .available_items()
        .iter()
        .map(|item| item.id)
        .collect();
    assert_eq!(available_ids, vec![2, 3, 4]);
}

#[test]
fn longest_loan_item_prefers_books_over_audiobooks() {
    let library = library_with_items();

    // Both books grant 21 days, which outranks the audiobook's 14 and the
    // ebook's 7, regardless of which of the two books wins the tie.
    assert_eq!(library.longest_loan_item().unwrap().loan_days(), 21);
}
