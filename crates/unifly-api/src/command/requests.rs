// ── Typed request structs for Command payloads ──
//
// Every Command variant that previously took `serde_json::Value`
// now uses one of these strongly-typed request structs instead.

mod dns;
mod network;
mod policy;
mod traffic;
mod vouchers;

pub use dns::{CreateDnsPolicyRequest, UpdateDnsPolicyRequest};
pub use network::{
    CreateNetworkRequest, CreateWifiBroadcastRequest, UpdateNetworkRequest,
    UpdateWifiBroadcastRequest,
};
pub use policy::{
    CreateAclRuleRequest, CreateFirewallPolicyRequest, CreateFirewallZoneRequest,
    TrafficFilterSpec, UpdateAclRuleRequest, UpdateFirewallPolicyRequest,
    UpdateFirewallZoneRequest,
};
pub use traffic::{CreateTrafficMatchingListRequest, UpdateTrafficMatchingListRequest};
pub use vouchers::CreateVouchersRequest;
