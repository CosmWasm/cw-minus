use cosmwasm_std::Response;

/// This defines a set of attributes which should be added to `Response`.
#[deprecated(
    note = "This is probably not needed anymore. If you use it, please share in https://github.com/CosmWasm/cw-utils/issues/17."
)]
pub trait Event {
    /// Append attributes to response
    fn add_attributes(&self, response: &mut Response);
}
