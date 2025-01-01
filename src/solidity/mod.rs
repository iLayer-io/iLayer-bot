use alloy::sol;
use eyre::Result;

use crate::dao::models;

// TODO Wait for the fix to be tagged, then remove this.
sol!(
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    struct _bytes64 {
        bytes32 lower;
        bytes32 upper;
    }
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Orderbook,
    "abi/Orderbook.abi.json"
);

pub fn map_solidity_order_to_model(
    order_id: Vec<u8>,
    order: &Validator::Order,
) -> Result<models::Order> {
    // TODO FIXME: Improve error handling
    let mut user = order.user.lower.to_vec();
    user.extend(order.user.upper.iter());

    let mut filler = order.filler.lower.to_vec();
    filler.extend(order.filler.upper.iter());

    let mut call_recipient = order.callRecipient.lower.to_vec();
    call_recipient.extend(order.callRecipient.upper.iter());

    let call_data = order.callData.to_vec();

    Ok(models::Order {
        user: user,
        id: order_id,
        filler: filler,
        source_chain_selector: order.sourceChainSelector.as_le_bytes().to_vec(),
        destination_chain_selector: order.destinationChainSelector.as_le_bytes().to_vec(),
        sponsored: order.sponsored,
        // TODO Map deadlines to DateTime
        primary_filler_deadline: order.primaryFillerDeadline.to(),
        deadline: order.deadline.to(),
        call_recipient: call_recipient,
        call_data: call_data, 
        // TODO FIXME add inputs and outputs
    })
}

impl std::fmt::Debug for Validator::Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Order")
            .field("sourceChainSelector", &self.sourceChainSelector)
            .field("filler", &self.filler)
            .field("primaryFillerDeadline", &self.primaryFillerDeadline)
            .field("deadline", &self.deadline)
            .field("user", &self.user)
            .field("callRecipient", &self.callRecipient)
            .field("callData", &self.callData)
            .finish()
    }
}
impl std::fmt::Debug for Orderbook::OrderCreated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderCreated")
            .field("order", &self.order)
            .field("orderId", &self.orderId)
            .finish()
    }
}

impl std::fmt::Debug for Orderbook::OrderWithdrawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderWithdrawn")
            .field("caller", &self.caller)
            .field("orderId", &self.orderId)
            .finish()
    }
}

impl std::fmt::Debug for Orderbook::OrderFilled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderFilled")
            .field("filler", &self.filler)
            .field("orderId", &self.orderId)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        primitives::{Address, Bytes, FixedBytes, Log}, signers::local::PrivateKeySigner, sol_types::SolEvent
    };
    use std::str::FromStr;

    use super::{map_solidity_order_to_model, Orderbook};

    #[test]
    fn test_ordercreated_decode() {
        let topics: Vec<FixedBytes<32>> = vec![
            FixedBytes::from_str(
                "0x1f3e9ee381e3de37fa4a5d5d5e8e320b51fd6547b591c80a169dbcf6160878e3",
            )
            .unwrap(),
            FixedBytes::from_str(
                "0x777a108f0d7d6ef99218eb59bc1900ed56d401db4fc9bbff76d85c68c5cb0168",
            )
            .unwrap(),
        ];
        let data = Bytes::from_str(
            "0x000000000000000000000000a0ee7a142d267c1f36714e4a8f75612f20a7972\
            0000000000000000000000000000000000000000000000000000000000000006000\
            0000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000a0ee7a142d267c1f36714e4a8f75612f20a7972000000000\
            0000000000000000000000000000000000000000000000000000000000000000000\
            000000000000023618e81e3f5cdf7f54c3d65f7fbc0abf5b21e8f00000000000000\
            0000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000001c000000000000000000000\
            0000000000000000000000000000000000000000026000000000000000000000000\
            00000000000000000000000000000000000007a6900000000000000000000000000\
            00000000000000000000000000000000007a6900000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000006772a62200000000000000000000000000000000000\
            00000000000000000000068a3ac1200000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000030000000000000000000000000000000000000000000000000\
            00000000000000001000000000000000000000000700b6a60ce7eaaea56f065753d\
            8dcb9653dbad3500000000000000000000000000000000000000000000000000000\
            00000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffff\
            ffffffff0000000000000000000000000000000000000000000000000de0b6b3a76\
            4000000000000000000000000000000000000000000000000000000000000000000\
            01000000000000000000000000a15bb66138824a1c7167f5e85b957d04dd34e4680\
            000000000000000000000000000000000000000000000000000000000000000ffff\
            ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000\
            000000000000000000000000000000000000000000de0b6b3a76400000000000000\
            000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let address = Address::from_str("0x8ce361602b935680e8dec218b820ff5056beb7af").unwrap();
        let log = Log::new(address, topics, data).unwrap();
        let order_created = Orderbook::OrderCreated::decode_log(&log, false).unwrap();
        let actual: crate::dao::models::Order = map_solidity_order_to_model(
            "0x777a108f0d7d6ef99218eb59bc1900ed56d401db4fc9bbff76d85c68c5cb0168"
                .as_bytes()
                .to_vec(),
            &order_created.order,
        )
        .unwrap();

        let id = "0x777a108f0d7d6ef99218eb59bc1900ed56d401db4fc9bbff76d85c68c5cb0168"
            .as_bytes()
            .to_vec();
        let user = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 238, 122, 20, 45, 38, 124, 31, 54, 113, 78,
            74, 143, 117, 97, 47, 32, 167, 151, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let filler = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35, 97, 142, 129, 227, 245, 205, 247, 245, 76, 61,
            101, 247, 251, 192, 171, 245, 178, 30, 143, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let destination_chain_selector = vec![
            105, 122, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let source_chain_selector = vec![
            105, 122, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let sponsored = false;
        let primary_filler_deadline = 1735566882;
        let deadline = 1755556882;
        let call_recipient = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let call_data = vec![];

        let expected = crate::dao::models::Order {
            user: user,
            id: id,
            filler: filler,
            source_chain_selector: source_chain_selector,
            destination_chain_selector: destination_chain_selector,
            sponsored: sponsored,
            primary_filler_deadline: primary_filler_deadline,
            deadline: deadline,
            call_recipient: call_recipient,
            call_data: call_data,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_priv_key_to_address_conversion() {
        // This is a placeholder for the actual test implementation.
        // You would need to implement the logic to convert a private key to an address
        // and then verify the conversion is correct.
        let priv_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";
        let expected_address_str = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";

        // Implement the conversion logic here
        let signer = PrivateKeySigner::from_str(priv_key).unwrap();
        let actual_address_str = signer.address().to_string();

        assert_eq!(actual_address_str, expected_address_str);
    }

    #[test]
    fn test_address_from_vec8() {
        let filler = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35, 97, 142, 129, 227, 245, 205, 247, 245, 76, 61,
            101, 247, 251, 192, 171, 245, 178, 30, 143, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected_address = Address::from_slice(&filler[12..32]);

        let priv_key = "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97";
        let signer = PrivateKeySigner::from_str(priv_key).unwrap();
        let actual_address = signer.address();

        assert_eq!(actual_address, expected_address);
    }
}
