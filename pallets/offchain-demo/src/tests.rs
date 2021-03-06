use crate::*;
use parking_lot::{RwLock};
use codec::{
	Decode,
	alloc::sync::{Arc}
};
use frame_support::{
	assert_ok, impl_outer_event, impl_outer_origin, parameter_types,
};
use sp_io::TestExternalities;
use sp_core::{
	H256, sr25519,
	offchain::{OffchainExt, TransactionPoolExt,
		testing::{self, PoolState, OffchainState},
	},
	testing::KeyStore,
	traits::KeystoreExt,
};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

use crate as offchain_demo;

impl_outer_origin! {
	pub enum Origin for TestRuntime where system = system {}
}

impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		system<T>,
		offchain_demo<T>,
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRuntime;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1_000_000;
	pub const MaximumBlockLength: u32 = 10 * 1_000_000;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

// The TestRuntime implements two pallet/frame traits: system, and simple_event
impl system::Trait for TestRuntime {
	type Origin = Origin;
	type Index = u64;
	type Call = ();
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
}

// --- mocking offchain-demo trait

type TestExtrinsic = TestXt<Call<TestRuntime>, ()>;
type SubmitTransaction = system::offchain::TransactionSubmitter<
	crypto::Public,
	TestRuntime,
	TestExtrinsic
>;

parameter_types! {
	pub const GracePeriod: u64 = 2;
}

impl Trait for TestRuntime {
	type Call = Call<TestRuntime>;
	type Event = TestEvent;
	type GracePeriod = GracePeriod;
	type SubmitSignedTransaction = SubmitTransaction;
	type SubmitUnsignedTransaction = SubmitTransaction;
}

impl system::offchain::CreateTransaction<TestRuntime, TestExtrinsic> for TestRuntime {
	type Public = sr25519::Public;
	type Signature = sr25519::Signature;

	fn create_transaction<TSigner: system::offchain::Signer<Self::Public, Self::Signature>> (
		call: Call<TestRuntime>,
		_public: Self::Public,
		_account: <TestRuntime as system::Trait>::AccountId,
		index: <TestRuntime as system::Trait>::Index,
	) -> Option<(Call<TestRuntime>, <TestExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> {
		Some((call, (index, ())))
	}
}

pub type System = system::Module<TestRuntime>;
pub type OffchainDemo = Module<TestRuntime>;

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build() -> (TestExternalities, Arc<RwLock<PoolState>>, Arc<RwLock<OffchainState>>) {
		const PHRASE: &str = "expire stage crawl shell boss any story swamp skull yellow bamboo copy";

		let (offchain, offchain_state) = testing::TestOffchainExt::new();
		let (pool, pool_state) = testing::TestTransactionPoolExt::new();
		let keystore = KeyStore::new();
		keystore.write().sr25519_generate_new(
			KEY_TYPE,
			Some(&format!("{}/hunter1", PHRASE))
		).unwrap();

		let storage = system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();

		let mut t = TestExternalities::from(storage);
		t.register_extension(OffchainExt::new(offchain));
		t.register_extension(TransactionPoolExt::new(pool));
		t.register_extension(KeystoreExt(keystore));

		(t, pool_state, offchain_state)
	}
}

#[test]
fn submit_number_signed_works() {
	let (mut t, _, _) = ExtBuilder::build();
	t.execute_with(|| {
		// call submit_number_signed
		let num = 32;
		let acct: <TestRuntime as system::Trait>::AccountId = Default::default();
		assert_ok!(OffchainDemo::submit_number_signed(Origin::signed(acct), num));
		// A number is inserted to <Numbers> vec
		assert_eq!(<Numbers>::get(), vec![num]);
		// storage <NextTx> is incremented
		assert_eq!(<NextTx<TestRuntime>>::get(), <TestRuntime as Trait>::GracePeriod::get());
		// AddSeq is incremented
		assert_eq!(<AddSeq>::get(), 1);
		// An event is emitted
		assert!(System::events().iter().any(|er| er.event ==
			TestEvent::offchain_demo(RawEvent::NewNumber(Some(acct), num))));

		// Insert another number
		let num2 = num * 2;
		assert_ok!(OffchainDemo::submit_number_signed(Origin::signed(acct), num2));
		// A number is inserted to <Numbers> vec
		assert_eq!(<Numbers>::get(), vec![num, num2]);
		// storage <NextTx> is incremented
		assert_eq!(<NextTx<TestRuntime>>::get(), <TestRuntime as Trait>::GracePeriod::get() * 2);
		// AddSeq is incremented
		assert_eq!(<AddSeq>::get(), 2);
	});
}

#[test]
fn offchain_send_signed_tx() {
	let (mut t, pool_state, _offchain_state) = ExtBuilder::build();

	t.execute_with(|| {
		// when
		let num = 32;
		OffchainDemo::send_signed(num).unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		assert!(pool_state.read().transactions.is_empty());
		let tx = TestExtrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature.unwrap().0, 0);
		assert_eq!(tx.call, Call::submit_number_signed(num));
	});
}

#[test]
fn offchain_send_unsigned_tx() {
	let (mut t, pool_state, _offchain_state) = ExtBuilder::build();

	t.execute_with(|| {
		// when
		let num = 32;
		OffchainDemo::send_unsigned(num).unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		assert!(pool_state.read().transactions.is_empty());
		let tx = TestExtrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		assert_eq!(tx.call, Call::submit_number_unsigned(num, num));
	});
}
