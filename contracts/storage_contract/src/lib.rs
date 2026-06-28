#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

/// On-chain record for a single storage unit listed by a facility operator.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Unit {
    /// Address of the facility operator that owns the unit.
    pub operator: Address,
    /// Size of the unit in square meters.
    pub size: u32,
    /// Monthly rental rate (in contract-defined units, e.g. USDC stroops).
    pub monthly_rate: u32,
    /// `true` while the unit is available for a new booking.
    pub available: bool,
}

/// On-chain record for an active or historical rental.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Rental {
    /// The unit being rented.
    pub unit_id: u32,
    /// Address of the renter.
    pub renter: Address,
    /// Unix timestamp at which the rental term starts.
    pub start: u64,
    /// Unix timestamp at which the rental term ends.
    pub end: u64,
    /// Status code: `0` = Active, `1` = Late, `2` = Terminated, `3` = Expired.
    pub status: u32,
    /// Late fee assessed by the operator (capped on-chain).
    pub late_fee: u32,
}

// Storage keys built with `symbol_short!` to keep them cheap on-chain.
const RENTAL_COUNT: Symbol = symbol_short!("RCOUNT");
const MAX_LATE_FEE: u32 = 10_000;

/// `storage_contract` — on-chain registry and rental ledger for self-storage units.
///
/// A facility operator lists storage units of varying sizes; renters book a unit
/// for a time period; late fees and early-termination rules are recorded on-chain
/// so that both parties share a tamper-proof rental history.
#[contract]
pub struct StorageContract;

#[contractimpl]
impl StorageContract {
    /// Register a new storage unit with the given size and monthly rate.
    ///
    /// The `operator` must authorize the call. Each `unit_id` can only be listed
    /// once. `size` and `monthly_rate` must be strictly positive.
    pub fn list_unit(
        env: Env,
        operator: Address,
        unit_id: u32,
        size: u32,
        monthly_rate: u32,
    ) {
        operator.require_auth();

        if size == 0 {
            panic!("Unit size must be positive");
        }
        if monthly_rate == 0 {
            panic!("Monthly rate must be positive");
        }

        let key = (symbol_short!("UNIT"), unit_id);
        if env.storage().instance().has(&key) {
            panic!("Unit already listed");
        }

        let unit = Unit {
            operator: operator.clone(),
            size,
            monthly_rate,
            available: true,
        };
        env.storage().instance().set(&key, &unit);
    }

    /// Book a previously listed unit for the period `[start, end]`.
    ///
    /// The `renter` must authorize the call. The unit must exist and be
    /// currently available. Returns the new monotonically increasing `rental_id`.
    pub fn rent_unit(
        env: Env,
        renter: Address,
        unit_id: u32,
        start: u64,
        end: u64,
    ) -> u32 {
        renter.require_auth();

        if end <= start {
            panic!("End must be strictly after start");
        }

        let ukey = (symbol_short!("UNIT"), unit_id);
        let mut unit: Unit = env
            .storage()
            .instance()
            .get(&ukey)
            .unwrap_or_else(|| panic!("Unit not found"));

        if !unit.available {
            panic!("Unit not available");
        }

        // Mark the unit as booked.
        unit.available = false;
        env.storage().instance().set(&ukey, &unit);

        // Allocate a new rental id.
        let count: u32 = env
            .storage()
            .instance()
            .get(&RENTAL_COUNT)
            .unwrap_or(0u32);
        let rental_id = count + 1;
        env.storage().instance().set(&RENTAL_COUNT, &rental_id);

        let rental = Rental {
            unit_id,
            renter: renter.clone(),
            start,
            end,
            status: 0, // Active
            late_fee: 0,
        };
        env.storage()
            .instance()
            .set(&(symbol_short!("RENT"), rental_id), &rental);

        rental_id
    }

    /// Allow the renter to terminate a rental early, recording an on-chain reason.
    ///
    /// The `renter` must authorize the call and must own the rental. Returns the
    /// late fee that had been assessed (zero if the renter was current on rent).
    pub fn terminate(env: Env, renter: Address, rental_id: u32, reason: Symbol) -> u32 {
        renter.require_auth();

        let key = (symbol_short!("RENT"), rental_id);
        let mut rental: Rental = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("Rental not found"));

        if rental.renter != renter {
            panic!("Not the renter of this rental");
        }
        if rental.status == 2 {
            panic!("Rental already terminated");
        }

        rental.status = 2; // Terminated
        let fee = rental.late_fee;
        env.storage().instance().set(&key, &rental);

        // Persist the termination reason for audit.
        env.storage()
            .instance()
            .set(&(symbol_short!("RREASON"), rental_id), &reason);

        fee
    }

    /// Operator flags a rental as late and records the `late_fee` to charge.
    ///
    /// Only the operator of the underlying unit may call this. The fee is capped
    /// on-chain to keep the contract safe from operator abuse.
    pub fn mark_late(env: Env, operator: Address, rental_id: u32, late_fee: u32) {
        operator.require_auth();

        let key = (symbol_short!("RENT"), rental_id);
        let mut rental: Rental = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("Rental not found"));

        let ukey = (symbol_short!("UNIT"), rental.unit_id);
        let unit: Unit = env
            .storage()
            .instance()
            .get(&ukey)
            .unwrap_or_else(|| panic!("Unit record missing"));

        if unit.operator != operator {
            panic!("Not the unit operator");
        }
        if rental.status == 2 {
            panic!("Rental already terminated");
        }
        if late_fee > MAX_LATE_FEE {
            panic!("Late fee exceeds on-chain cap");
        }

        rental.status = 1; // Late
        rental.late_fee = late_fee;
        env.storage().instance().set(&key, &rental);
    }

    /// Renew an active rental by extending its `end` timestamp.
    ///
    /// The `renter` must authorize the call. Any outstanding late fee is cleared
    /// in the same transaction. Returns the new `end` timestamp.
    pub fn renew(env: Env, renter: Address, rental_id: u32, new_end: u64) -> u64 {
        renter.require_auth();

        let key = (symbol_short!("RENT"), rental_id);
        let mut rental: Rental = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("Rental not found"));

        if rental.renter != renter {
            panic!("Not the renter of this rental");
        }
        if rental.status == 2 {
            panic!("Cannot renew a terminated rental");
        }
        if new_end <= rental.end {
            panic!("New end must be later than current end");
        }

        rental.end = new_end;
        rental.status = 0; // Re-activate
        rental.late_fee = 0; // Clear late fee on renewal
        env.storage().instance().set(&key, &rental);

        new_end
    }

    /// Read the on-chain status of a rental.
    ///
    /// Returns a `u32` status code: `0` = Active, `1` = Late, `2` = Terminated,
    /// `3` = Expired. Panics if the rental id is unknown.
    pub fn get_rental(env: Env, rental_id: u32) -> u32 {
        let key = (symbol_short!("RENT"), rental_id);
        let rental: Rental = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("Rental not found"));
        rental.status
    }
}
