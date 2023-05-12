mod business_error;
mod store_token_error;
mod subscribe_error;

pub use business_error::BizErrorEnum;
pub use store_token_error::StoreTokenError;
pub use subscribe_error::SubscribeError;

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by: \n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
