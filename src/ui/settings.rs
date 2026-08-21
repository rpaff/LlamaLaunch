/// Form for adding a new model.
#[derive(Clone)]
pub struct AddModelForm {
    pub name: String,
    pub description: String,
    pub args: String,
}

impl AddModelForm {
    pub fn new() -> Self {
        AddModelForm {
            name: String::new(),
            description: String::new(),
            args: String::new(),
        }
    }
}
