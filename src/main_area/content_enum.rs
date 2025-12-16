use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum Content {
    Edit,
    View,
    NewFile,
    NewTask,
    Graph,
    Blank,
}