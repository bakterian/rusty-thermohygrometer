use std::fmt::Write;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::{delay::FreeRtos, modem::Modem};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

pub struct Wifi;

impl Wifi {
	pub fn init(modem: Modem) -> EspWifi<'static> {
		let mut wifi_driver = EspWifi::new(
			modem,
			EspSystemEventLoop::take().expect("Failed to take system event loop"),
			Some(EspDefaultNvsPartition::take().expect("Failed to take default nvs partition")),
		)
		.expect("Failed to create esp wifi device");

		// Set WiFi transmit power (example: 8.5 dBm)
        //wifi_driver.set_tx_power(8.5).expect("Failed to set WiFi TX power");

		let mut cc = ClientConfiguration {
				auth_method: AuthMethod::WPA2Personal,
				..Default::default()
			};
			cc.ssid.write_str(env!("WIFI_SSID")).expect("Failed to set WiFi SSID");
			cc.password.write_str(env!("WIFI_PWD")).expect("Failed to set WiFi password");

		wifi_driver
			.set_configuration(&Configuration::Client(cc))
			.expect("Failed to set wifi driver configuration");

		wifi_driver.start().expect("Failed to start wifi driver");

		loop {
			match wifi_driver.is_started() {
				Ok(true) => {
					#[cfg(debug_assertions)]
					println!("Wifi driver started");
					break;
				}
				Ok(false) => {
					#[cfg(debug_assertions)]
					println!("Waiting for wifi driver to start")
				}
				Err(_e) => {
					#[cfg(debug_assertions)]
					println!("Error while starting wifi driver: {_e:?}")
				}
			}
		}

		loop {
			match wifi_driver.is_connected() {
				Ok(true) => {
					#[cfg(debug_assertions)]
					println!("Wifi is connected");
					break;
				}
				Ok(false) => {
					#[cfg(debug_assertions)]
					println!("Waiting for Wifi connection")
				}
				Err(_e) => {
					#[cfg(debug_assertions)]
					println!("Failed to connect wifi driver: {_e:?}")
				}
			}

			if let Err(_e) = wifi_driver.connect() {
				#[cfg(debug_assertions)]
				println!("Error while connecting wifi driver: {_e:?}")
			}

			FreeRtos::delay_ms(1000);
		}


		// Wait for DHCP lease and DNS assignment
        loop {
            let netif = wifi_driver.sta_netif();
            if let Ok(ip_info) = netif.get_ip_info() {
                if ip_info.ip != std::net::Ipv4Addr::UNSPECIFIED {
                    #[cfg(debug_assertions)]
                    println!("Got IP: {}", ip_info.ip);

					// Print DNS info if available
					let dns_info = netif.get_dns();
					#[cfg(debug_assertions)]
					println!("DNS: {:?}", dns_info);
                    break;
                }
            }
            FreeRtos::delay_ms(500);
        }

		wifi_driver
	}
}
