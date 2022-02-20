// Various newtypes so that I can (re)implement traits from upstream crates optimized for size.
// Perhaps eventually they will become full-fledged crates.

pub mod fmt {
    use core::convert::Infallible;
    use fixed::types::U0F16;
    use ufmt::{uDisplay, Formatter};
    use ufmt_write::uWrite;

    pub struct U0F16SmallFmt(U0F16);
    pub struct UartWriter(msp430f5529::USCI_A1_UART_MODE);

    impl uDisplay for U0F16SmallFmt {
        fn fmt<W>(&self, f: &mut Formatter<W>) -> Result<(), W::Error>
        where
            W: uWrite + ?Sized,
        {
            fixed16_fmt_impl(self.0.to_bits(), 16, f)
        }
    }

    impl From<U0F16> for U0F16SmallFmt {
        fn from(i: U0F16) -> Self {
            U0F16SmallFmt(i)
        }
    }

    #[inline(never)]
    fn fixed16_fmt_impl<W>(
        inner: u16,
        num_frac_bits: u8,
        f: &mut Formatter<W>,
    ) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let mut int_buf = [0u8; 6]; // 0 to 65535 (5 plus 1 sign)
        let mut frac_buf = [0u8; 6]; // Down to 0.000031

        let neg = false; // No sign bit
        let frac_mask = 1u16
            .checked_shl(num_frac_bits as u32)
            .unwrap_or(0)
            .wrapping_sub(1);

        let mut int_part: u16;
        let mut frac_part: u16;
        let mut int_buf_start: usize;
        let mut frac_buf_end: usize = 6;

        if neg {
            int_buf_start = 0;

            let twos_comp = inner; // Would be wrapping_abs if signed.

            // Handle most negative value specially.
            if twos_comp == inner {
                int_part = 2u16.pow((15 - num_frac_bits).into());
                frac_part = 0;
            } else {
                int_part = (twos_comp as u16) >> num_frac_bits;
                frac_part = (twos_comp as u16) & frac_mask;
            }
        } else {
            int_buf_start = 1;
            int_part = (inner as u16)
                .checked_shr(num_frac_bits as u32)
                .unwrap_or(0);
            frac_part = (inner as u16) & frac_mask;
        }

        for (offs, i) in int_buf.iter_mut().rev().enumerate() {
            let tens: u8 = (int_part % 10) as u8;
            *i = tens + ('0' as u8);

            int_part = int_part / 10;

            // TODO: Fill?
            if int_part == 0 {
                // offs + 1 because we just finished processing the last _used_
                // cell and store sign in the next cell.
                if neg {
                    int_buf_start = 5 - (offs + 1);
                } else {
                    int_buf_start = 5 - offs;
                }

                break;
            }
        }

        for (offs, fr) in frac_buf.iter_mut().enumerate() {
            let mut tmp_frac_part = frac_part;
            let mut tmp_num_frac_bits = num_frac_bits;

            // We multiply by 10, and then shift to only leave the int part.
            // We need at least 4 bits of room to store the result of multiplying
            // by 10, otherwise it'll get truncated. Temporary shift the fractional
            // part to get 4 bits of free space for the multiply and shift.
            //
            // We don't have this problem for the int part because we can grab
            // the bits shifted out via the remainder.
            if num_frac_bits > 11 {
                tmp_frac_part /= 16;
                tmp_num_frac_bits -= 4;
            }

            let tmp_frac_part = tmp_frac_part * 10;
            let frac_part_in_int_pos = tmp_frac_part >> tmp_num_frac_bits;

            let tens = (frac_part_in_int_pos % 10) as u8;
            *fr = tens + ('0' as u8);

            // Then update and remove part of frac that went past radix point.
            frac_part = frac_part.wrapping_mul(10);
            frac_part = frac_part & frac_mask;

            // TODO: Fill?
            // offs + 1 because we just finished processing the last _used_
            // cell.
            if frac_part == 0 || 6 <= offs + 1 {
                frac_buf_end = offs + 1;
                break;
            }
        }

        if neg {
            int_buf[int_buf_start] = '-' as u8;
        }

        f.write_str(core::str::from_utf8(&int_buf[int_buf_start..]).unwrap())?;
        f.write_char('.')?;
        f.write_str(core::str::from_utf8(&frac_buf[..frac_buf_end]).unwrap())?;

        Ok(())
    }

    impl UartWriter {
        pub fn new(uart: msp430f5529::USCI_A1_UART_MODE) -> Self {
            UartWriter(uart)
        }
    }

    impl uWrite for UartWriter {
        type Error = Infallible;

        fn write_str(&mut self, s: &str) -> Result<(), Infallible> {
            s.as_bytes().iter().for_each(|b| {
                while self.0.uca1ifg.read().uctxifg().bit_is_clear() {}

                self.0.uca1txbuf.write(|w| unsafe { w.bits(*b) });
            });

            Ok(())
        }
    }
}
