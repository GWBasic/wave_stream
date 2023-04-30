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
