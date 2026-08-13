use std::fmt;

/// Every expected failure in the lending library; never produced by a panic.
#[derive(Debug, PartialEq, Eq)]
pub enum LibraryError {
    EmptyTitle,
    DuplicateItemId {
        id: u32,
    },
    DuplicateMemberId {
        id: u32,
    },
    ItemNotFound {
        id: u32,
    },
    MemberNotFound {
        id: u32,
    },
    ItemAlreadyOnLoan {
        id: u32,
        member_id: u32,
    },
    ItemNotOnLoan {
        id: u32,
    },
    ItemIsLost {
        id: u32,
    },
    BorrowLimitReached {
        member_id: u32,
        limit: usize,
    },
    InvalidReturnDay {
        day_borrowed: u32,
        day_returned: u32,
    },
}

impl fmt::Display for LibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::EmptyTitle => {
                write!(formatter, "an item must have a non-empty title")
            }
            LibraryError::DuplicateItemId { id } => {
                write!(formatter, "an item with id {id} is already stocked")
            }
            LibraryError::DuplicateMemberId { id } => {
                write!(formatter, "a member with id {id} is already registered")
            }
            LibraryError::ItemNotFound { id } => {
                write!(formatter, "no item with id {id} is in the catalogue")
            }
            LibraryError::MemberNotFound { id } => {
                write!(formatter, "no member with id {id} is registered")
            }
            LibraryError::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    formatter,
                    "item {id} is already on loan to member {member_id}"
                )
            }
            LibraryError::ItemNotOnLoan { id } => {
                write!(formatter, "item {id} is not currently on loan")
            }
            LibraryError::ItemIsLost { id } => {
                write!(
                    formatter,
                    "item {id} has been reported lost and cannot be lent"
                )
            }
            LibraryError::BorrowLimitReached { member_id, limit } => {
                write!(
                    formatter,
                    "member {member_id} already holds the maximum of {limit} items"
                )
            }
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => {
                write!(
                    formatter,
                    "cannot return on day {day_returned} before it was borrowed on day {day_borrowed}"
                )
            }
        }
    }
}

impl std::error::Error for LibraryError {}
