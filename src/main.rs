mod wifi;

use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::mqtt::client::{EspMqttClient, EventPayload, MqttClientConfiguration, MqttProtocolVersion};
use esp_idf_svc::ipv4::Ipv4Addr;
use crate::wifi::Wifi;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::mqtt::client::QoS;
use std::{thread::sleep, time::Duration};
use anyhow;

fn remote_servers_reachable(wifi: EspWifi<'static>) -> bool 
{
	let gw_addr = wifi
		.sta_netif()
		.get_ip_info()
		.expect("Failed to get ip info")
		.subnet
		.gateway;

	let google_addr = Ipv4Addr::new(8, 8, 8, 8);

	let ip_addr_array = [gw_addr, google_addr];

	let mut ping_succesfull = true;

	for ip in ip_addr_array {

		println!("Pinging ip address: {}", ip);

		let ping = EspPing::default()
			.ping(ip, &Default::default());

		match &ping {
			Ok(summary) => println!("Ping success summary: {:?}", summary),
			Err(e) => {
				println!("Ping error: {:?}", e);
				ping_succesfull = false;
				break;
			}
		}
	}

	ping_succesfull
}


struct AppConfiguration<'a> {
	wifi_ssid: &'a str,
	wifi_pwd: &'a str,
	mqtt_broker_username: &'a str,
	mqtt_broker_pwd: &'a str,
	mqtt_broker_url: &'a str,
	mqtt_remote_ctrl_commands_topic: &'a str,
	mqtt_temperature_data_topic: &'a str,
	mqtt_humidity_data_topic: &'a str,
}	

fn main() -> anyhow::Result<()> 
{
	let app_config = AppConfiguration {
		wifi_ssid: env!("WIFI_SSID"),
		wifi_pwd: env!("WIFI_PWD"),
		mqtt_broker_username: env!("MQTT_USERNAME"),
		mqtt_broker_pwd: env!("MQTT_PWD"),
		mqtt_broker_url: env!("MQTT_BROKER_URL"),
		mqtt_remote_ctrl_commands_topic: env!("MQTT_REMOTE_CTRL_COMMANDS_TOPIC"),
		mqtt_temperature_data_topic:  env!("MQTT_TEMP_DATA_TOPIC"),
		mqtt_humidity_data_topic: env!("MQTT_HUMIDITY_DATA_TOPIC"),
	};

	//TODO: verify that all configuration parameters are filled and in valid range
	//TODO: add a Trait that provides only WiFi configuration parameters, and implement it for AppConfiguration struct,
	// 	    so that Wifi::init() can take a generic parameter and not read the environment variables directly

	esp_idf_sys::link_patches();

	let peripherals = Peripherals::take().expect("Failed to take peripherals");

	let wifi = Wifi::init(peripherals.modem); // Connectivity goes away when dropped

	FreeRtos::delay_ms(5000); // Wait for the DHCP server to deliver a lease

	if !remote_servers_reachable(wifi) {
		println!("Remote servers not reachable, application stops here");
		Err(anyhow::anyhow!("Remote servers not reachable"))
	}
	else {
		println!("Remote servers reachable, setting up MQTT client");

		let mqtt_config = MqttClientConfiguration {
			protocol_version: Some(MqttProtocolVersion::V3_1_1),
			username: Some(app_config.mqtt_broker_username),
			password: Some(app_config.mqtt_broker_pwd),
			..Default::default()
		};

		let mut client = EspMqttClient::new_cb(
			app_config.mqtt_broker_url,
			&mqtt_config,
			|message_event| {
				match message_event.payload() {
					EventPayload::BeforeConnect => println!("Before Connect event received"),
					EventPayload::Connected(session_present) => 
					{
						println!("Connected to broker");
						println!("Session : {}", if session_present { "present" } else { "not present" });
					},
					EventPayload::Disconnected => println!("Disconnected"),
					EventPayload::Subscribed(id) => println!("Subscribed to {} id", id),
					EventPayload::Unsubscribed(id) => println!("Unsubscribed from {} id", id),
					EventPayload::Published(id) => println!("Published {} od", id),
					EventPayload::Deleted(id) => println!("Deleted {} id", id),
					EventPayload::Error(e) => println!("Mqtt client error: {:?}", e),
					EventPayload::Received { id, topic, data, details } =>
					{
						println!("Received message. id: {}, topic: {:?}, data: {:?}, details: {:?}", id, topic, core::str::from_utf8(data), details);
					}
				};
			},
		)?;

		// Subscribe to MQTT Topic for remote control commands
		client.subscribe(app_config.mqtt_remote_ctrl_commands_topic, QoS::AtLeastOnce)?;

		let mut temp_val = 0.5;

		loop {
			sleep(Duration::from_secs(1));
			client.publish(app_config.mqtt_temperature_data_topic, QoS::ExactlyOnce, true, format!("Temp: {}", temp_val).as_bytes())?;
			temp_val = temp_val + 0.1;
		}
	}


}
