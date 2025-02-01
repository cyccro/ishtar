use std::{collections::HashMap, sync::Arc};

pub type IshtarColors = Arc<HashMap<String, u32>>;

#[derive(Debug)]
pub enum AreaOrder {
    Horizontal,
    Vertical,
}
