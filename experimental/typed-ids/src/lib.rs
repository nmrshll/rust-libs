use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
};

pub struct Id<ItemT, IdT> {
    pub id: IdT,
    _type: PhantomData<ItemT>,
}
impl<ItemT, IdT> Id<ItemT, IdT> {
    pub fn new(raw: impl Into<IdT>) -> Self {
        Self {
            id: raw.into(),
            _type: PhantomData,
        }
    }
}
impl<ItemT, IdT> Deref for Id<ItemT, IdT> {
    type Target = IdT;
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}
impl<ItemT, IdT: Debug> Debug for Id<ItemT, IdT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}
impl<ItemT, IdT: Display> Display for Id<ItemT, IdT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
impl<ItemT, IdT: Copy> Copy for Id<ItemT, IdT> {}
impl<ItemT, IdT: Clone> Clone for Id<ItemT, IdT> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _type: PhantomData,
        }
    }
}
impl<ItemT, IdT: PartialEq> PartialEq for Id<ItemT, IdT> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<ItemT, IdT: Eq> Eq for Id<ItemT, IdT> {}
impl<ItemT, IdT: Hash> Hash for Id<ItemT, IdT> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self._type.hash(state);
    }
}

impl<ItemT, IdT> From<IdT> for Id<ItemT, IdT> {
    fn from(value: IdT) -> Self {
        Id::new(value)
    }
}
impl<ItemT> From<&str> for Id<ItemT, String> {
    fn from(id: &str) -> Self {
        Id::new(id)
    }
}

#[derive(PartialEq, Eq)]
pub struct ExternalId<ItemT, IdT, IdIss: Issuer> {
    pub issuer: PhantomData<IdIss>,
    pub id: Id<ItemT, IdT>,
}
impl<ItemT, IdT, IdIss: Issuer> ExternalId<ItemT, IdT, IdIss> {
    pub fn new(id: impl Into<IdT>) -> Self {
        Self {
            issuer: PhantomData,
            id: Id::new(id),
        }
    }
    pub fn issuer() -> &'static str {
        IdIss::issuer_id()
    }
}
impl<ItemT, IdT, Iss: Issuer> Deref for ExternalId<ItemT, IdT, Iss> {
    type Target = IdT;
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}
impl<ItemT, IdT: Debug, Iss: Issuer> Debug for ExternalId<ItemT, IdT, Iss> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}
impl<ItemT, IdT: Display, Iss: Issuer> Display for ExternalId<ItemT, IdT, Iss> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
impl<ItemT, IdT: Copy, Iss: Issuer> Copy for ExternalId<ItemT, IdT, Iss> {}
impl<ItemT, IdT: Clone, Iss: Issuer> Clone for ExternalId<ItemT, IdT, Iss> {
    fn clone(&self) -> Self {
        Self {
            issuer: PhantomData,
            id: self.id.clone(),
        }
    }
}
impl<ItemT, IdT: Hash, Iss: Issuer> Hash for ExternalId<ItemT, IdT, Iss> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.issuer.hash(state);
    }
}

impl<ItemT, IdT, Iss: Issuer> From<IdT> for ExternalId<ItemT, IdT, Iss> {
    fn from(id: IdT) -> Self {
        ExternalId::new(id)
    }
}
impl<ItemT, Iss: Issuer> From<&str> for ExternalId<ItemT, String, Iss> {
    fn from(id: &str) -> Self {
        ExternalId::new(id)
    }
}

pub trait Issuer {
    fn issuer_id() -> &'static str;
}
pub trait IsExternalId: Debug + Display + Clone {
    fn issuer(&self) -> &str;
}
impl<ItemT: Clone, IdT: Debug + Display + Clone, Iss: Issuer> IsExternalId
    for ExternalId<ItemT, IdT, Iss>
{
    fn issuer(&self) -> &str {
        Iss::issuer_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MyType;

    type MyTypeId = Id<MyType, String>;

    #[test]
    fn test_deref_id() {
        fn fn_takes_id_as_param(id: &MyTypeId) {
            println!("{id:?}")
        }

        let id = MyTypeId::new("some_string");
        fn_takes_id_as_param(&id);

        fn fn_takes_underlying_str_as_param(id: &str) {
            println!("{id}")
        }
        fn_takes_underlying_str_as_param(&id);
    }

    // #[test]
    // fn test_deref_exernalid() {
    //     #[derive(Copy, Clone, Debug)]
    //     pub struct SomeService;

    //     type SomeServiceMyTypeExtId = ExternalId<MyType, String, SomeService>;

    //     fn fn_takes_id_as_param(id: &SomeServiceMyTypeExtId) {
    //         println!("{id:?}")
    //     }

    //     let id = SomeServiceMyTypeExtId::new(SomeService {}, "some_string");
    //     fn_takes_id_as_param(&id);

    //     fn fn_takes_underlying_str_as_param(id: &str) {
    //         println!("{id}")
    //     }
    //     fn_takes_underlying_str_as_param(&id);
    // }
}
