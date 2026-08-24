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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub accounts: HashMap<Vec<u8>, Account>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_account(&mut self, address: Vec<u8>, initial_balance: Balance) {
        let account = Account {
            address: address.clone(),
            balance: initial_balance,
        };
        self.accounts.insert(address, account);
    }

    pub fn get_balance(&self, address: &[u8]) -> Option<Balance> {
        self.accounts.get(address).map(|acc| acc.balance)
    }

    /// قلب تپنده: اعمال تراکنش روی State
    pub fn apply_transaction(&mut self, tx: &Transaction, public_key: &VerifyingKey) -> Result<(), String> {
        // 1. بررسی امضا
        tx.verify_transaction(public_key).map_err(|e| format!("Crypto Error: {:?}", e))?;

        // 2. استخراج اطلاعات تراکنش از Payload با استفاده از bincode
        let sender_addr = tx.sender.clone();
        let amount: u64 = bincode::deserialize(&tx.payload)
            .map_err(|_| "Invalid payload format".to_string())?;

        // پیدا کردن گیرنده از access_list (اولین آدرس غیر از فرستنده)
        let receiver_addr = tx.access_list.iter()
            .find(|&addr| addr != &sender_addr)
            .ok_or("No receiver found in access list".to_string())?
            .clone();

        // 3. بررسی موجودی فرستنده
        let sender_balance = self.get_balance(&sender_addr)
            .ok_or("Sender account not found".to_string())?;

        if sender_balance < amount {
            return Err("Insufficient balance".to_string());
        }

        // 4. اجرای انتقال (Atomic update)
        // ابتدا موجودی فرستنده را کم می‌کنیم
        if let Some(sender_acc) = self.accounts.get_mut(&sender_addr) {
            sender_acc.balance -= amount;
        }

        // سپس به گیرنده اضافه می‌کنیم (اگر وجود نداشت، بساز)
        let receiver_acc = self.accounts.entry(receiver_addr.clone()).or_insert(Account {
            address: receiver_addr,
            balance: 0,
        });
        receiver_acc.balance += amount;

        Ok(())
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
    pub fn new_transfer(sender: Vec<u8>, receiver: Vec<u8>, amount: u64) -> Self {
        let mut access_list = HashSet::new();
        access_list.insert(sender.clone());
        access_list.insert(receiver.clone());

        // اصلاح اصلی: استفاده از bincode برای ذخیره مبلغ جهت سازگاری با deserialize
        let payload = bincode::serialize(&amount).unwrap_or_default();

        Self {
            sender,
            nonce: 0,
            tx_type: TransactionType::Transfer,
            payload,
            access_list,
            signature: vec![],
        }
    }

    fn get_signing_data(&self) -> Vec<u8> {
        let mut temp_tx = self.clone();
        temp_tx.signature = vec![];
        bincode::serialize(&temp_tx).unwrap_or_default()
    }

    pub fn sign_transaction(&mut self, signing_key: &SigningKey) {
        let data = self.get_signing_data();
        let signature = sign(signing_key, &data);
        self.signature = signature.to_bytes().to_vec();
    }

    pub fn verify_transaction(&self, public_key: &VerifyingKey) -> Result<(), CryptoError> {
        if self.signature.is_empty() {
            return Err(CryptoError::InvalidSignature);
        }

        let data = self.get_signing_data();

        // تبدیل Vec به آرایه ثابت [u8; 64] برای کتابخانه رمزنگاری
        let sig_bytes: [u8; 64] = self.signature.clone()
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?;

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
    }

    #[test]
    fn test_full_transaction_lifecycle_and_state_update() {
        let keypair = generate_keypair();
        let sender_addr = keypair.verifying_key.to_bytes().to_vec();
        let receiver_addr = vec![4, 5, 6, 7];

        // 1. آماده‌سازی State
        let mut state = State::new();
        state.create_account(sender_addr.clone(), 500);

        // 2. ایجاد و امضای تراکنش
        let mut tx = Transaction::new_transfer(
            sender_addr.clone(),
            receiver_addr.clone(),
            200
        );
        tx.sign_transaction(&keypair.signing_key);

        // 3. بررسی صحت امضا قبل از اعمال
        tx.verify_transaction(&keypair.verifying_key).expect("Signature verification failed before applying");

        // 4. اعمال تراکنش روی State
        state.apply_transaction(&tx, &keypair.verifying_key).expect("Failed to apply transaction");

        // 5. بررسی تغییرات موجودی
        assert_eq!(state.get_balance(&sender_addr), Some(300));
        assert_eq!(state.get_balance(&receiver_addr), Some(200));

        // 6. تست تراکنش با موجودی ناکافی
        let mut tx_too_expensive = Transaction::new_transfer(
            sender_addr.clone(),
            receiver_addr,
            1000
        );
        tx_too_expensive.sign_transaction(&keypair.signing_key);
        let err_result = state.apply_transaction(&tx_too_expensive, &keypair.verifying_key);
        assert!(err_result.is_err());
        assert_eq!(err_result.unwrap_err(), "Insufficient balance");
    }
}
