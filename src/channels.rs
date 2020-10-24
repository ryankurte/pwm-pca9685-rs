use crate::{hal, Channel, Error, Pca9685, Register};



impl<I2C, E> hal::pwm::Pwm<> for Pca9685<I2C>
where
    I2C: hal::blocking::i2c::Write<Error = E> + hal::blocking::i2c::WriteRead<Error = E>,
{
    type Duty = u16;
    type Channel = Channel;
    type Time = ();
    type Error = Error<E>;

    fn try_disable(&mut self, ch: &Channel) -> Result<(), Self::Error> { 
        self.set_channel_full_off(ch)
    }

    fn try_enable(&mut self, ch: &Channel) -> Result<(), Self::Error> { 
        unimplemented!()
    }

    fn try_get_duty(&self, _ch: &Channel) -> Result<u16, Self::Error> { 
        unimplemented!()
    }

    fn try_get_max_duty(&self) -> Result<u16, Self::Error> { 
        Ok(0)
    }

    fn try_set_duty(&mut self, _ch: &Channel, _p: u16) -> Result<(), Self::Error> { 
        Ok(())
    }

    fn try_get_period(&self) -> Result<Self::Time, Self::Error> { 
        unimplemented!()
    
    }

    fn try_set_period<T>(&mut self, _: T) -> Result<(), Self::Error> 
        where T: Into<Self::Time>
    {
        unimplemented!()
    }

}

impl<I2C, E> Pca9685<I2C>
where
    I2C: hal::blocking::i2c::Write<Error = E> + hal::blocking::i2c::WriteRead<Error = E>,
{
    /// Set the `ON` counter for the selected channel.
    ///
    /// Note that the full off setting takes precedence over the `on` settings.
    /// See section 7.3.3 "LED output and PWM control" of the datasheet for
    /// further details.
    pub fn set_channel_on(&mut self, channel: Channel, value: u16) -> Result<(), Error<E>> {
        if value > 4095 {
            return Err(Error::InvalidInputData);
        }
        let reg = get_register_on(channel);
        self.write_double_register(reg, value)
    }

    /// Set the `OFF` counter for the selected channel.
    pub fn set_channel_off(&mut self, channel: Channel, value: u16) -> Result<(), Error<E>> {
        if value > 4095 {
            return Err(Error::InvalidInputData);
        }
        let reg = get_register_off(channel);
        self.write_double_register(reg, value)
    }

    /// Set the `ON` and `OFF` counters for the selected channel.
    ///
    /// Note that the full off setting takes precedence over the `on` settings.
    /// See section 7.3.3 "LED output and PWM control" of the datasheet for
    /// further details.
    pub fn set_channel_on_off(
        &mut self,
        channel: Channel,
        on: u16,
        off: u16,
    ) -> Result<(), Error<E>> {
        if on > 4095 || off > 4095 {
            return Err(Error::InvalidInputData);
        }
        let reg = get_register_on(channel);
        self.write_two_double_registers(reg, on, off)
    }

    /// Set the channel always on.
    ///
    /// The turning on is delayed by the value argument.
    /// Note that the full off setting takes precedence over the `on` settings.
    ///
    /// See section 7.3.3 "LED output and PWM control" of the datasheet for
    /// further details.
    pub fn set_channel_full_on(&mut self, channel: Channel, value: u16) -> Result<(), Error<E>> {
        if value > 4095 {
            return Err(Error::InvalidInputData);
        }
        let reg = get_register_on(channel);
        let value = value | 0b0001_0000_0000_0000;
        self.write_double_register(reg, value)
    }

    /// Set the channel always off.
    ///
    /// This takes precedence over the `on` settings and can be cleared by setting
    /// the `off` counter with [`set_channel_off`](struct.Pca9685.html#method.set_channel_off).
    ///
    /// See section 7.3.3 "LED output and PWM control" of the datasheet for
    /// further details.
    pub fn set_channel_full_off(&mut self, channel: Channel) -> Result<(), Error<E>> {
        let reg = get_register_off(channel);
        let value = 0b0001_0000_0000_0000;
        self.write_double_register(reg, value)
    }

    /// Set the `ON` and `OFF` counter for each channel at once.
    ///
    /// The index of the value in the arrays corresponds to the channel: 0-15.
    /// Note that the full off setting takes precedence over the `on` settings.
    /// See section 7.3.3 "LED output and PWM control" of the datasheet for
    /// further details.
    pub fn set_all_on_off(&mut self, on: &[u16; 16], off: &[u16; 16]) -> Result<(), Error<E>> {
        let mut data = [0; 65];
        data[0] = Register::C0_ON_L;
        for (i, (on, off)) in on.iter().zip(off).enumerate() {
            if *on > 4095 || *off > 4095 {
                return Err(Error::InvalidInputData);
            }
            data[i * 4 + 1] = *on as u8;
            data[i * 4 + 2] = (*on >> 8) as u8;
            data[i * 4 + 3] = *off as u8;
            data[i * 4 + 4] = (*off >> 8) as u8;
        }
        self.enable_auto_increment()?;
        self.i2c.try_write(self.address, &data).map_err(Error::I2C)
    }
}

macro_rules! get_register {
    ($channel:expr, $($C:ident, $reg:ident),*) => {
        match $channel {
            $(
                Channel::$C  => Register::$reg,
            )*
        }
    };
}

fn get_register_on(channel: Channel) -> u8 {
    get_register!(
        channel, C0, C0_ON_L, C1, C1_ON_L, C2, C2_ON_L, C3, C3_ON_L, C4, C4_ON_L, C5, C5_ON_L, C6,
        C6_ON_L, C7, C7_ON_L, C8, C8_ON_L, C9, C9_ON_L, C10, C10_ON_L, C11, C11_ON_L, C12,
        C12_ON_L, C13, C13_ON_L, C14, C14_ON_L, C15, C15_ON_L, All, ALL_C_ON_L
    )
}

fn get_register_off(channel: Channel) -> u8 {
    get_register!(
        channel,
        C0,
        C0_OFF_L,
        C1,
        C1_OFF_L,
        C2,
        C2_OFF_L,
        C3,
        C3_OFF_L,
        C4,
        C4_OFF_L,
        C5,
        C5_OFF_L,
        C6,
        C6_OFF_L,
        C7,
        C7_OFF_L,
        C8,
        C8_OFF_L,
        C9,
        C9_OFF_L,
        C10,
        C10_OFF_L,
        C11,
        C11_OFF_L,
        C12,
        C12_OFF_L,
        C13,
        C13_OFF_L,
        C14,
        C14_OFF_L,
        C15,
        C15_OFF_L,
        All,
        ALL_C_OFF_L
    )
}
