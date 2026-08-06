use candle_core::Device;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Auto,
    Cpu,
    Cuda,
}

pub fn device_kind_from_env() -> DeviceKind {
    match std::env::var("MOON_LOL_DEVICE")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "cuda" => DeviceKind::Cuda,
        "cpu" => DeviceKind::Cpu,
        _ => DeviceKind::Auto,
    }
}

pub fn select_device() -> anyhow::Result<Device> {
    match device_kind_from_env() {
        DeviceKind::Cpu => Ok(Device::Cpu),
        DeviceKind::Auto => Device::cuda_if_available(0).map_err(|e| anyhow::anyhow!("{e}")),
        DeviceKind::Cuda => {
            let d = Device::cuda_if_available(0)?;
            if d.is_cpu() {
                anyhow::bail!("MOON_LOL_DEVICE=cuda 但未编译 cuda feature 或未检测到 CUDA 设备");
            }
            Ok(d)
        }
    }
}
