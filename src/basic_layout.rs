use serde::{Serialize, Deserialize};

use crate::{element::Element, container::Container};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BasicLayout {
    Stack(Container),
    Row(Container),
    Text(Element),
}
