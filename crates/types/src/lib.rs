use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

pub enum Topic {
    Hello,
}

impl Into<String> for Topic {
    fn into(self) -> String {
        match self {
            Topic::Hello => "hello".into(),
        }
    }
}
