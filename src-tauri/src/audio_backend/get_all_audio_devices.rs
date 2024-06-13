use cpal::traits::{DeviceTrait, HostTrait};

pub fn get_device_info() -> Result<Vec<String>, anyhow::Error> {

    let available_hosts = cpal::available_hosts();

    let mut all_devices: Vec<String> = Vec::new();

    for host_id in available_hosts {
        let host = cpal::host_from_id(host_id)?;

        let devices = host.devices()?;

        for (_, device) in devices.enumerate() {

            all_devices.push(device.name()?);
        }

    }

    if all_devices.is_empty() {
        return Err(anyhow::anyhow!("No audio output devices were found."))
    }

    Ok(all_devices)
}