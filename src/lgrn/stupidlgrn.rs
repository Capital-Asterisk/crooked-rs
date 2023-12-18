
pub type BitVec = Vec<u64>;


#[inline]
pub fn bitvec_set(bitvec: &mut Vec<u64>, pos: usize) {
    bitvec[pos/64] |= 1 << (pos % 64);
}

pub fn bitvec_clear(bitvec: &mut Vec<u64>) {
    bitvec.iter_mut().for_each(|v| *v = 0 );
}


pub struct BitIt<'a> {
    slice:      &'a[u64],
    distance:   usize,
    block:      u64
}

impl<'a> BitIt<'a> {
    fn next_block_uwu(&mut self) {
        if self.block != 0 {
            return;
        }
        if let Some((i, &v)) = self.slice.iter().enumerate().skip(1).find(|&(_, &v)| v != 0) {
            self.slice = &self.slice[i..];
            self.block = v;
            self.distance += i * 64;
        }
    }
}

impl<'a> Iterator for BitIt<'a> {

    type Item = usize;
    
    fn next(&mut self) -> Option<Self::Item> {
    
        // thanks europe for help writing this function
        
        if self.block == 0 {
            return None
        }
    
        let out: usize = self.distance + self.block.trailing_zeros() as usize;
        
        self.block &= self.block.wrapping_sub(1);

        self.next_block_uwu();
        
        Some(out)
    }
}

pub fn bitslice_it<'a>(slice: &'a[u64]) -> BitIt<'a> {
    if slice.is_empty(){
        BitIt{slice: &[], distance: 0, block: 0}
    } else {
        let mut gwah =  BitIt{slice, distance: 0, block: slice[0]};
        gwah.next_block_uwu();
        gwah
    }
    
}




