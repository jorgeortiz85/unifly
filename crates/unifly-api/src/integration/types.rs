//! Integration API response types for the UniFi Network Integration API (v10.1.84).
//!
//! All types match the JSON responses from `/integration/v1/` endpoints.
//! Field names use camelCase via `#[serde(rename_all = "camelCase")]`.

mod common;
mod inventory;
mod network;
mod policy;
mod reference;

pub use common::{ApplicationInfoResponse, ErrorResponse, Page, SiteResponse};
pub use inventory::{
    ClientActionRequest, ClientActionResponse, ClientDetailsResponse, ClientResponse,
    DeviceActionRequest, DeviceAdoptionRequest, DeviceDetailsResponse, DeviceResponse,
    DeviceStatisticsResponse, DeviceTagResponse, PendingDeviceResponse, PortActionRequest,
};
pub use network::{
    NetworkCreateUpdate, NetworkDetailsResponse, NetworkReferencesResponse, NetworkResponse,
    WifiBroadcastCreateUpdate, WifiBroadcastDetailsResponse, WifiBroadcastResponse,
};
pub use policy::{
    AclRuleCreateUpdate, AclRuleOrdering, AclRuleResponse, ApplicationCategoryFilter,
    ApplicationFilter, DestTrafficFilter, DnsPolicyCreateUpdate, DnsPolicyResponse, DomainFilter,
    FirewallPolicyCreateUpdate, FirewallPolicyDestination, FirewallPolicyOrdering,
    FirewallPolicyOrderingEnvelope,
    FirewallPolicyPatch, FirewallPolicyResponse, FirewallPolicySource, FirewallZoneCreateUpdate,
    FirewallZoneResponse, IpAddressFilter, IpAddressItem, MacAddressFilter, NetworkFilter,
    PortFilter, PortItem, RegionFilter, SourceTrafficFilter, TrafficMatchingListCreateUpdate,
    TrafficMatchingListResponse, VoucherCreateRequest, VoucherDeletionResults, VoucherResponse,
};
pub use reference::{
    CountryResponse, DpiApplicationResponse, DpiCategoryResponse, RadiusProfileResponse,
    VpnServerResponse, VpnTunnelResponse, WanResponse,
};
