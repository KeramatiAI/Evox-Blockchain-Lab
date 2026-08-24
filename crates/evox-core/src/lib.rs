// use serde::{Serialize, Deserialize};
// use std::collections::HashSet;
//
// /// نوع مختلف تراکنش در شبکه EvoX
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub enum TransactionType {
//     Transfer,        // انتقال ساده دارایی
//     ContractCall,    // فراخوانی یک قرارداد هوشمند
//     Governance,      // تراکنش‌های مربوط به حاکمیت شبکه
// }
//
// /// ساختار اصلی تراکنش در EvoX
// /// طراحی شده برای سازگاری با اجرای موازی (Parallel Execution)
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct Transaction {
//     // 1. شناسه‌های پایه
//     pub sender: Vec<u8>,      // آدرس فرستنده (به صورت باینری برای سرعت بیشتر)
//     pub nonce: u64,           // برای جلوگیری از حملات Replay
//     pub tx_type: TransactionType,
//
//     // 2. داده‌های اصلی (Payload)
//     pub payload: Vec<u8>,     // داده‌های مربوط به انتقال یا پارامترهای قرارداد
//
//     // 3. بخش کلیدی برای Parallelism: Access List
//     // این لیست شامل تمام آدرس‌هایی است که این تراکنش قصد تغییر دادن آن‌ها را دارد.
//     // موتور اجرا با نگاه کردن به این لیست، متوجه می‌شود که آیا این تراکنش
//     // با تراکنش‌های دیگر تداخل دارد یا خیر.
//     pub access_list: HashSet<Vec<u8>>,
//
//     // 4. امنیت
//     pub signature: Vec<u8>,    // امضای دیجیتال تراکنش
// }
//
// impl Transaction {
//     /// ایجاد یک تراکنش ساده برای تست
//     pub fn new_transfer(sender: Vec<u8>, receiver: Vec<u8>, amount: u64) -> Self {
//         let mut access_list = HashSet::new();
//         access_list.insert(sender.clone());
//         access_list.insert(receiver.clone());
//
//         Self {
//             sender,
//             nonce: 0,
//             tx_type: TransactionType::Transfer,
//             payload: amount.to_be_bytes().to_vec(), // تبدیل عدد به بایت
//             access_list,
//             signature: vec![], // فعلاً خالی برای مرحله طراحی
//         }
//     }
// }
//
// pub fn check_core() -> &'static str {
//     "EvoX Core is operational"
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_core_connection() {
//         assert_eq!(check_core(), "EvoX Core is operational");
//     }
//
//     #[test]
//     fn test_transaction_creation() {
//         let sender = b"alice_address".to_vec();
//         let receiver = b"bob_address".to_vec();
//         let tx = Transaction::new_transfer(sender.clone(), receiver.clone(), 100);
//
//         assert_eq!(tx.tx_type, TransactionType::Transfer);
//         assert!(tx.access_list.contains(&sender));
//         assert!(tx.access_list.contains(&receiver));
//         assert_eq!(tx.payload, 100u64.to_be_bytes().to_vec());
//     }
// }

use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use evox_crypto::{sign, verify, SigningKey, VerifyingKey, CryptoError};
// use ed25519_dalek::Signature;
use evox_crypto::Signature;

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
    pub signature: Vec<u8>, // امضا به صورت بایت ذخیره می‌شود
}

impl Transaction {
    /// ایجاد یک تراکنش انتقال ساده (بدون امضا در ابتدا)
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
            signature: vec![], // ابتدا خالی است
        }
    }

    /// داده‌های اصلی تراکنش را برای امضا آماده می‌کند (بدون فیلد signature)
    /// این تابع برای جلوگیری از تداخل، داده‌ها را سریالایز می‌کند
    fn get_signing_data(&self) -> Vec<u8> {
        // ما یک نسخه از خود را بدون فیلد امضا می‌سازیم تا امضا کنیم
        // برای سادگی در این مرحله، از bincode استفاده می‌کنیم
        // در نسخه نهایی، این بخش بسیار بهینه خواهد بود
        let mut temp_tx = self.clone();
        temp_tx.signature = vec![];
        bincode::serialize(&temp_tx).unwrap_or_default()
    }

    /// امضا کردن تراکنش با استفاده از کلید خصوصی
    pub fn sign_transaction(&mut self, signing_key: &SigningKey) {
        let data = self.get_signing_data();
        let signature = sign(signing_key, &data);
        self.signature = signature.to_bytes().to_vec();
    }

    /// تأیید صحت تراکنش با استفاده از کلید عمومی فرستنده
    pub fn verify_transaction(&self, public_key: &VerifyingKey) -> Result<(), CryptoError> {
        if self.signature.is_empty() {
            return Err(CryptoError::InvalidSignature);
        }

        let data = self.get_signing_data();

        // تبدیل بایت‌های ذخیره شده به ساختار Signature
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
    fn test_full_transaction_lifecycle() {
        // 1. تولید کلیدها
        let keypair = generate_keypair();
        // let sender_address = keypair.verifying_key().to_bytes().to_vec();
        let sender_address = keypair.verifying_key.to_bytes().to_vec();

        let receiver_address = vec![1, 2, 3, 4];

        // 2. ایجاد تراکنش
        let mut tx = Transaction::new_transfer(
            sender_address.clone(),
            receiver_address,
            100
        );

        // 3. امضا کردن تراکنش
        tx.sign_transaction(&keypair.signing_key);
        assert!(!tx.signature.is_empty());

        // 4. تأیید تراکنش (باید موفق باشد)
        let verification_result = tx.verify_transaction(&keypair.verifying_key);
        assert!(verification_result.is_ok());

        // 5. تست دستکاری داده‌ها (Tampering)
        // تغییر مقدار Payload (مثلاً از 100 به 200)
        tx.payload = 200u64.to_be_bytes().to_vec();
        let tampered_result = tx.verify_transaction(&keypair.verifying_key);
        assert!(tampered_result.is_err());

        println!("Lifecycle test passed: Transaction signed and verified successfully!");
    }
}

