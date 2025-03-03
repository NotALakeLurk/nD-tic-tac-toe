use std::alloc::{self, Layout};

pub struct Board<'a> {
    dimension: u8, // this dimension will be suitable for out-of-bounds checks as tic-tac-toe
    // boards have sides with a known length (3), i.e. they are hypercubes
    data: &'a mut [u8],
}

#[derive(Debug)]
pub enum IndexError {
    OutOfDimension,
    OutOfBounds,
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds => write!(f, "given position exceeds bounds of board"),
            Self::OutOfDimension => write!(f, "not enough or too few dimensions given in position slice"),
        }
    }
}

impl std::error::Error for IndexError {}

impl Board<'_> {
    const SIZE: u8 = 3; // the length of a tic-tac-toe board, also the number of pieces in a row
    // to win

    pub fn new(dimension: u8) -> Self {
        let length = Self::get_data_length(dimension);
        let layout = Self::get_layout(dimension);

        let ptr = unsafe { alloc::alloc_zeroed(layout) };
        let data = unsafe { std::slice::from_raw_parts_mut(ptr, length) };

        return Self {
            dimension,
            data,
        }
    }

    fn get_data_length(dimension: u8) -> usize {
        usize::from(Self::SIZE).pow(dimension.into()) // length of 3 along each dimension, board is a hypercube
    }


    fn get_layout(dimension: u8) -> Layout {
        let length = Self::get_data_length(dimension);
        Layout::array::<u8>(length).expect("Board dimension too large")
    }

    pub fn get_mut(&mut self, pos: &[u8]) -> Result<&mut u8, IndexError> {
        if pos.len() != self.dimension.into() {
            return Err(IndexError::OutOfDimension); // error here 
        }

        let mut index: usize = 0;
        for (i, val) in pos.iter().enumerate() {
            if *val > Self::SIZE {
                return Err(IndexError::OutOfBounds);
            }

            index += usize::from(Self::SIZE).pow(i.try_into().unwrap()) * usize::from(*val);
        }

        Ok(self.data.get_mut(index).unwrap())
    }

    /// Get the value at a position
    /// note: consider using a result in the future?
    pub fn get(&self, pos: &[u8]) -> Result<u8, IndexError> {
        if pos.len() != self.dimension.into() {
            return Err(IndexError::OutOfDimension); // error here 
        }

        let mut index: usize = 0;
        for (i, val) in pos.iter().enumerate() {
            if *val > Self::SIZE {
                return Err(IndexError::OutOfBounds);
            }

            // index each dimension by adding its offset from 0
            index += usize::from(Self::SIZE).pow(i.try_into().unwrap()) * usize::from(*val);
        }

        Ok(self.data[index])
    }

    pub fn place_piece(&self, player: u8, position: &[u8]) -> Result<(), IndexError> {
        todo!()
    }
}

impl Drop for Board<'_> {
    fn drop(&mut self) {
        let layout = Self::get_layout(self.dimension);

        unsafe {
            alloc::dealloc(self.data.as_mut_ptr(), layout);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_board() {
        let board = Board::new(3);
        let expected = [0_u8; 3_usize.pow(3)];
        let actual = &board.data;
        
        // board data with expected data
        assert!(expected.iter()
            .zip(actual.iter())
            .all(|(a, b)| {a == b}) 
        );
    }

    #[test]
    fn get() {
        let board = Board::new(4);
        let expected = 4;

        // 0 1 2 |  9 10 11 | 18 19 20 \
        // 3 4 5 | 12 13 14 | 21 22 23 |
        // 6 7 8 | 15 16 17 | 24 25 26 / 1st 3d slice of 4d tic tac toe, first item in 2nd slice will be index 27
        board.data[27] = expected; // directly set value
        let acutal = board.get(&[0,0,0,1]).unwrap(); // get previously set position

        assert_eq!(acutal, expected);
    }

    #[test]
    fn get_mut() {
        let mut board = Board::new(6);
        let expected = 7;
        let pos = [0,0,0,0,3,0];

        *board.get_mut(&pos).unwrap() = expected;

        let actual = board.get(&pos).unwrap();

        assert_eq!(actual, expected);
    }
}
