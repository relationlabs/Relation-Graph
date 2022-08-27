//! Specific library to support IC
use getrandom::register_custom_getrandom;

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // TODO: get some randomness, just use timestamp as random for now
    let bytes = timestamp().to_le_bytes();
    buf.copy_from_slice(&bytes);
    return Ok(());
}
register_custom_getrandom!(custom_getrandom);

/// Returns the current time in milliseconds
pub(crate) fn timestamp() -> u64 {
    // NOTE: confusing time api output
    // ic_cdk::api::time(), e.g. output: 1638348267_014414_000_u64
    ic_cdk::api::time() / 1_000_000_u64
}