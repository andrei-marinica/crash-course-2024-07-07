mod basic_interact_cli;
mod basic_interact_config;
mod basic_interact_state;

use crash::crash_proxy;
use caller_sc::caller_proxy;
use basic_interact_config::Config;
use basic_interact_state::State;
use clap::Parser;

use multiversx_sc_snippets::{imports::*, sdk::wallet::Wallet};

const INTERACTOR_SCENARIO_TRACE_PATH: &str = "interactor_trace.scen.json";

const ADDER_CODE_PATH: MxscPath = MxscPath::new("../crash-sc/output/crash.mxsc.json");
const CALLER_CODE_PATH: MxscPath = MxscPath::new("../caller-sc/output/caller-sc.mxsc.json");

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut basic_interact = CrashInteract::init().await;

    let cli = basic_interact_cli::InteractCli::parse();
    match &cli.command {
        Some(basic_interact_cli::InteractCliCommand::Add(args)) => {
            basic_interact.add(args.value).await;
        },
        Some(basic_interact_cli::InteractCliCommand::Deploy) => {
            basic_interact.deploy().await;
        },
        Some(basic_interact_cli::InteractCliCommand::Feed) => {
            basic_interact.feed_contract_egld().await;
        },
        Some(basic_interact_cli::InteractCliCommand::MultiDeploy(args)) => {
            basic_interact.multi_deploy(&args.count).await;
        },
        Some(basic_interact_cli::InteractCliCommand::Sum) => {
            basic_interact.print_sum().await;
        },
        Some(basic_interact_cli::InteractCliCommand::Upgrade(args)) => {
            basic_interact.upgrade(args.value).await
        },
        Some(basic_interact_cli::InteractCliCommand::DeployCaller) => {
            basic_interact.deploy_caller().await;
        },
        Some(basic_interact_cli::InteractCliCommand::CallCaller(args)) => {
            basic_interact.call_caller(args.value).await;
        },
        
        None => {},
    }
}

#[allow(unused)]
struct CrashInteract {
    interactor: Interactor,
    wallet_address: Bech32Address,
    state: State,
}

impl CrashInteract {
    async fn init() -> Self {
        let config = Config::load_config();
        let mut interactor = Interactor::new(config.gateway())
            .await
            .with_tracer(INTERACTOR_SCENARIO_TRACE_PATH)
            .await;
        let wallet_address = interactor.register_wallet(Wallet::from_pem_file("crash.pem").unwrap());

        Self {
            interactor,
            wallet_address: wallet_address.into(),
            state: State::load_state(),
        }
    }

    async fn set_state(&mut self) {
        println!("wallet address: {}", self.wallet_address);
        self.interactor.retrieve_account(&self.wallet_address).await;
    }

    async fn deploy(&mut self) {
        // warning: multi deploy not yet fully supported
        // only works with last deployed address

        self.set_state().await;

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000)
            .typed(crash_proxy::CrashProxy)
            .init(0u32)
            .code(ADDER_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewBech32Address)
            .prepare_async()
            .run()
            .await;

        println!("new address: {new_address}");
        self.state.set_adder_address(new_address);
    }

    async fn deploy_caller(&mut self) {
        // warning: multi deploy not yet fully supported
        // only works with last deployed address

        self.set_state().await;

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000)
            .typed(caller_proxy::CrashProxy)
            .init(self.state.current_adder_address())
            .code(CALLER_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewBech32Address)
            .prepare_async()
            .run()
            .await;

        println!("new address: {new_address}");
        self.state.caller_address = Some(new_address);
    }

    async fn multi_deploy(&mut self, count: &u8) {
        if *count == 0 {
            println!("count must be greater than 0");
            return;
        }

        self.set_state().await;
        println!("deploying {count} contracts...");

        let mut buffer = self.interactor.homogenous_call_buffer();
        for _ in 0..*count {
            buffer.push_tx(|tx| {
                tx.from(&self.wallet_address)
                    .typed(crash_proxy::CrashProxy)
                    .init(0u32)
                    .code(ADDER_CODE_PATH)
                    .gas(NumExpr("70,000,000"))
                    .returns(ReturnsNewBech32Address)
            });
        }

        let results = buffer.run().await;

        // warning: multi deploy not yet fully supported
        // only works with last deployed address

        for new_address in results {
            println!("new address: {new_address}");

            self.state.set_adder_address(new_address);
        }
    }

    async fn feed_contract_egld(&mut self) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_adder_address())
            .egld(NumExpr("0,050000000000000000"))
            .prepare_async()
            .run()
            .await;
    }

    async fn add(&mut self, value: u32) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_adder_address())
            .gas(NumExpr("30,000,000"))
            .typed(crash_proxy::CrashProxy)
            .add(value)
            .prepare_async()
            .run()
            .await;

        println!("successfully performed add");
    }


    async fn call_caller(&mut self, value: u32) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_caller_address())
            .gas(NumExpr("30,000,000"))
            .typed(caller_proxy::CrashProxy)
            .call_add(value)
            .prepare_async()
            .run()
            .await;

        println!("successfully performed add");
    }

    async fn print_sum(&mut self) {
        let sum = self
            .interactor
            .query()
            .to(self.state.current_adder_address())
            .typed(crash_proxy::CrashProxy)
            .sum()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("sum: {sum}");
    }

    async fn upgrade(&mut self, new_value: u32) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_adder_address())
            .gas(NumExpr("30,000,000"))
            .typed(crash_proxy::CrashProxy)
            .upgrade(BigUint::from(new_value))
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .code(ADDER_CODE_PATH)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        let sum = self
            .interactor
            .query()
            .to(self.state.current_adder_address())
            .typed(crash_proxy::CrashProxy)
            .sum()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;
        assert_eq!(sum, RustBigUint::from(new_value));

        println!("response: {response:?}");
    }
}

