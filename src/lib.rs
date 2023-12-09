#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    VisaApiError(#[from] visa_api::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
pub const MANUFACTURER: &str = "Keithley Instrument";
pub const MODEL: &str = "2230";

pub struct Keithley2230 {
    instrument: visa_api::Instrument,
}

#[derive(Clone, Copy, Debug)]
pub enum PowerSupplyState {
    ON,
    OFF,
}

#[derive(Clone, Copy, Debug)]
pub enum ChannelOutputState {
    ON,
    OFF,
}

#[derive(Clone, Copy, Debug)]
pub enum Channel {
    CH1,
    CH2,
    CH3,
}

#[derive(Clone, Copy, Debug)]
pub enum Parallel {
    ON,
    OFF,
}

#[derive(Clone, Copy, Debug)]
pub enum Series {
    ON,
    OFF,
}

impl core::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::CH1 => write!(f, "CH1"),
            Channel::CH2 => write!(f, "CH2"),
            Channel::CH3 => write!(f, "CH3"),
        }
    }
}

impl core::fmt::Display for PowerSupplyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerSupplyState::ON => write!(f, "ON"),
            PowerSupplyState::OFF => write!(f, "OFF"),
        }
    }
}

impl core::fmt::Display for ChannelOutputState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelOutputState::ON => write!(f, "ON"),
            ChannelOutputState::OFF => write!(f, "OFF"),
        }
    }
}

impl Keithley2230 {
    pub fn new(instrument: visa_api::Instrument) -> Self {
        Self { instrument }
    }

    pub fn set_channel(&mut self, channel: Channel, voltage: f32, current: f32) -> Result<()> {
        let command = format!("APPL {}, {}, {}", channel, voltage, current);
        visa_api::Instrument::write(&mut self.instrument, &command)?;
        Ok(())
    }

    pub fn enable_output(&mut self, state: PowerSupplyState) -> Result<()> {
        let command = format!("OUTP:ENAB {}", state);
        visa_api::Instrument::write(&mut self.instrument, &command)?;
        Ok(())
    }

    pub fn enable_channels(&mut self, state: ChannelOutputState) -> Result<()> {
        let command = format!("OUTP:ENAB {}", state);
        visa_api::Instrument::write(&mut self.instrument, &command)?;
        Ok(())
    }

    pub fn enable_channel(&mut self, channel: Channel, state: ChannelOutputState) -> Result<()> {
        // Save currently selected channel
        let previous_channel = self.get_channel()?;

        // Select channel to enable/disable
        self.select_channel(channel)?;
        // Enable/disable channel
        let command = format!("CHAN:OUTP {}", state);
        visa_api::Instrument::write(&mut self.instrument, &command)?;

        // Restore previous channel
        self.select_channel(previous_channel)?;
        Ok(())
    }

    pub fn get_channel(&mut self) -> Result<Channel> {
        visa_api::Instrument::write(&mut self.instrument, "INST?")?;
        let channel = visa_api::Instrument::read(&self.instrument)?;
        let channel = match channel.trim() {
            "CH1" => Channel::CH1,
            "CH2" => Channel::CH2,
            "CH3" => Channel::CH3,
            _ => unreachable!(),
        };
        Ok(channel)
    }

    pub fn select_channel(&mut self, channel: Channel) -> Result<()> {
        let command = format!("INST {}", channel);
        visa_api::Instrument::write(&mut self.instrument, &command)?;
        Ok(())
    }

    pub fn switch_to_front_panel_control(&mut self) -> Result<()> {
        visa_api::Instrument::write(&mut self.instrument, "SYST:LOC")?;
        Ok(())
    }

    pub fn switch_to_front_remote_control(&mut self) -> Result<()> {
        visa_api::Instrument::write(&mut self.instrument, "SYST:REM")?;
        Ok(())
    }

    pub fn read_current(&mut self) -> Result<(f32, f32, f32)> {
        visa_api::Instrument::write(&mut self.instrument, "FETC:CURR? ALL")?;
        let response = visa_api::Instrument::read(&self.instrument)?;
        let response = response
            .split(',')
            .map(|x| x.trim().parse::<f32>().ok().unwrap_or(0.0))
            .collect::<Vec<f32>>();

        if response.len() == 3 {
            Ok((response[0], response[1], response[2]))
        } else {
            Ok((0.0, 0.0, 0.0))
        }
    }

    pub fn read_voltage(&mut self) -> Result<(f32, f32, f32)> {
        visa_api::Instrument::write(&mut self.instrument, "FETC:VOLT? ALL")?;
        let response = visa_api::Instrument::read(&self.instrument)?;
        let response = response
            .split(',')
            .map(|x| x.trim().parse::<f32>().ok().unwrap_or(0.0))
            .collect::<Vec<f32>>();

        if response.len() == 3 {
            Ok((response[0], response[1], response[2]))
        } else {
            Ok((0.0, 0.0, 0.0))
        }
    }

    pub fn read_power(&mut self) -> Result<(f32, f32, f32)> {
        visa_api::Instrument::write(&mut self.instrument, "FETC:POW? ALL")?;
        let response = visa_api::Instrument::read(&self.instrument)?;
        let response = response
            .split(',')
            .map(|x| x.trim().parse::<f32>().ok().unwrap_or(0.0))
            .collect::<Vec<f32>>();

        if response.len() == 3 {
            Ok((response[0], response[1], response[2]))
        } else {
            Ok((0.0, 0.0, 0.0))
        }
    }

    pub fn set_parallel(&mut self, parallel: Parallel) -> Result<()> {
        let command = match parallel {
            Parallel::ON => "OUTP:PAR CH1CH2",
            Parallel::OFF => "OUTP:PAR OFF",
        };
        visa_api::Instrument::write(&mut self.instrument, command)?;
        Ok(())
    }

    pub fn set_series(&mut self, series: Series) -> Result<()> {
        let command = match series {
            Series::ON => "OUTP:SER ON",
            Series::OFF => "OUTP:SER OFF",
        };
        visa_api::Instrument::write(&mut self.instrument, command)?;
        Ok(())
    }
}
