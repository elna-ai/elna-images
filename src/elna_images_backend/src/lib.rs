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
    pub static  OWNER: RefCell<String> = RefCell::new(String::new())
}

#[ic_cdk::init]
fn init(owner: Principal) {
    OWNER.with(|o| *o.borrow_mut() = owner.to_string());
}

#[ic_cdk::query]
fn get_owner() -> String {
    OWNER.with(|owner: &RefCell<String>| owner.borrow().clone())
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

    let last_id = ASSETS.with(|assets| {
        let assets = assets.borrow();
        assets.len()
    });
    let id = last_id + 1;
    let final_id = format!("{}{}", prefix.unwrap_or("".to_string()), id);

    ASSETS.with(|assets| {
        let mut assets = assets.borrow_mut();
        assets.insert(final_id.clone(), new_asset);
    });

    Ok(final_id)
}

#[ic_cdk::update]
fn delete_asset(asset_id: String) -> Result<String, Error> {
    let caller = ic_cdk::caller();
    match check_if_owner(caller) {
        Err(error) => {
            ic_cdk::println!(
                "fn: delete_assets, Err:{}, caller:{},asset_id:{}",
                error,
                caller,
                asset_id
            );
            return Err(error);
        }
        Ok(_) => (),
    };

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
        match assets.remove(&asset_id) {
            None => Err(Error::UnableToDelete),
            Some(value) => Ok(format!("Asset deleted:{}", value.file_name)),
        }
    })
}

export_candid!();
