use std::str::FromStr;
use visa_api::*;
pub struct Keithley2230 {
    pub inner: visa_api::Instrument,
}

pub const MANUFACTURER: &str = "Keithley Instruments";
pub const MODEL: &str = "2230";

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    VisaApiError(#[from] visa_api::Error),
    #[error(transparent)]
    StrumParseError(#[from] strum::ParseError),
    #[error("No Instrument found")]
    NoInstrumentFound(),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, strum::AsRefStr, strum::Display, Default, strum::EnumString)]
pub enum Channel {
    #[default]
    #[strum(serialize = "CH1")]
    CH1,
    #[strum(serialize = "CH2")]
    CH2,
    #[strum(serialize = "CH3")]
    CH3,
}

#[derive(Debug, PartialEq, strum::AsRefStr, strum::Display, Default)]
pub enum State {
    #[default]
    #[strum(serialize = "ON", serialize = "1")]
    ON,
    #[strum(serialize = "OFF", serialize = "0")]
    OFF,
}
#[derive(Debug, Default)]
pub struct Meas {
    pub ch1: ChMeas,
    pub ch2: ChMeas,
    pub ch3: ChMeas,
}

#[derive(Debug, Default)]
pub struct ChMeas {
    pub v: f32,
    pub i: f32,
    pub p: f32,
}

impl ChMeas {
    pub fn new(v: f32, i: f32, p: f32) -> Self {
        Self { v, i, p }
    }
}

impl Keithley2230 {
    pub fn new(rm: &DefaultRM) -> Result<Self> {
        let session = Instrument::new_session(&rm, MANUFACTURER, MODEL)?;
        if let Some(session) = session {
            Ok(Self { inner: session })
        } else {
            Err(Error::NoInstrumentFound())
        }
    }

    pub fn set_channel(&mut self, ch: Channel, v: f32, i: f32) -> Result<()> {
        let cmd = format!("APPL {}, {}, {}", ch, v, i);
        self.inner.write(&cmd)?;
        Ok(())
    }

    pub fn enable_output(&mut self, state: State) -> Result<()> {
        let cmd = format!("OUTP:ENAB {}", state);
        self.inner.write(&cmd)?;
        Ok(())
    }

    pub fn enable_channel(&mut self, ch: Channel, state: State) -> Result<()> {
        let prev_ch = self.get_channel()?;
        self.select_channel(ch)?;
        let cmd = format!("CHAN:OUTP {}", state);
        self.inner.write(&cmd)?;
        self.select_channel(prev_ch)?;
        Ok(())
    }

    pub fn get_channel(&mut self) -> Result<Channel> {
        self.inner.write("INST?")?;
        let ch = self.inner.read()?;
        let ch = Channel::from_str(&ch)?;
        Ok(ch)
    }

    pub fn select_channel(&mut self, ch: Channel) -> Result<()> {
        let cmd = format!("INST {}", ch);
        self.inner.write(&cmd)?;
        Ok(())
    }

    pub fn front_panel_ctrl(&mut self) -> Result<()> {
        self.inner.write("SYST:LOC")?;
        Ok(())
    }

    pub fn remote_ctrl(&mut self) -> Result<()> {
        self.inner.write("SYST:REM")?;
        Ok(())
    }

    pub fn read_i(&mut self) -> Result<(f32, f32, f32)> {
        self.inner.write("FETC:CURR? ALL")?;
        let response = self.inner.read()?;

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

    pub fn read_v(&mut self) -> Result<(f32, f32, f32)> {
        self.inner.write("FETC:VOLT? ALL")?;
        let response = self.inner.read()?;

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

    pub fn read_p(&mut self) -> Result<(f32, f32, f32)> {
        self.inner.write("FETC:POW? ALL")?;
        let response = self.inner.read()?;

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

    pub fn read_all(&mut self) -> Result<Meas> {
        let v = self.read_v()?;
        let i = self.read_i()?;
        let p = self.read_p()?;

        let meas = Meas {
            ch1: ChMeas::new(v.0, i.0, p.0),
            ch2: ChMeas::new(v.1, i.1, p.1),
            ch3: ChMeas::new(v.2, i.2, p.2),
        };

        Ok(meas)
    }

    pub fn set_paralel(&mut self, state: State) -> Result<()> {
        let cmd = format!("OUT:PAR {}", state);
        self.inner.write(&cmd)?;
        Ok(())
    }

    pub fn set_series(&mut self, state: State) -> Result<()> {
        let cmd = format!("OUT:SER {}", state);
        self.inner.write(&cmd)?;
        Ok(())
    }
}
