use fixed::{
    traits::{FromFixed, ToFixed},
    types::I16F16,
};

pub struct Random {
    high: u32,
    low: u32,
}

impl Random {
    pub fn new(seed: u32) -> Self {
        if seed == 0 {
            // TODO: not covered by tests
            return Random {
                high: 0xd67c_e1e8,
                low: 0x42cf_adf8,
            };
        }

        let mut random = Random {
            high: seed ^ 0xbead_29ba,
            low: seed,
        };
        for _ in 0..32 {
            random.cycle_self();
        }
        random
    }

    pub fn next(&mut self) -> I16F16 {
        self.next_max(I16F16::ONE)
    }

    pub fn next_max(&mut self, max: I16F16) -> I16F16 {
        if max == I16F16::ZERO {
            return I16F16::ZERO;
        }

        self.cycle_self();
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        I16F16::from_bits((self.high % max.to_bits() as u32) as i32)
    }

    pub fn next_uint_max<T>(&mut self, max: T) -> T
    where
        T: FromFixed + ToFixed,
    {
        self.next_max(I16F16::from_num(max)).floor().to_num()
    }

    fn cycle_self(&mut self) {
        let high_high = self.high >> 16;
        let high_low = self.high % (1 << 16);
        let swapped_high = (high_low << 16) | high_high;

        let new_high = u32::wrapping_add(swapped_high, self.low);
        let new_low = u32::wrapping_add(new_high, self.low);

        (self.high, self.low) = (new_high, new_low);
    }
}
