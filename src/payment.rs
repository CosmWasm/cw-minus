use cosmwasm_std::{Coin, MessageInfo, Uint128};
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

pub fn must_pay_many(
    info: &MessageInfo,
    required_denoms: &[&str],
) -> Result<Vec<Uint128>, PaymentError> {
    if info.funds.is_empty() {
        return Err(PaymentError::NoFunds {});
    }

    if info.funds.len() != required_denoms.len() {
        return Err(PaymentError::IncorrectNumberOfDenoms {});
    }

    let mut amounts: Vec<Uint128> = Vec::new();

    for denom in required_denoms.iter() {
        match info.funds.iter().find(|c| &c.denom == denom) {
            Some(coin) => {
                if coin.amount == Uint128::zero() {
                    return Err(PaymentError::NoFunds {});
                }
                amounts.push(coin.amount);
            }
            None => return Err(PaymentError::MissingDenom(denom.to_string())),
        }
    }

    Ok(amounts)
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
    fn must_pay_two_coins_works() {
        let atom: &str = "uatom";
        let osmo: &str = "uosmo";
        let eth: &str = "eth";
        let no_payment = mock_info(SENDER, &[]);
        let atom_payment = mock_info(SENDER, &coins(100, atom));
        let osmo_payment = mock_info(SENDER, &coins(100, osmo));
        let two_coin_payment = mock_info(SENDER, &[coin(50, atom), coin(120, eth)]);
        let duplicate_coins_payment = mock_info(SENDER, &[coin(50, atom), coin(120, atom)]);
        let three_coin_payment =
            mock_info(SENDER, &[coin(50, atom), coin(120, eth), coin(120, osmo)]);

        let (coin_one_amount, coin_two_amount) =
            must_pay_two_coins(&two_coin_payment, atom, eth).unwrap();
        assert_eq!(coin_one_amount, Uint128::new(50));
        assert_eq!(coin_two_amount, Uint128::new(120));

        let err = must_pay_two_coins(&duplicate_coins_payment, atom, osmo).unwrap_err();
        assert_eq!(err, PaymentError::MissingDenom(osmo.to_string()));

        let err = must_pay_two_coins(&no_payment, atom, osmo).unwrap_err();
        assert_eq!(err, PaymentError::NoFunds {});

        let err = must_pay_two_coins(&atom_payment, atom, osmo).unwrap_err();
        assert_eq!(err, PaymentError::MissingDenom(osmo.to_string()));

        let err = must_pay_two_coins(&osmo_payment, atom, osmo).unwrap_err();
        assert_eq!(err, PaymentError::MissingDenom(atom.to_string()));

        let err = must_pay_two_coins(&two_coin_payment, atom, osmo).unwrap_err();
        assert_eq!(err, PaymentError::MissingDenom(osmo.to_string()));

        let err = must_pay_two_coins(&three_coin_payment, osmo, atom).unwrap_err();
        assert_eq!(err, PaymentError::ExtraDenom(eth.to_string()));

        let err = must_pay_two_coins(&two_coin_payment, osmo, atom).unwrap_err();
        assert_eq!(err, PaymentError::MissingDenom(osmo.to_string()));
    }
}
