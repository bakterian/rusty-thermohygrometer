mod wifi;

use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::ping::EspPing;
use esp_idf_sys as _;

use esp_idf_svc::ipv4::Ipv4Addr;

use crate::wifi::Wifi;

fn main() {
	esp_idf_sys::link_patches();

	let peripherals = Peripherals::take().expect("Failed to take peripherals");

	let wifi = Wifi::init(peripherals.modem); // Connectivity goes away when dropped

	FreeRtos::delay_ms(5000); // Wait for the DHCP server to deliver a lease

	let gw_addr = wifi
		.sta_netif()
		.get_ip_info()
		.expect("Failed to get ip info")
		.subnet
		.gateway;

	let google_addr = Ipv4Addr::new(8, 8, 8, 8);

	let ip_addr_array = [gw_addr, google_addr];

	for ip in ip_addr_array {

		println!("Pinging ip address: {}", ip);

		let ping = EspPing::default()
			.ping(ip, &Default::default())
			.expect("Failed to ping");

		println!("Ping summary: {:?}", ping);
	}

}
