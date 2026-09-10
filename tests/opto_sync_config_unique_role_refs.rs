use opto_sync_lib_core::parse_opto_sync_config;

#[test]
fn one_binding_inventory_can_be_shared_across_hybrid_roles() {
    parse_opto_sync_config(r#"
version = 1
mode = "hybrid"
strict = true

[flags2env]
contract = ".cli-flags.toml"
require_audit = true
precedence = "argv-over-env"

[sync]
push_interval_ms = 1000
pull_interval_ms = 1000
max_batch_size = 8
conflict_policy = "last-write-wins"

[[env]]
name = "api_base_url"
key = "OPTO_SYNC_API_HTTP_BASE"
kind = "url"
required = false
secret = false
default = "http://127.0.0.1:8080/"

[[env]]
name = "bind_addr"
key = "OPTO_SYNC_API_BIND"
kind = "string"
required = false
secret = false
default = "127.0.0.1:8080"

[client]
enabled = true
api_base_url_binding = "api_base_url"

[server]
enabled = true
bind_addr_binding = "bind_addr"
"#).expect("hybrid config should be valid");
}
