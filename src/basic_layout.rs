use serde::{Deserialize, Serialize};

use crate::{container::Container, element::Element};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BasicLayout {
    Stack(Container),
    Row(Container),
    Text(Element),
}
