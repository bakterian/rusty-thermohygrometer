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

		wifi_driver
	}
}
