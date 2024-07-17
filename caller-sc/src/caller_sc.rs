#![no_std]

use multiversx_sc::imports::*;

pub mod crash_proxy;
pub mod caller_proxy;

/// One of the simplest smart contracts possible,
/// it holds a single variable in storage, which anyone can increment.
#[multiversx_sc::contract]
pub trait Crash {
    #[view(targetAddress)]
    #[storage_mapper("targetAddress")]
    fn target_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[init]
    fn init(&self, address: ManagedAddress) {
        self.target_address().set(address);
    }

    #[upgrade]
    fn upgrade(&self, address: ManagedAddress) {
        self.init(address);
    }

    /// Add desired amount to the storage variable.
    #[endpoint(callAdd)]
    fn call_add(&self, value: BigUint) {
        self.tx()
            .to(self.target_address().get())
            .gas(2_000_000)
            .typed(crash_proxy::CrashProxy)
            .add(value)
            .sync_call();
    }
}
