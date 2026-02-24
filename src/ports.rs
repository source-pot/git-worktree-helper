use rand::Rng;
use std::collections::HashMap;
use std::net::TcpListener;

const PORT_MIN: u16 = 10000;
const PORT_MAX: u16 = 11000;
const MAX_ATTEMPTS: u32 = 50;

/// Allocate a random available port, avoiding any already-allocated ports.
fn allocate_port(already_allocated: &[u16]) -> anyhow::Result<u16> {
    let mut rng = rand::thread_rng();
    for _ in 0..MAX_ATTEMPTS {
        let port = rng.gen_range(PORT_MIN..=PORT_MAX);
        if already_allocated.contains(&port) {
            continue;
        }
        if is_port_available(port) {
            return Ok(port);
        }
    }
    anyhow::bail!("Failed to find an available port after {MAX_ATTEMPTS} attempts")
}

fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// For each entry in the config ports map, allocate a random available port.
/// Returns a map of env_var_name → port_string.
pub fn allocate_ports(config: &HashMap<String, String>) -> anyhow::Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    let mut allocated: Vec<u16> = Vec::new();

    for env_var in config.values() {
        let port = allocate_port(&allocated)?;
        allocated.push(port);
        result.insert(env_var.clone(), port.to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_port_returns_port_in_valid_range() {
        let port = allocate_port(&[]).unwrap();
        assert!((PORT_MIN..=PORT_MAX).contains(&port));
    }

    #[test]
    fn allocate_ports_empty_config_returns_empty_map() {
        let config = HashMap::new();
        let result = allocate_ports(&config).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn allocate_ports_multiple_entries_returns_distinct_ports() {
        let mut config = HashMap::new();
        config.insert("frontend".to_string(), "FRONTEND_PORT".to_string());
        config.insert("backend".to_string(), "BACKEND_PORT".to_string());
        config.insert("db".to_string(), "DB_PORT".to_string());

        let result = allocate_ports(&config).unwrap();
        assert_eq!(result.len(), 3);

        let ports: Vec<u16> = result.values().map(|v| v.parse().unwrap()).collect();
        // All distinct
        let mut unique = ports.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), ports.len());

        // All in range
        for port in &ports {
            assert!((PORT_MIN..=PORT_MAX).contains(port));
        }
    }

    #[test]
    fn is_port_available_returns_false_for_bound_port() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!is_port_available(port));
    }
}
