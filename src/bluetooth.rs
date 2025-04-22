use std::cmp::PartialEq;
use std::process::Command;
#[cfg(unix)]
use bluez_async::{BluetoothError, BluetoothSession, DeviceId, MacAddress};
#[cfg(unix)]
use std::sync::{Arc, Mutex};
#[cfg(unix)]
use std::time::Duration;
#[cfg(unix)]
use tokio::runtime::Builder;
#[cfg(unix)]
use tokio::time::sleep;

impl BluetoothDevices {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    /// Searches for Bluetooth devices to pair
    #[cfg(unix)]
    pub fn refresh_bluetooth(&mut self) {
        let devices_clone = Arc::clone(&self.devices);
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let devices = locate_devices().await;
            if let Ok(devices_safe) = devices {
                let mut locked = devices_clone.lock().unwrap();
                *locked = devices_safe;
            }
        });
    }
    
    /// Gets the name of the device at a specific index
    #[cfg(unix)]
    pub fn get_device_name(&self, index: usize) -> &String {
        let device = &self.devices[index];
        if &device.name != "Unknown" {
            &device.name
        } else if &device.alias != "None" {
            &device.alias
        } else {
            &device.mac_address.to_string()
        }
    }
}

/// Fetches a vector of Bluetooth devices waiting to be paired
#[cfg(unix)]
async fn locate_devices() -> Result<Vec<BluetoothDevice>, BluetoothError> {
    let (_, session) = BluetoothSession::new().await?;

    session.start_discovery().await?;
    sleep(Duration::from_secs(5)).await;
    session.stop_discovery().await?;

    let devices = session.get_devices().await?;

    let mut bluetooth_devices: Vec<BluetoothDevice> = Vec::new();

    for device_info in devices {
        let device = BluetoothDevice {
            name: device_info.name.unwrap_or("Unknown".to_string()),
            paired: device_info.paired,
            connected: device_info.connected,
            id: device_info.id,
            alias: device_info.alias.unwrap_or("None".to_string()),
            mac_address: device_info.mac_address,
        };

        bluetooth_devices.push(device);
    }

    Ok(bluetooth_devices)
}

/// Async connection to a Bluetooth device
#[cfg(unix)]
async fn connect_device(device_id: &DeviceId) -> Result<(), BluetoothError> {
    let (_, session) = BluetoothSession::new().await?;
    session
        .connect_with_timeout(device_id, Duration::from_secs(10))
        .await
}

/// Bluetooth Device data
///
/// name: The name of the devices
/// paired: If the device is paired
/// connected: If the device is connected
/// id: The device Bluetooth id
/// alias: The device's nickname
/// mac_address: The MAC of the device
pub struct BluetoothDevice {
    pub(crate) name: String,
    pub(crate) paired: bool,
    pub connected: bool,
    pub bl_type: BluetoothDeviceType,
    #[cfg(unix)]
    pub id: DeviceId,
    pub alias: String,
    #[cfg(unix)]
    pub mac_address: MacAddress,
}

impl BluetoothDevice {
    pub fn connect(&self) {
        #[cfg(unix)]
        self.connect_to_device();
        if self.bl_type == BluetoothDeviceType::AUDIO {
            #[cfg(unix)]
            self.switch_audio_device();
        }
    }

    /// Switches the current audio device to the inputted BluetoothDevice
    #[cfg(unix)]
    pub fn switch_audio_device(&self) {
        let device_id = self.id.to_string();
        let command = format!("pw-cli set-default {}", device_id);

        Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .expect("failed to execute process");
    }

    /// Connects to a Bluetooth device
    #[cfg(unix)]
    pub fn connect_to_device(&self) {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            if connect_device(&self.id).await.is_err() {
                // TODO Error handling
            };
        });
    }
}

pub struct BluetoothDevices {
    pub devices: Vec<BluetoothDevice>,
}

#[derive(PartialEq)]
pub enum BluetoothDeviceType {
    INPUT,
    AUDIO,
}