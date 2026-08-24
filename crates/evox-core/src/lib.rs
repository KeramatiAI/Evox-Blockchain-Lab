use serde::{Serialize, Deserialize};
use std::collections::{HashSet, HashMap};
use evox_crypto::{sign, verify, SigningKey, VerifyingKey, CryptoError, Signature};

/// نمایش موجودی اکانت‌ها
pub type Balance = u64;

/// ساختار اکانت در شبکه
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: Vec<u8>,
    pub balance: Balance,
}

/// مدیریت وضعیت کل شبکه (State)
#[derive(Default, Debug, Clone)]
pub struct State {
    pub accounts: HashMap<Vec<u8>, Account>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    /// اضافه کردن یک اکانت جدید (برای تست یا Genesis)
    pub fn create_account(&mut self, address: Vec<u8>, initial_balance: Balance) {
        let account = Account {
            address: address.clone(),
            balance: initial_balance,
        };
        self.accounts.insert(address, account);
    }

    /// دریافت موجودی یک اکانت
    pub fn get_balance(&self, address: &[u8]) -> Option<Balance> {
        self.accounts.get(address).map(|acc| acc.balance)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Transfer,
    ContractCall,
    Governance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub sender: Vec<u8>,
    pub nonce: u64,
    pub tx_type: TransactionType,
    pub payload: Vec<u8>,
    pub access_list: HashSet<Vec<u8>>,
    pub signature: Vec<u8>,
}

impl Transaction {
    /// ایجاد یک تراکنش انتقال ساده
    pub fn new_transfer(sender: Vec<u8>, receiver: Vec<u8>, amount: u64) -> Self {
        let mut access_list = HashSet::new();
        access_list.insert(sender.clone());
        access_list.insert(receiver.clone());

        Self {
            sender,
            nonce: 0,
            tx_type: TransactionType::Transfer,
            payload: amount.to_be_bytes().to_vec(),
            access_list,
            signature: vec![],
        }
    }

    /// آماده‌سازی داده‌ها برای امضا (بدون فیلد signature)
    fn get_signing_data(&self) -> Vec<u8> {
        let mut temp_tx = self.clone();
        temp_tx.signature = vec![];
        bincode::serialize(&temp_tx).unwrap_or_default()
    }

    /// امضا کردن تراکنش
    pub fn sign_transaction(&mut self, signing_key: &SigningKey) {
        let data = self.get_signing_data();
        let signature = sign(signing_key, &data);
        self.signature = signature.to_bytes().to_vec();
    }

    /// تأیید صحت تراکنش
    pub fn verify_transaction(&self, public_key: &VerifyingKey) -> Result<(), CryptoError> {
        if self.signature.is_empty() {
            return Err(CryptoError::InvalidSignature);
        }

        let data = self.get_signing_data();

        // تبدیل بایت‌ها به آرایه 64 بایتی برای Signature
        let sig_bytes: [u8; 64] = self.signature.clone()
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?;

        // تبدیل آرایه به شیء Signature
        let signature = Signature::from_bytes(&sig_bytes);

        verify(public_key, &data, &signature)
    }
}

pub fn check_core() -> &'static str {
    "EvoX Core is operational"
}

#[cfg(test)]
mod tests {
    use super::*;
    use evox_crypto::generate_keypair;

    #[test]
    fn test_core_connection() {
        assert_eq!(check_core(), "EvoX Core is operational");
    }

    #[test]
    fn test_state_management() {
        let mut state = State::new();
        let addr = vec![1, 2, 3];
        state.create_account(addr.clone(), 1000);

        assert_eq!(state.get_balance(&addr), Some(1000));
        assert_eq!(state.get_balance(&vec![9, 9]), None);
    }

    #[test]
    fn test_full_transaction_lifecycle() {
        let keypair = generate_keypair();
        let sender_address = keypair.verifying_key.to_bytes().to_vec();
        let receiver_address = vec![4, 5, 6, 7];

        let mut tx = Transaction::new_transfer(
            sender_address.clone(),
            receiver_address,
            100
        );

        tx.sign_transaction(&keypair.signing_key);
        assert!(!tx.signature.is_empty());

        let verification_result = tx.verify_transaction(&keypair.verifying_key);
        assert!(verification_result.is_ok());

        tx.payload = 200u64.to_be_bytes().to_vec();
        let tampered_result = tx.verify_transaction(&keypair.verifying_key);
        assert!(tampered_result.is_err());
    }
}
