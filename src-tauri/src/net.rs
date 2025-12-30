use std::sync::OnceLock;

/// 缓存IPv6连接性检查结果，避免重复执行
static IPV6_CONNECTIVITY_CACHE: OnceLock<bool> = OnceLock::new();

/// 检查设备是否支持IPv6连接
/// 优化：1. 通过检查本地网络接口而非外部连接来判断
///       2. 结果缓存，避免重复执行
pub fn has_ipv6_connectivity() -> bool {
    // 如果缓存中有结果，直接返回
    *IPV6_CONNECTIVITY_CACHE.get_or_init(|| {
        // 获取所有网络接口
        if let Ok(interfaces) = getifaddrs::getifaddrs() {
            // 遍历所有接口和地址
            for iface in interfaces {
                // 检查是否为IPv6地址
                match iface.address {
                    getifaddrs::Address::V6(sock_addr_v6) => {
                        // 检查是否为全局IPv6地址
                        let ipv6_addr = sock_addr_v6.address;
                        if !ipv6_addr.is_loopback() && !ipv6_addr.is_unicast_link_local() {
                            // 如果有可用的全局IPv6地址，返回true
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        // 默认返回false
        false
    })
}
