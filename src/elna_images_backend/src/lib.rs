mod error;
mod utils;
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use error::Error;
use ic_cdk::export_candid;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{storable::Bound, Storable};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use serde::Serialize;
use std::{borrow::Cow, cell::RefCell};
use utils::check_if_owner;

#[derive(CandidType, Debug, Serialize, Deserialize, Clone)]
struct Asset {
    owner: Principal,
    asset: String,
    file_name: String,
}
impl Storable for Asset {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static ASSETS: RefCell<StableBTreeMap<String,Asset,VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))))
    );
    pub static STATE: RefCell<StableBTreeMap<String,String,VirtualMemory<DefaultMemoryImpl>>> =  RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );
}

#[ic_cdk::init]
fn init(owner: Principal) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.insert("owner".to_string(), owner.to_string())
    });
}

#[ic_cdk::query]
fn get_owner() -> String {
    STATE.with(|state| {
        state
            .borrow()
            .get(&"owner".to_string())
            .unwrap_or("".to_string())
    })
}

#[ic_cdk::update]
fn set_last_id(id: String) -> Result<String, Error> {
    let caller = ic_cdk::caller();
    match check_if_owner(caller) {
        Err(error) => {
            ic_cdk::println!("fn: get_all_assets, Err:{}, caller:{}", error, caller);
            return Err(error);
        }
        Ok(_) => (),
    };

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match state.insert("last_id".to_string(), id.clone()) {
            None => {
                ic_cdk::println!("unable to update last_id,id:{}", id);
                Err(Error::UnableToUpdate)
            }
            Some(id) => Ok(format!("updated to Id:{}", id)),
        }
    })
}

#[ic_cdk::query]
fn get_assets_length() -> Result<u64, Error> {
    let caller = ic_cdk::caller();
    match check_if_owner(caller) {
        Err(error) => {
            ic_cdk::println!("fn: get_all_assets, Err:{}, caller:{}", error, caller);
            return Err(error);
        }
        Ok(_) => (),
    };
    ASSETS.with(|assets| Ok(assets.borrow().len()))
}

#[ic_cdk::query]
fn get_last_id() -> Result<String, Error> {
    let caller = ic_cdk::caller();
    match check_if_owner(caller) {
        Err(error) => {
            ic_cdk::println!("fn: get_all_assets, Err:{}, caller:{}", error, caller);
            return Err(error);
        }
        Ok(_) => (),
    };
    STATE.with(|state| {
        Ok(state
            .borrow()
            .get(&"last_id".to_string())
            .unwrap_or("".to_string()))
    })
}

#[ic_cdk::query]
fn get_all_assets() -> Result<Vec<(String, Asset)>, Error> {
    let caller = ic_cdk::caller();
    match check_if_owner(caller) {
        Err(error) => {
            ic_cdk::println!("fn: get_all_assets, Err:{}, caller:{}", error, caller);
            return Err(error);
        }
        Ok(_) => (),
    };

    let all_assets = ASSETS.with(|assets| {
        assets
            .borrow()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    });
    Ok(all_assets)
}

#[ic_cdk::query]
fn get_asset(asset_id: String) -> Option<Asset> {
    let asset = ASSETS.with(|assets| {
        let assets = assets.borrow();
        assets.get(&asset_id)
    });
    asset
}

#[ic_cdk::update]
fn add_asset(new_asset: Asset, prefix: Option<String>) -> Result<String, Error> {
    let caller = ic_cdk::caller();
    if caller != new_asset.owner {
        ic_cdk::println!(
            "fn:add_asset, caller:{},asset_owner:{}",
            caller,
            new_asset.owner
        );
        return Err(Error::UploaderMismatch);
    }

    let last_id = STATE.with(|state| state.borrow().get(&"last_id".to_string()));
    match last_id {
        None => return Err(Error::UnableToUpdate),
        Some(last_id) => {
            let last_id = match last_id.parse::<u64>() {
                Err(_) => return Err(Error::UnableToReadLastId),
                Ok(id) => id,
            };
            let id = last_id + 1;
            let final_id = format!("{}{}", prefix.unwrap_or("".to_string()), id);
            ic_cdk::println!(
                "fn:add_asset, caller:{},new_id:{},last_id:{}",
                caller,
                final_id,
                last_id
            );

            ASSETS.with(|assets| {
                assets.borrow_mut().insert(final_id.clone(), new_asset);
            });
            STATE.with(|state| {
                state
                    .borrow_mut()
                    .insert("last_id".to_string(), final_id.clone())
            });

            Ok(final_id)
        }
    }
}

#[ic_cdk::update]
fn delete_asset(asset_id: String) -> Result<String, Error> {
    let caller = ic_cdk::caller();

    ASSETS.with(|assets| {
        let mut assets = assets.borrow_mut();
        let asset = assets.get(&asset_id);
        match asset {
            None => {
                ic_cdk::println!(
                    "fn:delete_asset, msg:Asset not found, caller:{},asset_id:{}",
                    caller,
                    asset_id
                );
                return Err(Error::NotFound);
            }
            Some(value) => {
                if value.owner != ic_cdk::caller() {
                    ic_cdk::println!(
                        "fn:delete_asset, msg:caller mismatch, caller:{},owner:{},asset_id:{}",
                        caller,
                        value.owner,
                        asset_id
                    );
                    return Err(Error::Unauthorized);
                }
            }
        }
        ic_cdk::println!(
            "fn:delete_asset, msg:DELETED, caller:{},asset_id:{}",
            caller,
            asset_id
        );
        match assets.remove(&asset_id) {
            None => Err(Error::UnableToDelete),
            Some(value) => Ok(format!("Asset deleted:{}", value.file_name)),
        }
    })
}

export_candid!();
