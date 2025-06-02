use super::FlagVec;

pub struct BoolVec {
    flags: Vec<bool>
}

impl FlagVec for BoolVec {
    type TrueFlagsIter<'a> = TrueFlagsIterator<'a>;

    fn new() -> Self {
        return Self{ flags: Vec::new() }
    }

    fn with_flags(num_flags: usize, value: bool) -> Self {
        return Self{ flags: vec![value; num_flags] }
    }

    fn num_flags(&self) -> usize {
        return self.flags.len()
    }

    fn get_flag(&self, flag: usize) -> bool {
        return self.flags[flag]
    }

    fn set_flag(&mut self, flag: usize, value: bool) {
        self.flags[flag] = value;
    }

    fn add_flags(&mut self, num_flags: usize, value: bool) {
        let new_num_flags: usize = self.flags.len() + num_flags;
        self.flags.resize(new_num_flags, value);
    }

    fn find_a_true_flag(&self) -> Option<usize> {
        for flag in 0..self.flags.len() {
            if self.flags[flag] == true {
                return Some(flag)
            }
        }

        return None
    }

    fn true_flags<'a>(&'a self) -> TrueFlagsIterator<'a> {
        return TrueFlagsIterator::new(&self.flags)
    }
}

pub struct TrueFlagsIterator<'a> {
    bits: &'a Vec<bool>,
    curr_bit: usize,
}

impl <'a> TrueFlagsIterator<'a> {
    pub fn new(bits: &'a Vec<bool>) -> Self {
        return Self { 
            bits,
            curr_bit: 0,
        } 
    }
}

impl <'a> Iterator for TrueFlagsIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.curr_bit >= self.bits.len() {
                return None
            }
            
            if self.bits[self.curr_bit] == true {
                let true_bit: usize = self.curr_bit;
                self.curr_bit += 1;
                return Some(true_bit)
            }

            self.curr_bit += 1;
        }
    }
}