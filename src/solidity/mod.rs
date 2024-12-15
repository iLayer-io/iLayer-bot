use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    event OrderCreated(uint256 chainId, uint256 coinId, uint256 amount);
);


#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use alloy::{primitives::{Bytes, FixedBytes, LogData, Uint}, sol_types::SolEvent};

    use super::*;
    
    #[test]
    fn test_ordercreated_signature() {
        assert_eq!(OrderCreated::SIGNATURE, "OrderCreated(uint256,uint256,uint256)");
    }

    
    #[test]
    fn test_ordercreated_decode() {
        let topic: FixedBytes<32> = FixedBytes::from_str("0x7e3297793a932665ad789941fefa66afb013f1e1dad602d7738f5ebf607b173b").unwrap();
        let  topics: Vec<FixedBytes<32>> =vec![topic];
        let data = Bytes::from_str("0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000029a").ok().unwrap();

        let log_data= LogData::new(
            topics.clone(),
            data
        ).unwrap();
        
        let parsed = OrderCreated::decode_raw_log(topics.clone(), &log_data.data, false);

        assert_eq!(parsed, Ok(OrderCreated {
            chainId: Uint::from(1),
            coinId: Uint::from(1),
            amount: Uint::from(666),
        }));
    }

    
    
}