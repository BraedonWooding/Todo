// A simple but efficient todo list structure

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Debug, Clone)]
pub struct TodoList {
    pub name: String,
    #[serde(skip)]
    pub path: String,
    pub contents: Vec<TodoItem>,
}

impl TodoList {
    pub fn create(name: String, path: String) -> TodoList {
        TodoList {
            name: name,
            path: path,
            contents: vec![],
        }
    }
}

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct TodoItem {
    pub ticked_off: bool,
    pub title: String,
    pub contents: Vec<TodoItem>,
    // as well as a link
}

impl TodoItem {
    pub fn create(title: String) -> TodoItem {
        TodoItem {
            ticked_off: false,
            title: title,
            contents: vec![],
        }
    }
}
