#![no_std]

 use soroban_sdk::{
     contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
 };

 use shared::utils::calculate_share;

// TODO: Implement multi-party payment contract
// This contract will handle:
// - Setup multiple stakeholders with share percentages
// - Receive and split payments automatically
// - Party withdrawal mechanism
// - Optional vesting schedules

 pub const PARTIES_SETUP: Symbol = symbol_short!("mp_setup");
 pub const PAYMENT_RECEIVED: Symbol = symbol_short!("mp_pay");
 pub const SHARE_WITHDRAWN: Symbol = symbol_short!("mp_with");

 #[contracttype]
 #[derive(Clone)]
 pub struct Party {
     pub address: Address,
     pub share_percentage: u32, // basis points (10000 = 100%)
     pub claimable: i128,
     pub total_received: i128,
 }

 #[contracttype]
 #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
 #[repr(u32)]
 pub enum DataKey {
     Parties = 0,
 }

 #[contracterror]
 #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
 #[repr(u32)]
 pub enum MultiPartyPaymentError {
     InvalidInput = 1000,
     InvalidShareTotal = 1001,
     PartyIndexOutOfBounds = 1002,
     PartiesNotSetup = 1003,
     PartyMismatch = 1004,
     NoClaimableAmount = 1005,
 }

 #[contract]
 pub struct MultiPartyPayment;

 fn get_parties_key(project_id: u64) -> (DataKey, u64) {
     (DataKey::Parties, project_id)
 }

 fn load_parties(env: &Env, project_id: u64) -> Result<Vec<Party>, MultiPartyPaymentError> {
     env.storage()
         .persistent()
         .get(&get_parties_key(project_id))
         .ok_or(MultiPartyPaymentError::PartiesNotSetup)
 }

 fn store_parties(env: &Env, project_id: u64, parties: &Vec<Party>) {
     env.storage()
         .persistent()
         .set(&get_parties_key(project_id), parties);
 }

 fn validate_parties(parties: &Vec<Party>) -> Result<(), MultiPartyPaymentError> {
     if parties.is_empty() {
         return Err(MultiPartyPaymentError::InvalidInput);
     }

     let mut total: u32 = 0;
     for p in parties.iter() {
         total = total
             .checked_add(p.share_percentage)
             .ok_or(MultiPartyPaymentError::InvalidInput)?;
     }

     if total != 10000 {
         return Err(MultiPartyPaymentError::InvalidShareTotal);
     }

     Ok(())
 }

 fn get_party_by_index(parties: &Vec<Party>, party_index: u32) -> Result<Party, MultiPartyPaymentError> {
     let idx: usize = party_index as usize;
     if idx >= parties.len() {
         return Err(MultiPartyPaymentError::PartyIndexOutOfBounds);
     }
     Ok(parties.get(idx).unwrap())
 }

 #[contractimpl]
 impl MultiPartyPayment {
     pub fn setup_parties(
         env: Env,
         project_id: u64,
         parties: Vec<Party>,
     ) -> Result<(), MultiPartyPaymentError> {
         env.invoker().require_auth();

         validate_parties(&parties)?;

         let mut sanitized = Vec::new(&env);
         for p in parties.iter() {
             sanitized.push_back(Party {
                 address: p.address.clone(),
                 share_percentage: p.share_percentage,
                 claimable: 0,
                 total_received: 0,
             });
         }

         store_parties(&env, project_id, &sanitized);

         env.events()
             .publish((PARTIES_SETUP,), (project_id, sanitized.len()));

         Ok(())
     }

     pub fn receive_payment(env: Env, project_id: u64, amount: i128) -> Result<(), MultiPartyPaymentError> {
         if amount <= 0 {
             return Err(MultiPartyPaymentError::InvalidInput);
         }

         let mut parties = load_parties(&env, project_id)?;

         let mut allocated_sum: i128 = 0;
         let last_idx = parties
             .len()
             .checked_sub(1)
             .ok_or(MultiPartyPaymentError::PartiesNotSetup)?;

         for i in 0..parties.len() {
             let mut p = parties.get(i).unwrap();
             let mut share_amt: i128 = calculate_share(amount, p.share_percentage);

             if i == last_idx {
                 let remainder = amount
                     .checked_sub(allocated_sum)
                     .ok_or(MultiPartyPaymentError::InvalidInput)?;
                 share_amt = remainder;
             }

             allocated_sum = allocated_sum
                 .checked_add(share_amt)
                 .ok_or(MultiPartyPaymentError::InvalidInput)?;

             p.claimable = p
                 .claimable
                 .checked_add(share_amt)
                 .ok_or(MultiPartyPaymentError::InvalidInput)?;
             p.total_received = p
                 .total_received
                 .checked_add(share_amt)
                 .ok_or(MultiPartyPaymentError::InvalidInput)?;

             parties.set(i, p);
         }

         store_parties(&env, project_id, &parties);
         env.events().publish((PAYMENT_RECEIVED,), (project_id, amount));

         Ok(())
     }

     pub fn withdraw_share(
         env: Env,
         project_id: u64,
         party_index: u32,
         party: Address,
     ) -> Result<i128, MultiPartyPaymentError> {
         party.require_auth();

         let mut parties = load_parties(&env, project_id)?;
         let idx: usize = party_index as usize;

         if idx >= parties.len() {
             return Err(MultiPartyPaymentError::PartyIndexOutOfBounds);
         }

         let mut p = parties.get(idx).unwrap();
         if p.address != party {
             return Err(MultiPartyPaymentError::PartyMismatch);
         }

         if p.claimable <= 0 {
             return Err(MultiPartyPaymentError::NoClaimableAmount);
         }

         let withdrawn = p.claimable;
         p.claimable = 0;
         parties.set(idx, p);

         store_parties(&env, project_id, &parties);

         env.events()
             .publish((SHARE_WITHDRAWN,), (project_id, party_index, party, withdrawn));

         Ok(withdrawn)
     }

     pub fn get_party(env: Env, project_id: u64, party_index: u32) -> Result<Party, MultiPartyPaymentError> {
         let parties = load_parties(&env, project_id)?;
         get_party_by_index(&parties, party_index)
     }
 }

 #[cfg(test)]
 mod tests {
     use super::*;
     use soroban_sdk::{testutils::{Address as TestAddress, Events as _}, Address, Env, IntoVal, Val, vec};

     fn party(env: &Env, address: Address, share_percentage: u32) -> Party {
         Party {
             address,
             share_percentage,
             claimable: 0,
             total_received: 0,
         }
     }

     #[test]
     fn test_setup_fails_if_share_total_not_10000() {
         let env = Env::default();
         let contract_id = env.register_contract(None, MultiPartyPayment);
         let client = MultiPartyPaymentClient::new(&env, &contract_id);

         env.mock_all_auths();

         let p1 = party(&env, Address::generate(&env), 5000);
         let p2 = party(&env, Address::generate(&env), 4000);
         let parties = Vec::from_array(&env, [p1, p2]);

         let res = client.try_setup_parties(&1u64, &parties);
         assert!(res.is_err());
     }

     #[test]
     fn test_receive_payment_distributes_correctly_with_remainder() {
         let env = Env::default();
         let contract_id = env.register_contract(None, MultiPartyPayment);
         let client = MultiPartyPaymentClient::new(&env, &contract_id);

         env.mock_all_auths();

         let p1_addr = Address::generate(&env);
         let p2_addr = Address::generate(&env);
         let p3_addr = Address::generate(&env);

         let parties = Vec::from_array(
             &env,
             [
                 party(&env, p1_addr.clone(), 3333),
                 party(&env, p2_addr.clone(), 3333),
                 party(&env, p3_addr.clone(), 3334),
             ],
         );

         client.setup_parties(&1u64, &parties);
         client.receive_payment(&1u64, &100i128);

         let p0 = client.get_party(&1u64, &0u32);
         let p1 = client.get_party(&1u64, &1u32);
         let p2 = client.get_party(&1u64, &2u32);

         assert_eq!(p0.claimable, 33);
         assert_eq!(p1.claimable, 33);
         assert_eq!(p2.claimable, 34);
     }

     #[test]
     fn test_withdraw_cannot_exceed_claimable_and_index_safe() {
         let env = Env::default();
         let contract_id = env.register_contract(None, MultiPartyPayment);
         let client = MultiPartyPaymentClient::new(&env, &contract_id);

         env.mock_all_auths();

         let p1_addr = Address::generate(&env);
         let p2_addr = Address::generate(&env);
         let parties = Vec::from_array(
             &env,
             [party(&env, p1_addr.clone(), 5000), party(&env, p2_addr.clone(), 5000)],
         );

         client.setup_parties(&7u64, &parties);
         client.receive_payment(&7u64, &10i128);

         let withdrawn = client.withdraw_share(&7u64, &0u32, &p1_addr);
         assert_eq!(withdrawn, 5);

         let res = client.try_withdraw_share(&7u64, &0u32, &p1_addr);
         assert!(res.is_err());

         let res = client.try_get_party(&7u64, &99u32);
         assert!(res.is_err());
     }

     #[test]
     fn test_events_emitted() {
         let env = Env::default();
         let contract_id = env.register_contract(None, MultiPartyPayment);
         let client = MultiPartyPaymentClient::new(&env, &contract_id);

         env.mock_all_auths();

         let p1_addr = Address::generate(&env);
         let p2_addr = Address::generate(&env);
         let parties = Vec::from_array(
             &env,
             [party(&env, p1_addr.clone(), 5000), party(&env, p2_addr.clone(), 5000)],
         );

         client.setup_parties(&1u64, &parties);
         client.receive_payment(&1u64, &10i128);
         client.withdraw_share(&1u64, &0u32, &p1_addr);

         let events = env.events().all();
         assert_eq!(events.len(), 3);

         let setup_topics = events.get(0).unwrap().1;
         assert_eq!(setup_topics.len(), 1);
         assert_eq!(setup_topics.get(0).unwrap(), PARTIES_SETUP.into_val(&env));

         let pay_topics = events.get(1).unwrap().1;
         assert_eq!(pay_topics.get(0).unwrap(), PAYMENT_RECEIVED.into_val(&env));

         let wd_topics = events.get(2).unwrap().1;
         assert_eq!(wd_topics.get(0).unwrap(), SHARE_WITHDRAWN.into_val(&env));
     }
 }
use shared::{
    errors::Error,
    events::{PAYMENT_RECEIVED, PAYMENT_SETUP, PAYMENT_WITHDRAWN},
};
use soroban_sdk::{contract, contractimpl, contracttype, token::TokenClient, Address, Env, Map};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Stakeholder(u64, Address), // (project_id, stakeholder) -> BasisPoints
    StakeholderInfo(u64, Address), // (project_id, stakeholder) -> StakeholderInfo
    ProjectToken(u64),         // project_id -> Address
    AccPaymentPerShare(u64),   // project_id -> i128
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct StakeholderInfo {
    pub shares: u32,
    pub accumulated_at_last_update: i128,
    pub claimable_amount: i128,
}

#[contract]
pub struct MultiPartyPayment;

#[cfg(test)]
mod tests;

const PRECISION: i128 = 1_000_000_000_000;

#[contractimpl]
impl MultiPartyPayment {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn setup_payment(
        env: Env,
        project_id: u64,
        token: Address,
        stakeholders: Map<Address, u32>,
    ) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let mut total_shares: u32 = 0;
        let current_acc = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::AccPaymentPerShare(project_id))
            .unwrap_or(0);

        for (address, shares) in stakeholders.iter() {
            total_shares += shares;
            let info = StakeholderInfo {
                shares,
                accumulated_at_last_update: current_acc,
                claimable_amount: 0,
            };
            env.storage().persistent().set(
                &DataKey::StakeholderInfo(project_id, address.clone()),
                &info,
            );
        }

        if total_shares != 10000 {
            return Err(Error::InvalidInput); // Must be exactly 100%
        }

        env.storage()
            .persistent()
            .set(&DataKey::ProjectToken(project_id), &token);

        env.events().publish((PAYMENT_SETUP,), project_id);
        Ok(())
    }

    pub fn receive_payment(
        env: Env,
        project_id: u64,
        payer: Address,
        amount: i128,
    ) -> Result<(), Error> {
        payer.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidInput);
        }

        let token_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::ProjectToken(project_id))
            .ok_or(Error::NotFound)?;
        let token_client = TokenClient::new(&env, &token_address);

        // Transfer to contract
        token_client.transfer(&payer, &env.current_contract_address(), &amount);

        // Update global accumulated payment
        let current_acc = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::AccPaymentPerShare(project_id))
            .unwrap_or(0);
        let delta = (amount.checked_mul(PRECISION).ok_or(Error::InvalidInput)?) / 10000;
        env.storage().persistent().set(
            &DataKey::AccPaymentPerShare(project_id),
            &(current_acc + delta),
        );

        env.events()
            .publish((PAYMENT_RECEIVED,), (project_id, payer, amount));
        Ok(())
    }

    pub fn withdraw(env: Env, project_id: u64, stakeholder: Address) -> Result<i128, Error> {
        stakeholder.require_auth();

        let token_address: Address = env
            .storage()
            .persistent()
            .get(&DataKey::ProjectToken(project_id))
            .ok_or(Error::NotFound)?;
        let mut info: StakeholderInfo = env
            .storage()
            .persistent()
            .get(&DataKey::StakeholderInfo(project_id, stakeholder.clone()))
            .ok_or(Error::NotFound)?;

        let current_acc = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::AccPaymentPerShare(project_id))
            .unwrap_or(0);

        let pending =
            (info.shares as i128 * (current_acc - info.accumulated_at_last_update)) / PRECISION;
        let total_claimable = info.claimable_amount + pending;

        if total_claimable <= 0 {
            return Err(Error::InvalidInput);
        }

        info.claimable_amount = 0;
        info.accumulated_at_last_update = current_acc;
        env.storage().persistent().set(
            &DataKey::StakeholderInfo(project_id, stakeholder.clone()),
            &info,
        );

        let token_client = TokenClient::new(&env, &token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &stakeholder,
            &total_claimable,
        );

        env.events().publish(
            (PAYMENT_WITHDRAWN,),
            (project_id, stakeholder, total_claimable),
        );
        Ok(total_claimable)
    }

    pub fn get_balance(env: Env, project_id: u64, stakeholder: Address) -> Result<i128, Error> {
        let info: StakeholderInfo = env
            .storage()
            .persistent()
            .get(&DataKey::StakeholderInfo(project_id, stakeholder))
            .ok_or(Error::NotFound)?;
        let current_acc = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::AccPaymentPerShare(project_id))
            .unwrap_or(0);

        let pending =
            (info.shares as i128 * (current_acc - info.accumulated_at_last_update)) / PRECISION;
        Ok(info.claimable_amount + pending)
    }
}
