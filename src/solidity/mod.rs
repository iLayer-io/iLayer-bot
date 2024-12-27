use alloy::sol;
use chrono::DateTime;
use eyre::Result;

use crate::orm::models;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Counter,
    "abi/Counter.json"
);

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
) -> Result<models::NewOrder> {
    // TODO FIXME: Improve error handling
    let mut user = order.user.lower.to_vec();
    user.extend(order.user.upper.iter());

    let mut filler = order.filler.lower.to_vec();
    filler.extend(order.filler.upper.iter());

    let mut call_recipient = order.callRecipient.lower.to_vec();
    call_recipient.extend(order.callRecipient.upper.iter());

    let call_data = order.callData.to_vec();

    let primary_filler_deadline_vec = order.primaryFillerDeadline.as_le_bytes().to_vec();
    if primary_filler_deadline_vec.len() != 8 {
        return Err(eyre::eyre!(
            "Invalid primary deadline length, it must be 8 bytes to be a valid Unix timestamp"
        ));
    }
    let primary_filler_deadline_u64 =
        u64::from_le_bytes(primary_filler_deadline_vec.try_into().unwrap());
    let primary_filler_deadline = DateTime::from_timestamp(primary_filler_deadline_u64 as i64, 0);

    let deadline_vec = order.deadline.as_le_bytes().to_vec();
    if deadline_vec.len() != 8 {
        return Err(eyre::eyre!(
            "Invalid primary deadline length, it must be 8 bytes to be a valid Unix timestamp"
        ));
    }
    let deadline_u64 = u64::from_le_bytes(deadline_vec.try_into().unwrap());
    let deadline = DateTime::from_timestamp(deadline_u64 as i64, 0);

    Ok(models::NewOrder {
        user: user,
        order_id: order_id,
        filler: filler,
        source_chain_selector: order.sourceChainSelector.as_le_bytes().to_vec(),
        destination_chain_selector: order.destinationChainSelector.as_le_bytes().to_vec(),
        sponsored: false,
        primary_filler_deadline: primary_filler_deadline.expect("Unable to deserialize DateTime"),
        deadline: deadline.expect("Unable to deserialize DateTime"),
        call_recipient: call_recipient,
        call_data: call_data,
    })
}

impl std::fmt::Debug for Validator::Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Order")
            .field("sourceChainSelector", &self.sourceChainSelector)
            .field("filler", &self.filler)
            .field("primaryFillerDeadline", &self.primaryFillerDeadline)
            .field("deadline", &self.deadline)
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

sol!(
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    event OrderCreated(uint256 chainId, uint256 coinId, uint256 amount);
);

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use alloy::{
        primitives::{Bytes, FixedBytes, LogData, Uint},
        sol_types::SolEvent,
    };

    use super::*;

    #[test]
    fn test_ordercreated_signature() {
        assert_eq!(
            OrderCreated::SIGNATURE,
            "OrderCreated(uint256,uint256,uint256)"
        );
    }

    #[test]
    fn test_ordercreated_decode() {
        let topic: FixedBytes<32> = FixedBytes::from_str(
            "0x7e3297793a932665ad789941fefa66afb013f1e1dad602d7738f5ebf607b173b",
        )
        .unwrap();
        let topics: Vec<FixedBytes<32>> = vec![topic];
        let data = Bytes::from_str("0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000029a").ok().unwrap();

        let log_data = LogData::new(topics.clone(), data).unwrap();

        let parsed = OrderCreated::decode_raw_log(topics.clone(), &log_data.data, false);

        assert_eq!(
            parsed,
            Ok(OrderCreated {
                chainId: Uint::from(1),
                coinId: Uint::from(1),
                amount: Uint::from(666),
            })
        );
    }
}
