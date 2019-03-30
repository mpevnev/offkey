use alsa::pcm::{Access, Format, HwParams};
use alsa::{Direction, ValueOr, PCM};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MicSettings {
    pub format: Option<Format>,
    pub channels: Option<u32>,
    pub rate: Option<u32>,
    pub access: Option<Access>,
}

pub fn open_microphone(device: &str, settings: MicSettings) -> alsa::Result<PCM> {
    let mic = PCM::new(device, Direction::Capture, true)?;
    {
        let params = HwParams::any(&mic)?;
        if let Some(ch) = settings.channels {
            params.set_channels(ch)?;
        }
        if let Some(rate) = settings.rate {
            params.set_rate(rate, ValueOr::Nearest)?;
        }
        if let Some(fmt) = settings.format {
            params.set_format(fmt)?;
        }
        if let Some(acc) = settings.access {
            params.set_access(acc)?;
        }
        mic.hw_params(&params)?;
    }
    Ok(mic)
}
