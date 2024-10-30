use super::{Error, OWNER};
use candid::Principal;
use std::cell::RefCell;

pub fn check_if_owner(caller: Principal) -> Result<(), Error> {
    let owner = OWNER.with(|owner: &RefCell<String>| owner.borrow().clone());
    if caller.to_string() != owner {
        ic_cdk::println!("owner:{}\tcaller:{}", owner, caller);
        return Err(Error::Unauthorized);
    }

    Ok(())
}
