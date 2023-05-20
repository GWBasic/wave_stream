#[derive(Debug, Clone, PartialEq)]
pub struct SamplesByChannel<T> {
    pub front_left: Option<T>,
    pub front_right: Option<T>,
    pub front_center: Option<T>,
    pub low_frequency: Option<T>,
    pub back_left: Option<T>,
    pub back_right: Option<T>,
    pub front_left_of_center: Option<T>,
    pub front_right_of_center: Option<T>,
    pub back_center: Option<T>,
    pub side_left: Option<T>,
    pub side_right: Option<T>,
    pub top_center: Option<T>,
    pub top_front_left: Option<T>,
    pub top_front_center: Option<T>,
    pub top_front_right: Option<T>,
    pub top_back_left: Option<T>,
    pub top_back_center: Option<T>,
    pub top_back_right: Option<T>,
}

impl<T: Copy> SamplesByChannel<T> {
    pub fn new() -> SamplesByChannel<T> {
        SamplesByChannel {
            front_left: None,
            front_right: None,
            front_center: None,
            low_frequency: None,
            back_left: None,
            back_right: None,
            front_left_of_center: None,
            front_right_of_center: None,
            back_center: None,
            side_left: None,
            side_right: None,
            top_center: None,
            top_front_left: None,
            top_front_center: None,
            top_front_right: None,
            top_back_left: None,
            top_back_center: None,
            top_back_right: None,
        }
    }

    pub fn front_left(mut self, sample: T) -> SamplesByChannel<T> {
        self.front_left = Some(sample);

        self
    }

    pub fn front_right(mut self, sample: T) -> SamplesByChannel<T> {
        self.front_right = Some(sample);

        self
    }

    pub fn front_center(mut self, sample: T) -> SamplesByChannel<T> {
        self.front_center = Some(sample);

        self
    }

    pub fn low_frequency(mut self, sample: T) -> SamplesByChannel<T> {
        self.low_frequency = Some(sample);

        self
    }

    pub fn back_left(mut self, sample: T) -> SamplesByChannel<T> {
        self.back_left = Some(sample);

        self
    }

    pub fn back_right(mut self, sample: T) -> SamplesByChannel<T> {
        self.back_right = Some(sample);

        self
    }

    pub fn front_left_of_center(mut self, sample: T) -> SamplesByChannel<T> {
        self.front_left_of_center = Some(sample);

        self
    }

    pub fn front_right_of_center(mut self, sample: T) -> SamplesByChannel<T> {
        self.front_right_of_center = Some(sample);

        self
    }

    pub fn back_center(mut self, sample: T) -> SamplesByChannel<T> {
        self.back_center = Some(sample);

        self
    }

    pub fn side_left(mut self, sample: T) -> SamplesByChannel<T> {
        self.side_left = Some(sample);

        self
    }

    pub fn side_right(mut self, sample: T) -> SamplesByChannel<T> {
        self.side_right = Some(sample);

        self
    }

    pub fn top_center(mut self, sample: T) -> SamplesByChannel<T> {
        self.top_center = Some(sample);

        self
    }

    pub fn top_front_left(mut self, sample: T) -> SamplesByChannel<T> {
        self.top_front_left = Some(sample);

        self
    }

    pub fn top_front_center(mut self, sample: T) -> SamplesByChannel<T> {
        self.top_front_center = Some(sample);

        self
    }

    pub fn top_front_right(mut self, sample: T) -> SamplesByChannel<T> {
        self.top_front_right = Some(sample);

        self
    }

    pub fn top_back_left(mut self, sample: T) -> SamplesByChannel<T> {
        self.top_back_left = Some(sample);

        self
    }

    pub fn top_back_center(mut self, sample: T) -> SamplesByChannel<T> {
        self.top_back_center = Some(sample);

        self
    }

    pub fn top_back_right(mut self, sample: T) -> SamplesByChannel<T> {
        self.top_back_right = Some(sample);

        self
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut vec = Vec::new();

        match self.front_left {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.front_right {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.front_center {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.low_frequency {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.back_left {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.back_right {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.front_left_of_center {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.front_right_of_center {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.back_center {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.side_left {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.side_right {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.top_center {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.top_front_left {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.top_front_center {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.top_front_right {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.top_back_left {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.top_back_center {
            Some(sample) => vec.push(sample),
            None => {}
        };

        match self.top_back_right {
            Some(sample) => vec.push(sample),
            None => {}
        };

        vec
    }
}
