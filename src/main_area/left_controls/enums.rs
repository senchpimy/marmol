use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum SortOrder {
    NameAZ,
    NameZA,
    ModifiedNewOld,
    ModifiedOldNew,
    CreatedNewOld,
    CreatedOldNew,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum LeftTab {
    Files,
    Starred,
    Search,
}