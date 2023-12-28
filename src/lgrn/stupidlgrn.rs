use std::marker::PhantomData;

pub type BitVec = Vec<u64>;


#[inline]
pub fn bitvec_set(bitvec: &mut Vec<u64>, pos: usize) {
    bitvec[pos/64] |= 1 << (pos % 64);
}

pub fn bitvec_clear(bitvec: &mut Vec<u64>, pos: usize) {
    bitvec[pos/64] &= ! (1 << (pos % 64));
}

pub fn bitvec_clear_all(bitvec: &mut Vec<u64>) {
    bitvec.iter_mut().for_each(|v| *v = 0 );
}

pub fn bitvec_test(bitvec: &Vec<u64>, pos: usize) -> bool {
    bitvec[pos/64] & (1 << (pos % 64)) != 0
}


pub struct BitIt<'a, const INVERT: bool> {
    slice:      &'a[u64],
    distance:   usize,
    block:      u64
}

impl<'a, const INVERT: bool> BitIt<'a, INVERT> {

    const EMPTY_BLOCK: u64 = if INVERT {!0x0u64} else {0x0u64};

    fn next_block_uwu(&mut self) {
        if self.block != 0 {
            return;
        }
        if let Some((i, &v)) = self.slice.iter().enumerate().skip(1).find(|&(_, &v)| v != Self::EMPTY_BLOCK) {
            self.slice = &self.slice[i..];
            self.block = if INVERT {!v} else {v};
            self.distance += i * 64;
        }
    }
}

impl<'a, const INVERT: bool> Iterator for BitIt<'a, INVERT> {

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

pub fn bitslice_it<'a>(slice: &'a[u64]) -> BitIt<'a, false> {
    if slice.is_empty(){
        BitIt{slice: &[], distance: 0, block: 0}
    } else {
        let mut gwah = BitIt{slice, distance: 0, block: slice[0]};
        gwah.next_block_uwu();
        gwah
    }
}

pub fn bitslice_it_invert<'a>(slice: &'a[u64]) -> BitIt<'a, true> {
    if slice.is_empty(){
        BitIt{slice: &[], distance: 0, block: 0}
    } else {
        let mut gwah = BitIt{slice, distance: 0, block: !slice[0]};
        gwah.next_block_uwu();
        gwah
    }
}

#[derive(Default)]
pub struct IdReg<T> {
    data: BitVec,
    phantom: PhantomData<T>
}

impl<T: Into<usize> + From<usize> + Copy> IdReg<T> {

    #[must_use]
    pub fn create(&mut self) -> Option<T> {
        let pos: usize = bitslice_it(&self.data).next()?;
        bitvec_clear(&mut self.data, pos);
        Some(T::from(pos))
    }

    #[must_use]
    pub fn create_resize(&mut self) -> T {
        if let Some(id) = self.create() {
            return id;
        } else {
            self.resize(self.capacity() * 2);
            return self.create().unwrap();
        }
    }

    pub fn capacity(&self) -> usize { self.data.capacity() * 64 }

    pub fn resize(&mut self, capacity: usize) {
        self.data.resize(capacity / 64 + (capacity % 64 != 0) as usize, ! 0x0u64 );
    }

    pub fn exists(&self, id: T) -> bool {
        let pos: usize = id.into();
        return ! bitvec_test(&self.data, pos);
    }

    pub fn iter_ids<'a>(&'a self) -> IdIt<'a, T> {
        IdIt(bitslice_it_invert(self.data.as_slice()), PhantomData::default())
    }

    pub fn iter_ids_mut<'a>(&'a self) -> IdIt<'a, T> {
        IdIt(bitslice_it_invert(self.data.as_slice()), PhantomData::default())
    }

    pub fn remove(&mut self, id: T) -> bool {
        let prev = bitvec_test(&self.data, id.into());
        bitvec_set(&mut self.data, id.into());
        prev
    }
}

pub struct IdIt<'a, T>(BitIt<'a, true>, PhantomData<T>);

impl<'a, T: From<usize>> Iterator for IdIt<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(T::from(self.0.next()?.into()))
    }
}

macro_rules! id_type {
    ($type_name:ident) => {

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $type_name (pub usize);

        impl Default for $type_name {
            fn default() -> Self {
                $type_name(!0x0usize)
            }
        }

        impl From<usize> for $type_name {
            fn from(item: usize) -> Self {
                $type_name(item)
            }
        }

        impl Into<usize> for $type_name {
            fn into(self) -> usize {
                self.0
            }
        }
    };
}
pub(crate) use id_type;


#[cfg(test)]
mod tests {
    use std::ops::Not;

    use super::*;

    id_type!(FishId);

    #[test]
    fn test_gwahh() {

        let mut fish_ids: IdReg<FishId> = Default::default();

        assert_eq!(fish_ids.capacity(), 0);

        fish_ids.resize(1);
        assert!(fish_ids.exists(FishId(0)).not());

        assert_eq!(fish_ids.data[0], 0xffffffffffffffff);

        let a: FishId = fish_ids.create().unwrap();

        assert_eq!(a, FishId(0));
        assert!(fish_ids.exists(a));

        assert!(fish_ids.exists(FishId(1)).not());

        let b: FishId = fish_ids.create().unwrap();

        assert_eq!(b.0, 1);
        assert!(fish_ids.exists(b));

        assert!(fish_ids.exists(FishId(63)).not());

        for _ in 2..64 {
            let _: FishId = fish_ids.create().unwrap();
        }

        assert!(fish_ids.exists(FishId(63)));
        assert!(fish_ids.create().is_none());

        fish_ids.resize(128);

        assert!(fish_ids.exists(FishId(64)).not());

        assert!(fish_ids.create().is_some());

        assert!(fish_ids.exists(FishId(64)));

        let asdf = 0..64;

        for (a, b) in std::iter::zip(fish_ids.iter_ids(), asdf.into_iter()) {
            println!("{}, {}", a.0, b);
            assert_eq!(a.0, b);
        }
    }
}
