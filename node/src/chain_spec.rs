use fuso_runtime::{
    opaque::SessionKeys, AccountId, AuraConfig, BalancesConfig, CouncilConfig, FoundationConfig,
    GenesisConfig, GrandpaConfig, SessionConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                // vec![authority_keys_from_seed("Moonflower")],
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    development_config()
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        // x.0.clone(),
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        // x.0.clone(),
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        SessionKeys {
                            aura: x.0.clone(),
                            grandpa: x.1.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
        }),
        fuso_pallet_council: Some(CouncilConfig {
            members: vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
        }),
        pallet_aura: Some(AuraConfig {
            // authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
            authorities: vec![],
        }),
        pallet_grandpa: Some(GrandpaConfig {
            // authorities: initial_authorities
            // .iter()
            // .map(|x| (x.1.clone(), 1))
            // .collect(),
            authorities: vec![],
        }),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key,
        }),
        fuso_pallet_foundation: Some(FoundationConfig {
            fund: vec![
                (
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    1 << 60,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    1 << 60,
                ),
            ],
        }),
    }
}
