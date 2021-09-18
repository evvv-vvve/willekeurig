use anyhow::{Result, Error, anyhow};

#[derive(Debug, Clone)]
pub struct Identifier {
    namespace: String,
    name: String
}

impl Identifier {
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: String::from(namespace),
            name: String::from(name)
        }
    }

    pub fn from(id: String) -> Result<Self, Error> {
        Self::from_str(&id)
    }

    pub fn from_str(id: &str) -> Result<Self, Error> {
        let splits: Vec<&str> = id.split(":").collect();

        if splits.len() > 2 {
            Err(anyhow!(format!("IDs must contain only 1 colon (1 namespace, 1 identifying name), but '{}' has {}", id, splits.len())))
        } else {
            let id = Self { namespace: String::from(splits[0]), name: String::from(splits[1]) };

            id.validate()?;

            Ok(id)
        }
    }

    pub fn get_namespace(&self) -> String { self.namespace.clone() }
    
    pub fn get_name(&self) -> String { self.name.clone() }

    pub fn as_string(&self) -> String { format!("{}:{}", self.namespace, self.name) }

    pub fn validate(&self) -> Result<(), Error> {
        let valid_chars = "abcdefghijklmnopqrstuvwxyz0123456789_-.";
    
        if self.namespace.chars().all(|c| valid_chars.contains(c)) {
            if self.name.chars().all(|c| valid_chars.contains(c)) {
                Ok(())
            } else {
                Err(anyhow!(format!("identifier name '{}' in namespace '{}' contains invalid characters",
                    self.name, self.namespace)))
            }
        } else {
            Err(anyhow!(format!("identifier namespace '{}' contains invalid characters", self.name)))
        }
    }
}