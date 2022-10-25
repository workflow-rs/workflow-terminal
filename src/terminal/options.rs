use web_sys::Element;

pub enum TargetElement {
    Body,
    Element(Element),
    TagName(String),
    Id(String)
}

pub struct Options{
    pub prompt: Option<String>,
    pub element: TargetElement,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            prompt: None,
            element: TargetElement::Body
        }
    }
}

impl Options{
    pub fn new() -> Options{
        Options::default()
    }

    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn with_element(mut self, element: TargetElement) -> Self {
        self.element = element;
        self
    }



    pub fn prompt(&self) -> String {
        self.prompt.as_ref().unwrap_or(&"$ ".to_string()).clone()
    }
}