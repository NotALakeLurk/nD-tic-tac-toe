use std::alloc::{self, Layout};

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

#[derive(Debug)]
pub enum PlaceError {
    Unsupported,
    Occupied,
}

impl std::fmt::Display for PlaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "position is not supported by previous pieces"),
            Self::Occupied => write!(f, "position is already occupied"),
        }
    }
}

impl std::error::Error for PlaceError {}

#[derive(Debug)]
pub enum Error {
    PlaceError(PlaceError),
    IndexError(IndexError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaceError(e) => write!(f, "Placement error: {}", e),
            Self::IndexError(e) => write!(f, "Index error: {}", e),
        }
    }
}

impl From<IndexError> for Error {
    fn from(value: IndexError) -> Self {
        Self::IndexError(value)
    }
}

impl From<PlaceError> for Error {
    fn from(value: PlaceError) -> Self {
        Self::PlaceError(value)
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Board<'a> {
    pub dimension: u8, // this dimension will be suitable for out-of-bounds checks as tic-tac-toe
    // boards have sides with a known length (3), i.e. they are hypercubes
    pub data: &'a mut [u8],
}

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

    /// Place a piece on the board, taking into account gravity. Errors if position cannot be
    /// indexed or position is unsupported. Otherwise returns ().
    pub fn place_piece(&mut self, player: u8, position: &[u8]) -> Result<bool, Error> {
        // find the highest non-zero dimension of the position
        let highest = (position.len()-1) - position.iter().rev().position(|&e| e != 0).unwrap_or(position.len()-1);

        // ignore support for placements on the original 2d board
        if highest > 1 {
            // get the position directly "below" this (i.e. the position that supports the current
            // position)
            let mut supporting_pos = Vec::from(position);
            supporting_pos[highest] -= 1;

            // 0 == no piece there == no support for current position
            if self.get(&supporting_pos)? == 0 {
                return Err(PlaceError::Unsupported.into());
            }
        }

        let val = self.get_mut(position)?;

        if *val != 0 {
            return Err(PlaceError::Occupied.into())
        }

        // place the piece
        *val = player;

        let win = self.is_win_at(position)?;

        return Ok(win);
    }

    /// Check to see if there is a win at the given position. Intended to be used directly after
    /// placing a piece to detect a winning move. 
    pub fn is_win_at(&self, pos: &[u8]) -> Result<bool, Error> {
        // the key will be to just check_win_dir each directional vector from the position
        //todo!("Woah! This part is way harder than expected!");
        
        // note: this entire loop may well be completely replaced with a precalculated list of
        // vectors to that point to neighbors (but tic-tac-toe is not a very intensive task rn)

        let len = pos.len();

        // setup a direction vector that we'll use to calculate each neighbor direction
        let mut dir: Vec<i8> = Vec::with_capacity(len);
        unsafe {dir.set_len(len)};
        dir.fill(-1);

        while dir[0] <= 0 {
            // skip the vector that points nowhere, else we'll always measure a win
            // check to see if win and report back if it is
            if !dir.iter().all(|n| *n==0) && self.check_win_dir(pos, &dir)? {
                return Ok(true)
            }

            // add one at the last dimension
            dir[len-1] += 1;

            // propagate addition up dimensions
            for i in (0..dir.len()).rev() {
                // for any component that is beyond pointing to a neighbor |1| or 0
                if dir[i] > 1 {
                    // carry over to next component (checked on next iteration of for loop)
                    dir[i-1] += 1;
                    // reset this component
                    dir[i] = -1;
                }
            }
        }
        
        // no win found
        Ok(false)
    }

    /// Check for a win at a position along a given vector
    fn check_win_dir(&self, pos: &[u8], dir: &[i8]) -> Result<bool, Error> {
        // the key to doing this is realizing that the vector wraps at the edges of the board. For
        // example, if you check along a 1d board: 
        // ```
        // for i in 0..3 {
        //  if (pos+(i*dir)) %euclid 3 != player { return false };
        // }
        // ```
        // This is because if you just add the directional vector you travel a certain path, now
        // when you constrain it to the board it wraps around perfectly!
        // pos = (0,0); dir = (1,-1); =>
        // pos2 = (1,2); pos3 = (2,1);
        // These positions lie on a line and it works out!
        
        if pos.len() != dir.len() {
            return Err(IndexError::OutOfDimension.into());
        }

        let dir = Vec::from(dir);
        let mut pos = Vec::from(pos);

        let player = self.get(&pos)?;
        if player == 0 {
            return Ok(false);
        }

        // 2 steps as the length of the board is 3 in any dimension (we already got the player from
        // the starting position
        for _ in 0..2 {
            // travel along the direction vector
            for i in 0..pos.len() {
                // add each component, limiting to the indexable area (3 in each dimension)
                pos[i] = (pos[i] as i8 + dir[i]).rem_euclid(3).unsigned_abs();
            }

            // check if the position is the player
            if self.get(&pos)? != player {
                return Ok(false);
            }
        }

        return Ok(true);
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

    #[test]
    fn valid_placement() {
        let mut board = Board::new(3);
        let expected = [
            0,0,0,1,0,0,0,0,0,
            0,0,0,1,0,0,0,0,0,
        ];

        board.place_piece(1, &[0,1,0]).unwrap();
        board.place_piece(1, &[0,1,1]).unwrap();

        // compare board data with expected data
        assert!(expected.iter()
            .zip(board.data.iter())
            .all(|(a, b)| {a == b}) 
        );
    }

    #[test]
    fn unsupported_placement() {
        let mut board = Board::new(3);
        let expected = Error::PlaceError(PlaceError::Unsupported);

        let actual = board.place_piece(1, &[0,1,1]).unwrap_err();

        assert!(matches!(actual, expected))
    }

    #[test]
    fn occupied_placement() {
        let mut board = Board::new(3);
        let expected = Error::PlaceError(PlaceError::Occupied);

        board.place_piece(1, &[0,1,0]).unwrap();
        let actual = board.place_piece(1, &[0,1,0]).unwrap_err();

        assert!(matches!(actual, expected))
    }
    
    #[test]
    fn win_dir_straight() {
        let mut board = Board::new(3);
        let expected = true;
        
        board.place_piece(1, &[0,0,0]).unwrap();
        board.place_piece(1, &[0,1,0]).unwrap();
        board.place_piece(1, &[0,2,0]).unwrap();

        let actual = board.check_win_dir(&[0,0,0], &[0,1,0]).unwrap();

        assert_eq!(actual, expected);
    }
    
    #[test]
    fn win_dir_diag() {
        let mut board = Board::new(3);
        let expected = true;
        
        board.place_piece(1, &[0,0,0]).unwrap();
        board.place_piece(1, &[1,1,0]).unwrap();
        board.place_piece(1, &[2,2,0]).unwrap();

        let actual = board.check_win_dir(&[0,0,0], &[1,1,0]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn win_dir_loop() {
        let mut board = Board::new(3);
        let expected = true;
        
        board.place_piece(1, &[0,0,0]).unwrap();
        board.place_piece(1, &[1,1,0]).unwrap();
        board.place_piece(1, &[2,2,0]).unwrap();

        let actual = board.check_win_dir(&[2,2,0], &[1,1,0]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn win_dir_no_win() {
        let mut board = Board::new(3);
        let expected = false;
        
        board.place_piece(1, &[0,0,0]).unwrap();

        let actual = board.check_win_dir(&[0,0,0], &[1,1,0]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn win_no_win() {
        let mut board = Board::new(2);
        let expected = false;

        let actual = board.place_piece(1, &[0,2]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn win_straight() {
        let mut board = Board::new(2);
        let expected = true;

        board.place_piece(1, &[0,0]).unwrap();
        board.place_piece(1, &[0,1]).unwrap();
        let actual = board.place_piece(1, &[0,2]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn win_diag() {
        let mut board = Board::new(2);
        let expected = true;

        board.place_piece(1, &[0,0]).unwrap();
        board.place_piece(1, &[1,1]).unwrap();
        let actual = board.place_piece(1, &[2,2]).unwrap();

        assert_eq!(actual, expected);
    }
}
