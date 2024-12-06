use super::{Error, STATE};
use candid::Principal;

pub fn check_if_owner(caller: Principal) -> Result<(), Error> {
    let owner = STATE.with(|state| {
        state
            .borrow()
            .get(&"owner".to_string())
            .unwrap_or("".to_string())
    });

    if caller.to_string() != owner {
        ic_cdk::println!("owner:{}\tcaller:{}", owner, caller);
        return Err(Error::Unauthorized);
    }

    Ok(())
}
