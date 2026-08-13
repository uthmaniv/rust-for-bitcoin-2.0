use crate::catalogue::{Item, LoanStatus, LoanTerms};
use crate::error::LibraryError;
use crate::member::Member;

pub const MAX_ITEMS_PER_MEMBER: usize = 3;

/// Owns every item and member; keeps the item status and the member's
/// borrowed-id list in agreement. Fields are private by design.
#[derive(Debug, Default)]
pub struct Library {
    items: Vec<Item>,
    members: Vec<Member>,
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_item(&mut self, item: Item) -> Result<(), LibraryError> {
        if item.title.trim().is_empty() {
            return Err(LibraryError::EmptyTitle);
        }
        if self.items.iter().any(|stocked| stocked.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }
        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        if self
            .members
            .iter()
            .any(|registered| registered.id == member.id)
        {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        self.members.iter().find(|member| member.id == id)
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.filter_items(|item| item.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.filter_items(|item| item.status == LoanStatus::Available)
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        self.items.iter().max_by_key(|item| item.loan_days())
    }

    /// Returns every item satisfying `predicate`, without cloning any.
    pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
    where
        F: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // Resolve to indexes first so we can mutate each side one at a time.
        let item_index = self
            .items
            .iter()
            .position(|item| item.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;
        let member_index = self
            .members
            .iter()
            .position(|member| member.id == member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        match self.items[item_index].status {
            LoanStatus::Lost => {
                return Err(LibraryError::ItemIsLost { id: item_id });
            }
            // The item's own holder, not the member trying to borrow it.
            LoanStatus::OnLoan {
                member_id: current_holder,
                ..
            } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id: current_holder,
                });
            }
            LoanStatus::Available => {}
        }

        if self.members[member_index].borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        self.items[item_index].status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };
        self.members[member_index].borrowed_item_ids.push(item_id);
        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        let item_index = self
            .items
            .iter()
            .position(|item| item.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let (days_held, member_id) = match self.items[item_index].status {
            LoanStatus::Lost => {
                return Err(LibraryError::ItemIsLost { id: item_id });
            }
            LoanStatus::OnLoan {
                day_borrowed,
                member_id,
            } => {
                let days_held =
                    day.checked_sub(day_borrowed)
                        .ok_or(LibraryError::InvalidReturnDay {
                            day_borrowed,
                            day_returned: day,
                        })?;
                (days_held, member_id)
            }
            LoanStatus::Available => {
                return Err(LibraryError::ItemNotOnLoan { id: item_id });
            }
        };

        let fee = self.items[item_index].late_fee_cents(days_held);

        self.items[item_index].status = LoanStatus::Available;
        if let Some(member) = self
            .members
            .iter_mut()
            .find(|member| member.id == member_id)
        {
            member
                .borrowed_item_ids
                .retain(|held_id| *held_id != item_id);
        }
        Ok(fee)
    }
}

// The `Lost` state has no public endpoint yet, so these paths are unit tests.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::{Item, MediaKind};

    fn library_with_lost_item() -> (Library, u32) {
        let mut library = Library::new();
        library
            .add_item(Item::new(
                1,
                "Dune".to_string(),
                "Frank Herbert".to_string(),
                MediaKind::Book { pages: 320 },
            ))
            .unwrap();
        library
            .register_member(Member::new(100, "Ada".to_string()))
            .unwrap();
        library.items[0].status = LoanStatus::Lost;
        (library, 1)
    }

    #[test]
    fn checkout_of_a_lost_item_is_rejected() {
        let (mut library, lost_id) = library_with_lost_item();

        assert_eq!(
            library.checkout(lost_id, 100, 0),
            Err(LibraryError::ItemIsLost { id: lost_id })
        );
        assert_eq!(library.items[0].status, LoanStatus::Lost);
    }

    #[test]
    fn returning_a_lost_item_is_rejected() {
        let (mut library, lost_id) = library_with_lost_item();

        assert_eq!(
            library.return_item(lost_id, 5),
            Err(LibraryError::ItemIsLost { id: lost_id })
        );
    }
}
