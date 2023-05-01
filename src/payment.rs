use cosmwasm_std::{Coin, MessageInfo, Uint128};
use std::collections::HashSet;
use thiserror::Error;

/// returns an error if any coins were sent
pub fn nonpayable(info: &MessageInfo) -> Result<(), PaymentError> {
    if info.funds.is_empty() {
        Ok(())
    } else {
        Err(PaymentError::NonPayable {})
    }
}

/// If exactly one coin was sent, returns it regardless of denom.
/// Returns error if 0 or 2+ coins were sent
pub fn one_coin(info: &MessageInfo) -> Result<Coin, PaymentError> {
    match info.funds.len() {
        0 => Err(PaymentError::NoFunds {}),
        1 => {
            let coin = &info.funds[0];
            if coin.amount.is_zero() {
                Err(PaymentError::NoFunds {})
            } else {
                Ok(coin.clone())
            }
        }
        _ => Err(PaymentError::MultipleDenoms {}),
    }
}

/// Requires exactly one denom sent, which matches the requested denom.
/// Returns the amount if only one denom and non-zero amount. Errors otherwise.
pub fn must_pay(info: &MessageInfo, denom: &str) -> Result<Uint128, PaymentError> {
    let coin = one_coin(info)?;
    if coin.denom != denom {
        Err(PaymentError::MissingDenom(denom.to_string()))
    } else {
        Ok(coin.amount)
    }
}

/// Similar to must_pay, but it any payment is optional. Returns an error if a different
/// denom was sent. Otherwise, returns the amount of `denom` sent, or 0 if nothing sent.
pub fn may_pay(info: &MessageInfo, denom: &str) -> Result<Uint128, PaymentError> {
    if info.funds.is_empty() {
        Ok(Uint128::zero())
    } else if info.funds.len() == 1 && info.funds[0].denom == denom {
        Ok(info.funds[0].amount)
    } else {
        // find first mis-match
        let wrong = info.funds.iter().find(|c| c.denom != denom).unwrap();
        Err(PaymentError::ExtraDenom(wrong.denom.to_string()))
    }
}

fn stringify_coins(coins: &[Coin]) -> String {
    match coins {
        [] => "None".to_string(),
        _ => coins
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Assert that exact coins & amount were sent along with a message
pub fn exact_funds_sent(info: &MessageInfo, expected: &[Coin]) -> Result<(), PaymentError> {
    let same_quantity = info.funds.len() == expected.len();
    let expected_hash_set: HashSet<_> = expected.iter().map(|c| c.to_string()).collect();
    let message_funds_hash_set: HashSet<_> = info.funds.iter().map(|c| c.to_string()).collect();

    if !same_quantity || expected_hash_set != message_funds_hash_set {
        return Err(PaymentError::FundsMismatch {
            expected: stringify_coins(expected),
            received: stringify_coins(&info.funds),
        });
    }
    Ok(())
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PaymentError {
    #[error("Must send reserve token '{0}'")]
    MissingDenom(String),

    #[error("Received unsupported denom '{0}'")]
    ExtraDenom(String),

    #[error("Sent more than one denomination")]
    MultipleDenoms {},

    #[error("No funds sent")]
    NoFunds {},

    #[error("This message does no accept funds")]
    NonPayable {},

    #[error("Sent funds mismatch. Expected: {expected:?}, received {received:?}")]
    FundsMismatch { expected: String, received: String },
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{coin, coins};

    const SENDER: &str = "sender";

    #[test]
    fn nonpayable_works() {
        let no_payment = mock_info(SENDER, &[]);
        nonpayable(&no_payment).unwrap();

        let payment = mock_info(SENDER, &coins(100, "uatom"));
        let res = nonpayable(&payment);
        assert_eq!(res.unwrap_err(), PaymentError::NonPayable {});
    }

    #[test]
    fn may_pay_works() {
        let atom: &str = "uatom";
        let no_payment = mock_info(SENDER, &[]);
        let atom_payment = mock_info(SENDER, &coins(100, atom));
        let eth_payment = mock_info(SENDER, &coins(100, "wei"));
        let mixed_payment = mock_info(SENDER, &[coin(50, atom), coin(120, "wei")]);

        let res = may_pay(&no_payment, atom).unwrap();
        assert_eq!(res, Uint128::zero());

        let res = may_pay(&atom_payment, atom).unwrap();
        assert_eq!(res, Uint128::new(100));

        let err = may_pay(&eth_payment, atom).unwrap_err();
        assert_eq!(err, PaymentError::ExtraDenom("wei".to_string()));

        let err = may_pay(&mixed_payment, atom).unwrap_err();
        assert_eq!(err, PaymentError::ExtraDenom("wei".to_string()));
    }

    #[test]
    fn must_pay_works() {
        let atom: &str = "uatom";
        let no_payment = mock_info(SENDER, &[]);
        let atom_payment = mock_info(SENDER, &coins(100, atom));
        let zero_payment = mock_info(SENDER, &coins(0, atom));
        let eth_payment = mock_info(SENDER, &coins(100, "wei"));
        let mixed_payment = mock_info(SENDER, &[coin(50, atom), coin(120, "wei")]);

        let res = must_pay(&atom_payment, atom).unwrap();
        assert_eq!(res, Uint128::new(100));

        let err = must_pay(&no_payment, atom).unwrap_err();
        assert_eq!(err, PaymentError::NoFunds {});

        let err = must_pay(&zero_payment, atom).unwrap_err();
        assert_eq!(err, PaymentError::NoFunds {});

        let err = must_pay(&eth_payment, atom).unwrap_err();
        assert_eq!(err, PaymentError::MissingDenom(atom.to_string()));

        let err = must_pay(&mixed_payment, atom).unwrap_err();
        assert_eq!(err, PaymentError::MultipleDenoms {});
    }

    #[test]
    fn exact_funds_sent_success() {
        let message_info = mock_info(
            SENDER,
            &[coin(50, "uosmo"), coin(42, "umars"), coin(120, "uatom")],
        );
        let expected = vec![coin(50, "uosmo"), coin(42, "umars"), coin(120, "uatom")];
        exact_funds_sent(&message_info, &expected).unwrap();

        // Re-order does not matter
        let expected = vec![coin(42, "umars"), coin(50, "uosmo"), coin(120, "uatom")];
        exact_funds_sent(&message_info, &expected).unwrap();

        // When sent & expected none
        let message_info = mock_info(SENDER, &[]);
        exact_funds_sent(&message_info, &[]).unwrap();
    }

    #[test]
    fn exact_funds_sent_when_expected_more() {
        // When sent none
        let message_info = mock_info(SENDER, &[]);
        let expected = vec![coin(50, "uosmo"), coin(120, "uatom")];
        let err = exact_funds_sent(&message_info, &expected).unwrap_err();
        assert_eq!(
            err,
            PaymentError::FundsMismatch {
                expected: "50uosmo, 120uatom".to_string(),
                received: "None".to_string()
            }
        );

        // When sent all correct, but missing one
        let message_info = mock_info(SENDER, &[coin(50, "uosmo"), coin(120, "uatom")]);
        let expected = vec![coin(50, "uosmo"), coin(120, "uatom"), coin(42, "umars")];
        let err = exact_funds_sent(&message_info, &expected).unwrap_err();
        assert_eq!(
            err,
            PaymentError::FundsMismatch {
                expected: "50uosmo, 120uatom, 42umars".to_string(),
                received: "50uosmo, 120uatom".to_string()
            }
        );
    }

    #[test]
    fn exact_funds_sent_when_expected_less() {
        // When expected none
        let message_info = mock_info(SENDER, &[coin(50, "uosmo"), coin(120, "uatom")]);
        let err = exact_funds_sent(&message_info, &[]).unwrap_err();
        assert_eq!(
            err,
            PaymentError::FundsMismatch {
                expected: "None".to_string(),
                received: "50uosmo, 120uatom".to_string()
            }
        );

        // When sent one extra
        let message_info = mock_info(SENDER, &[coin(50, "uosmo"), coin(120, "uatom")]);
        let expected = vec![coin(120, "uatom")];
        let err = exact_funds_sent(&message_info, &expected).unwrap_err();
        assert_eq!(
            err,
            PaymentError::FundsMismatch {
                expected: "120uatom".to_string(),
                received: "50uosmo, 120uatom".to_string()
            }
        );
    }

    #[test]
    fn exact_funds_sent_when_expected_different_coins() {
        let message_info = mock_info(SENDER, &[coin(50, "uosmo"), coin(120, "uatom")]);
        let expected = vec![coin(42, "umars"), coin(11, "uxyz")];
        let err = exact_funds_sent(&message_info, &expected).unwrap_err();
        assert_eq!(
            err,
            PaymentError::FundsMismatch {
                expected: "42umars, 11uxyz".to_string(),
                received: "50uosmo, 120uatom".to_string()
            }
        );
    }

    #[test]
    fn exact_funds_sent_when_duplicates() {
        let message_info = mock_info(SENDER, &[coin(50, "uosmo"), coin(120, "uatom")]);
        let expected = vec![coin(120, "uatom"), coin(120, "uatom")];
        let err = exact_funds_sent(&message_info, &expected).unwrap_err();
        assert_eq!(
            err,
            PaymentError::FundsMismatch {
                expected: "120uatom, 120uatom".to_string(),
                received: "50uosmo, 120uatom".to_string()
            }
        );

        let message_info = mock_info(SENDER, &[coin(120, "uatom"), coin(120, "uatom")]);
        let expected = vec![coin(50, "uosmo"), coin(120, "uatom")];
        let err = exact_funds_sent(&message_info, &expected).unwrap_err();
        assert_eq!(
            err,
            PaymentError::FundsMismatch {
                expected: "50uosmo, 120uatom".to_string(),
                received: "120uatom, 120uatom".to_string()
            }
        );
    }
}
