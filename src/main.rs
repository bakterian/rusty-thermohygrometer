mod wifi;

use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::mqtt::client::{EspMqttClient, EventPayload, MqttClientConfiguration};
use esp_idf_svc::ipv4::Ipv4Addr;
use crate::wifi::Wifi;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::mqtt::client::QoS;
use std::{thread::sleep, time::Duration};
use anyhow;
use std::net::ToSocketAddrs;

use embedded_hal::spi::*;
use esp_idf_hal::gpio;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::config::Config;
use esp_idf_hal::spi::*;


fn blink_max7219<'a>(spi: &mut SpiDeviceDriver<'a, SpiDriver<'a>>)
{
// Application

    // 1) Initalize Matrix Display

    // 1.a) Power Up Device

    // - Prepare Data to be Sent
    // 8-bit Data/Command Corresponding to Matrix Power Up
    let data: u8 = 0x01;
    // 4-bit Address of Shutdown Mode Command
    let addr: u8 = 0x0C;
    // Package into array to pass to SPI write method
    // Write method will grab array and send all data in it
    let send_array: [u8; 2] = [addr, data];

    // - Send Data
    // Shift in 16 bits by passing send_array (bits will be shifted MSB first)
    // Note that write method handles the CS pin state
    spi.write(&send_array).unwrap();

    // 1.b) Set up Decode Mode

    // - Prepare Information to be Sent
    // 8-bit Data/Command Corresponding to No Decode Mode
    let data: u8 = 0x00;
    // 4-bit Address of Decode Mode Command
    let addr: u8 = 0x09;
    // Package into array to pass to SPI write method
    // Write method will grab array and send all data in it
    let send_array: [u8; 2] = [addr, data];

    // - Send Data
    // Shift in 16 bits by passing send_array (bits will be shifted MSB first)
    spi.write(&send_array).unwrap();

    // 1.c) Configure Scan Limit

    // - Prepare Information to be Sent
    // 8-bit Data/Command Corresponding to Scan Limit Displaying all digits
    let data: u8 = 0x07;
    // 4-bit Address of Scan Limit Command
    let addr: u8 = 0x0B;
    // Package into array to pass to SPI write method
    // Write method will grab array and send all data in it
    let send_array: [u8; 2] = [addr, data];

    // - Send Data
    // Shift in 16 bits by passing send_array (bits will be shifted MSB first)
    spi.write(&send_array).unwrap();

    // 1.c) Configure Intensity

    // - Prepare Information to be Sent
    // 8-bit Data/Command Corresponding to (15/32 Duty Cycle) Medium Intensity
    let data: u8 = 0x07;
    // 4-bit Address of Intensity Control Command
    let addr: u8 = 0x0A;
    // Package into array to pass to SPI write method
    // Write method will grab array and send all data in it
    let send_array: [u8; 2] = [addr, data];

    // - Send Data
    // Shift in 16 bits by passing send_array (bits will be shifted MSB first)
    spi.write(&send_array).unwrap();

	let mut data: u8 = 1;


	// 2) Light up LED Matrix row by row with 500ms delay in between
	// Iterate over all rows of LED matrix
	for addr in 1..9 {
		// addr refrences the row data will be sent to
		let send_array: [u8; 2] = [addr, data];
		// Shift a 1 with evey loop
		data = data << 1;

		// Send data just like earlier
		spi.write(&send_array).unwrap();

		// Delay for 500ms to show effect
		FreeRtos::delay_ms(500_u32);
	}

	FreeRtos::delay_ms(1000_u32);

	let send_array: [u8; 2] = [4, 255];
	spi.write(&send_array).unwrap();

	FreeRtos::delay_ms(3000_u32);

	// Clear the LED matrix row by row with 500ms delay in between
	for addr in 1..9 {
		let send_array: [u8; 2] = [addr, data];
		spi.write(&send_array).unwrap();
		FreeRtos::delay_ms(500_u32);
	}
}

fn remote_servers_reachable(wifi: EspWifi<'static>) -> bool 
{
	let gw_addr = wifi
		.sta_netif()
		.get_ip_info()
		.expect("Failed to get ip info")
		.subnet
		.gateway;

	// Helper function to resolve a hostname to an IPv4 address
	fn resolve_hostname_to_ipv4(hostname: &str) -> anyhow::Result<Ipv4Addr> {
		let addr = (hostname, 80)
			.to_socket_addrs()?
			.find_map(|sockaddr| {
				if let std::net::SocketAddr::V4(ipv4) = sockaddr {
					Some(Ipv4Addr::from(ipv4.ip().octets()))
				} else {
					None
				}
			})
			.ok_or_else(|| anyhow::anyhow!("No IPv4 address found for hostname"))?;
		Ok(addr)
	}

	let google_addr = resolve_hostname_to_ipv4("google.com").expect("Failed to resolve google.com");

	//let hive_borker_addr = resolve_hostname_to_ipv4("broker.mqttdashboard.com").expect("Failed to parse broker.mqttdashboard.com address");

	let ip_addr_array = [gw_addr, google_addr];

	let mut ping_succesfull = true;

	for ip in ip_addr_array {

		println!("Pinging ip address: {}", ip);

		// let ping = EspPing::default()
		// 	.ping(ip, &Default::default());

		let ping = EspPing::default()
			.ping(ip, &Default::default());

		match &ping {
			Ok(summary) => 
			{
				if(summary.received > 0) {
					println!("Ping success summary: {:?}", summary);
				} else {
					println!("Ping to {} failed", ip);
					ping_succesfull = false;
					break;
				}
			}
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

    // Create handles for SPI pins
    let sclk = peripherals.pins.gpio8;
    let mosi = peripherals.pins.gpio10;
    let cs = peripherals.pins.gpio5;

    // Instantiate SPI Driver
    let spi_drv = SpiDriver::new(
        peripherals.spi2,
        sclk,
        mosi,
        None::<gpio::AnyIOPin>,
        &SpiDriverConfig::new(),
    )
    .unwrap();

    // Configure Parameters for SPI device
    let config = Config::new().baudrate(2.MHz().into()).data_mode(Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    });

	// Instantiate SPI Device Driver and Pass Configuration
	let mut spi: SpiDeviceDriver<'_, SpiDriver<'_>> = SpiDeviceDriver::new(spi_drv, Some(cs), &config).unwrap();

	blink_max7219(&mut spi);


	let wifi = Wifi::init(peripherals.modem); // Connectivity goes away when dropped

	FreeRtos::delay_ms(5000); // Wait for the DHCP server to deliver a lease

	if !remote_servers_reachable(wifi) {
		println!("Remote servers not reachable, application stops here");
		Err(anyhow::anyhow!("Remote servers not reachable"))
	}
	else {
		println!("Remote servers reachable, setting up MQTT client");

		let mqtt_config = MqttClientConfiguration {
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
