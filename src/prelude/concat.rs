pub trait Concatenate<T> {
    fn concat(self, item: T) -> Concat;
}

pub struct Concat {
    item: String
}

impl Concat {
    pub fn to_string(self) -> String {
        self.item
    }
}

impl Concatenate<&str> for Concat {
    fn concat(mut self, item: &str) -> Concat {
        self.item.push_str(item);
        self
    }
}

impl Concatenate<String> for Concat {
    fn concat(mut self, item: String) -> Concat {
        self.item.push_str(item.as_str());
        self
    }
}

impl Concatenate<&String> for Concat {
    fn concat(mut self, item: &String) -> Concat {
        self.item.push_str(item.as_str());
        self
    }
}

impl<T> Concatenate<&str> for T
    where T: Into<String>
{
    fn concat(self, item: &str) -> Concat {
        let result = Concat {
            item: self.into()
        };

        result.concat(item)
    }
}

impl<T> Concatenate<String> for T
    where T: Into<String>
{
    fn concat(self, item: String) -> Concat {
        let result = Concat {
            item: self.into()
        };

        result.concat(item.as_str())
    }
}

impl<T> Concatenate<&String> for T
    where T: Into<String>
{
    fn concat(self, item: &String) -> Concat {
        let result = Concat {
            item: self.into()
        };

        result.concat(item.as_str())
    }
}