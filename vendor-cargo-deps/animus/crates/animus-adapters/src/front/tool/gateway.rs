include!("gateway/delivery/types.rs");
include!("gateway/delivery/request.rs");
include!("gateway/delivery/policy.rs");
include!("gateway/delivery/outbox.rs");
include!("gateway/delivery/retry/receipts.rs");
include!("gateway/delivery/errors.rs");

#[cfg(test)]
mod tests {
    include!("gateway/tests/tests001.rs");
    include!("gateway/tests/tests002.rs");
}
