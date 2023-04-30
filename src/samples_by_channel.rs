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

impl<T : Copy> SamplesByChannel<T> {
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
